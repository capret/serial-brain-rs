package com.plugin.record_stream

import android.app.Activity
import android.os.Handler
import android.os.Looper
import android.util.Base64
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import org.opencv.android.OpenCVLoader
import org.opencv.core.Core
import org.opencv.core.CvType
import org.opencv.core.Mat
import org.opencv.core.Size
import org.opencv.imgproc.Imgproc
import org.opencv.videoio.VideoWriter
import org.opencv.videoio.VideoWriter.fourcc
import org.opencv.videoio.Videoio
import java.io.File
import java.util.LinkedList
import java.util.concurrent.CountDownLatch
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.math.max

/**
 * RecordStreamPlugin – Tauri Android plugin that receives raw RGB frames
 * (base‑64 encoded) from the JS side and writes rotating H.264 / MJPG segments.
 */
@TauriPlugin
class RecordStreamPlugin(activity: Activity) : Plugin(activity) {

    /* ------------------------------------------------------------------
     *  Constants / configuration
     * ------------------------------------------------------------------ */
    companion object {
        const val TAG = "RecordStream"
        private const val API_ANDROID = Videoio.CAP_ANDROID   // 2000
        private const val API_ANY     = Videoio.CAP_ANY       // 0

        const val EFFECT_NONE       = 0
        const val EFFECT_GRAYSCALE  = 1
        const val EFFECT_CANNY_EDGE = 2
        const val EFFECT_BLUR       = 3
        const val EFFECT_SEPIA      = 4
    }

    /* Desired output size / fps – set via configureRecord() */
    private var width = 320
    private var height = 240
    private var fps = 30.0

    /* Encoder state */
    private var videoWriter: VideoWriter? = null
    private val recording = AtomicBoolean(false)

    /* Single worker for lifetime of plugin */
    private val ioPool: ExecutorService = Executors.newSingleThreadExecutor()

    /* Cleanup guard */
    private val cleanupRunning = AtomicBoolean(false)

    private val ui = Handler(Looper.getMainLooper())

    /* Frame-rate smoothing */
    private val frameQueue = LinkedList<Pair<Long, Mat>>()
    private var lastFrame: Mat? = null

    private var frameInterval = 0L
    private var nextFrameTime = 0L
    private var startTime = 0L
    private var frameCount = 0

    /* Selected effect */
    private var effectType = EFFECT_NONE

    /* ------------------------------------------------------------------ */
    init {
        if (!OpenCVLoader.initDebug()) Log.e(TAG, "OpenCV init failed")
        else Log.i(TAG, "OpenCV ${Core.VERSION} initialised")
    }

    /* ------------------------------------------------------------------
     *  Public Tauri commands
     * ------------------------------------------------------------------ */

    @Command
    fun configureRecord(invoke: Invoke): JSObject {
        invoke.getArgs().let {
            width = it.optInt("width", 320)
            height = it.optInt("height", 240)
            fps = it.optDouble("fps", 30.0)
        }
        Log.i(TAG, "Configured $width x $height @ $fps fps")
        return JSObject().apply { put("success", true) }
    }

    @Command
    fun startRecord(invoke: Invoke): JSObject {
        val filePath = invoke.getArgs().optString("file_path", "")
        val empty = JSObject()
        if (filePath.isEmpty()) { invoke.reject("file_path missing"); return empty }
        if (recording.get()) { invoke.reject("Already recording"); return empty }
        if (cleanupRunning.get()) { invoke.reject("Cleanup running"); return empty }

        ioPool.execute {
            val result = try {
                openWriterAndInit(filePath)
                JSObject().apply { put("success", true) }
            } catch (e: Exception) {
                Log.e(TAG, "startRecord error", e)
                JSObject().apply { put("success", false); put("error", e.message) }
            }
            ui.post { invoke.resolve(result) }
        }
        return empty
    }

    @Command
    fun pushFrame(invoke: Invoke): JSObject {
        val empty = JSObject()
        if (!recording.get()) { invoke.reject("Not recording"); return empty }
        val args = invoke.getArgs()
        val b64 = args.optString("rgb", "")
        val frameW = args.optInt("width", width)
        val frameH = args.optInt("height", height)
        if (b64.isEmpty()) { invoke.reject("Empty frame"); return empty }

        ioPool.execute {
            try {
                val rgbBytes = Base64.decode(b64, Base64.DEFAULT)
                val rgbMat = Mat(frameH, frameW, CvType.CV_8UC3).apply { put(0, 0, rgbBytes) }
                val resized = if (frameW != width || frameH != height) {
                    val dst = Mat(); Imgproc.resize(rgbMat, dst, Size(width.toDouble(), height.toDouble())); rgbMat.release(); dst
                } else rgbMat
                val bgr = Mat(); Imgproc.cvtColor(resized, bgr, Imgproc.COLOR_RGB2BGR); if (resized !== rgbMat) resized.release()
                val finalMat = applyEffect(bgr)
                enqueueFrame(finalMat)
                ui.post { invoke.resolve(JSObject().apply { put("success", true) }) }
            } catch (e: Exception) { Log.e(TAG, "pushFrame", e); ui.post { invoke.reject(e.message) } }
        }
        return empty
    }

    @Command
    fun stopRecord(invoke: Invoke): JSObject {
        val empty = JSObject(); if (!recording.get()) { invoke.reject("Not recording"); return empty }
        if (!cleanupRunning.compareAndSet(false, true)) { invoke.reject("Cleanup already running"); return empty }

        val latch = CountDownLatch(1)
        ioPool.execute {
            try {
                val writer = videoWriter
                val qCopy = LinkedList(frameQueue)
                val last = lastFrame?.clone()
                recording.set(false); videoWriter = null; frameQueue.clear(); lastFrame?.release(); lastFrame = null; nextFrameTime = 0L
                while (qCopy.isNotEmpty()) { val m = qCopy.removeFirst().second; writer?.write(m); m.release() }
                last?.let { writer?.write(it); it.release() }
                safeClose(writer)
            } finally { cleanupRunning.set(false); latch.countDown() }
        }
        ioPool.execute { latch.await(100, TimeUnit.MILLISECONDS); ui.post { invoke.resolve(JSObject().apply { put("success", true) }) } }
        return empty
    }

    /* ------------------------------------------------------------------
     *  Helpers
     * ------------------------------------------------------------------ */

    private fun openWriterAndInit(origPath: String) {
        val outFile = File(safeName(origPath)); outFile.parentFile?.mkdirs()
        val size = Size(width.toDouble(), height.toDouble())
        val h264 = fourcc('H', '2', '6', '4'); val mjpg = fourcc('M', 'J', 'P', 'G')
        var writer = VideoWriter()
        var opened: Boolean

        // try Android Media backend first
        opened = writer.open(outFile.absolutePath, API_ANDROID, h264, fps, size, true)

        // fallback H.264 via CAP_ANY (may pick MediaCodec again but worth a try)
        if (!opened) opened = writer.open(outFile.absolutePath, API_ANY, h264, fps, size, true)

        // final fallback MJPG
        if (!opened) {
            val aviFile = if (outFile.extension.equals("mp4", true)) File(outFile.parent, outFile.nameWithoutExtension + "_a.avi") else outFile
            opened = writer.open(aviFile.absolutePath, API_ANDROID, mjpg, fps, size, true) ||
                    writer.open(aviFile.absolutePath, API_ANY, mjpg, fps, size, true)
        }
        if (!opened) throw RuntimeException("OpenCV unable to open VideoWriter")

        videoWriter = writer; recording.set(true)
        frameInterval = (1000.0 / fps).toLong(); nextFrameTime = 0L; startTime = System.currentTimeMillis(); frameCount = 0
        Log.i(TAG, "Recording started – $width x $height @ $fps")
    }

    private fun enqueueFrame(m: Mat) { synchronized(this) { frameQueue.add(System.currentTimeMillis() to m); lastFrame?.release(); lastFrame = m.clone(); flushQueue() } }

    private fun flushQueue() {
        val writer = videoWriter ?: return; if (!writer.isOpened) return; val now = System.currentTimeMillis(); if (nextFrameTime == 0L) nextFrameTime = now
        while (nextFrameTime <= now) {
            val f = if (frameQueue.isNotEmpty()) frameQueue.removeFirst().second else lastFrame?.clone(); f?.let { writer.write(it); frameCount++; if (it !== lastFrame) it.release() }; nextFrameTime += frameInterval; if (now - nextFrameTime > 1000) { nextFrameTime = now + frameInterval; break }
        }
    }

    private fun applyEffect(src: Mat): Mat {
        if (effectType == EFFECT_NONE) return src
        val dst = Mat()
        when (effectType) {
            EFFECT_GRAYSCALE -> { Imgproc.cvtColor(src, dst, Imgproc.COLOR_BGR2GRAY); Imgproc.cvtColor(dst, dst, Imgproc.COLOR_GRAY2BGR) }
            EFFECT_CANNY_EDGE -> { val gray = Mat(); Imgproc.cvtColor(src, gray, Imgproc.COLOR_BGR2GRAY); Imgproc.Canny(gray, gray, 50.0, 150.0); Imgproc.cvtColor(gray, dst, Imgproc.COLOR_GRAY2BGR); gray.release() }
            EFFECT_BLUR       -> Imgproc.GaussianBlur(src, dst, Size(15.0, 15.0), 0.0)
            EFFECT_SEPIA      -> { val k = Mat(3, 3, CvType.CV_32F).apply { put(0,0, 0.272,0.534,0.131, 0.349,0.686,0.168, 0.393,0.769,0.189) }; Core.transform(src, dst, k); k.release() }
        }
        src.release(); return dst
    }

    private fun safeClose(w: VideoWriter?) { try { if (w?.isOpened == true) w.release(); Thread.sleep(200) } catch (_: Exception) {} }
    private fun safeName(path: String): String = File(path).let { if (it.nameWithoutExtension.matches(Regex("\\d+"))) File(it.parent, it.nameWithoutExtension + "_a.${it.extension}").absolutePath else it.absolutePath }
}
