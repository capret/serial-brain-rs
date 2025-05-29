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
import java.io.File
import java.util.LinkedList
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger
import kotlin.math.max

/**
 * RecordStreamPlugin – Tauri Android plugin that receives raw RGB frames
 * (base-64 encoded) from the JS side and writes rotating H.264 MP4 segments.
 *
 * Encoder lifecycle is strictly sequential: we never try to open a new
 * VideoWriter while a previous one is still shutting down, eliminating the
 * MediaCodec/Images-backend race that caused OpenCV assertions.
 */
@TauriPlugin
class RecordStreamPlugin(activity: Activity) : Plugin(activity) {

    /* ------------------------------------------------------------------
     *  Constants / configuration
     * ------------------------------------------------------------------ */
    companion object {
        const val TAG               = "RecordStream"
        const val EFFECT_NONE       = 0
        const val EFFECT_GRAYSCALE  = 1
        const val EFFECT_CANNY_EDGE = 2
        const val EFFECT_BLUR       = 3
        const val EFFECT_SEPIA      = 4
    }

    /* Desired output size / fps – can be changed via configureRecord() */
    private var width  = 320
    private var height = 240
    private var fps    = 30.0

    /* Encoder state */
    private var videoWriter: VideoWriter? = null
    private var isRecording = false

    /* Live frame executor – one per active segment */
    private var recordingExec: ExecutorService? = null

    /* Outstanding cleanup executors from previous segments */
    private val cleanupExecs = ConcurrentHashMap<Int, ExecutorService>()
    private val cleanupIdGen = AtomicInteger(0)

    private val ui = Handler(Looper.getMainLooper())

    /* Frame-rate smoothing */
    private val frameQueue = LinkedList<Pair<Long, Mat>>()
    private var lastFrame: Mat? = null

    private var frameInterval = 0L
    private var nextFrameTime = 0L
    private var startTime     = 0L
    private var frameCount    = 0

    /* Selected effect */
    private var effectType = EFFECT_NONE

    /* ------------------------------------------------------------------ */
    init {
        if (!OpenCVLoader.initDebug()) {
            Log.e(TAG, "OpenCV init failed")
        } else {
            Log.i(TAG, "OpenCV ${Core.VERSION} initialised")
        }
    }

    /* ------------------------------------------------------------------
     *  Public Tauri commands
     * ------------------------------------------------------------------ */

    @Command
    fun ping(invoke: Invoke): JSObject {
        val echo = invoke.getArgs().optString("value", "")
        Log.i(TAG, "ping → $echo")
        return JSObject().apply { put("value", echo) }
    }

    /** Configure resolution/FPS before starting. */
    @Command
    fun configureRecord(invoke: Invoke): JSObject {
        invoke.getArgs().let { a ->
            width  = a.optInt("width",  320)
            height = a.optInt("height", 240)
            fps    = a.optDouble("fps",  30.0)
        }
        Log.i(TAG, "Configured $width x $height @ $fps fps")
        return JSObject().apply { put("success", true) }
    }

    /** Start a new segment – waits until previous clean-ups finish. */
    @Command
    fun startRecord(invoke: Invoke): JSObject {
        val filePath = invoke.getArgs().optString("file_path", "")
        val empty    = JSObject()

        if (filePath.isEmpty()) {
            invoke.reject("file_path missing")
            return empty
        }

        if (isRecording) {
            invoke.reject("Already recording")
            return empty
        }

        // Wait for prior cleanups (not on UI thread!)
        while (cleanupExecs.isNotEmpty()) {
            Log.i(TAG, "Waiting for ${cleanupExecs.size} cleanup thread(s)…")
            Thread.sleep(100)
        }

        recordingExec = Executors.newSingleThreadExecutor()
        recordingExec!!.execute {
            val result = try {
                openWriterAndInit(filePath)
                JSObject().apply { put("success", true) }
            } catch (e: Exception) {
                Log.e(TAG, "startRecord error", e)
                JSObject().apply {
                    put("success", false)
                    put("error", e.message ?: "open failed")
                }
            }
            ui.post { invoke.resolve(result) }
        }
        return empty
    }

    /** Push a base-64 RGB frame from JS. */
    @Command
    fun pushFrame(invoke: Invoke): JSObject {
        val empty = JSObject()

        if (!isRecording || videoWriter?.isOpened != true) {
            invoke.reject("Not recording")
            return empty
        }

        val args    = invoke.getArgs()
        val b64     = args.optString("rgb", "")
        val frameW  = args.optInt("width",  width)
        val frameH  = args.optInt("height", height)

        if (b64.isEmpty()) { invoke.reject("Empty frame"); return empty }

        recordingExec?.execute {
            try {
                /* ---------- decode ---------- */
                val rgbBytes: ByteArray = Base64.decode(b64, Base64.DEFAULT)
                val rgbMat   = Mat(frameH, frameW, CvType.CV_8UC3)
                rgbMat.put(0, 0, rgbBytes)

                /* ---------- resize if needed ---------- */
                val sized = if (frameW != width || frameH != height) {
                    val dst = Mat()
                    Imgproc.resize(
                        rgbMat, dst, Size(width.toDouble(), height.toDouble()),
                        0.0, 0.0, Imgproc.INTER_LINEAR
                    )
                    rgbMat.release(); dst
                } else rgbMat

                /* ---------- RGB → BGR ---------- */
                val bgr = Mat()
                Imgproc.cvtColor(sized, bgr, Imgproc.COLOR_RGB2BGR)
                if (sized !== rgbMat) sized.release()

                /* ---------- effect & enqueue ---------- */
                val finalMat = applyEffectIfNeeded(bgr)
                enqueueFrame(finalMat)

                ui.post { invoke.resolve(JSObject().apply { put("success", true) }) }
            } catch (e: Exception) {
                Log.e(TAG, "pushFrame failed", e)
                ui.post { invoke.reject(e.message ?: "pushFrame failed") }
            }
        }
        return empty
    }

    /** Change visual effect. */
    @Command
    fun setEffect(invoke: Invoke): JSObject {
        val id = invoke.getArgs().optInt("effect_id", EFFECT_NONE)
        effectType = when (id) {
            EFFECT_GRAYSCALE, EFFECT_CANNY_EDGE, EFFECT_BLUR, EFFECT_SEPIA -> id
            else -> EFFECT_NONE
        }
        Log.i(TAG, "Effect set → $effectType")
        return JSObject().apply {
            put("success", true); put("current_effect", effectType)
        }
    }

    /** Stop current recording; cleanup runs in background. */
    @Command
    fun stopRecord(invoke: Invoke): JSObject {
        val empty = JSObject()
        if (!isRecording) { invoke.reject("Not recording"); return empty }

        // Snapshot state
        val writer = videoWriter
        val exec   = recordingExec
        val qCopy  = LinkedList(frameQueue)
        val last   = lastFrame?.clone()
        val id     = cleanupIdGen.incrementAndGet()

        // Stats
        val elapsed = max(1, (System.currentTimeMillis() - startTime))
        Log.i(TAG, "stopRecord → frames=$frameCount, avgFPS=${frameCount*1000.0/elapsed}")

        // Reset for next segment
        isRecording = false
        videoWriter = null
        recordingExec = null
        frameQueue.clear(); lastFrame?.release(); lastFrame = null
        nextFrameTime = 0L; frameCount = 0

        // Kick background cleanup
        val ce = Executors.newSingleThreadExecutor()
        cleanupExecs[id] = ce

        ui.post { invoke.resolve(JSObject().apply { put("success", true) }) }

        ce.execute {
            try {
                Log.i(TAG, "cleanup #$id start")

                while (qCopy.isNotEmpty()) {
                    val (_, m) = qCopy.removeFirst()
                    writer?.write(m)
                    m.release()
                }
                last?.let { writer?.write(it); it.release() }

                exec?.shutdown()
                exec?.awaitTermination(5, TimeUnit.SECONDS)
                writer?.release()

                Log.i(TAG, "cleanup #$id done")
            } catch (e: Exception) {
                Log.e(TAG, "cleanup #$id error", e)
            } finally {
                cleanupExecs.remove(id)
                ce.shutdown()
            }
        }
        return empty
    }

    /* ------------------------------------------------------------------
     *  Internal helpers
     * ------------------------------------------------------------------ */

    /** Opens an MP4/AVI writer and primes all timing state. */
    @Throws(Exception::class)
    private fun openWriterAndInit(originalPath: String) {
        val outFile = File(sanitizeFilename(originalPath))
        outFile.parentFile?.mkdirs()

        var writer = VideoWriter()
        var opened = false
        var usedMJPG = false

        /* -------- try H.264 (MP4) -------- */
        if (outFile.extension.equals("mp4", true)) {
            val h264Fourcc = fourcc('H','2','6','4')
            Log.i(TAG, "Trying H.264 encoder → $outFile")
            writer.open(
                outFile.absolutePath,
                h264Fourcc,
                fps,
                Size(width.toDouble(), height.toDouble()),
                /*isColor=*/true
            )
            opened = writer.isOpened
        }

        /* -------- fallback MJPG (AVI) -------- */
        if (!opened) {
            usedMJPG = true
            val aviFile = if (outFile.extension.equals("mp4", true))
                File(outFile.parent, outFile.nameWithoutExtension + "_a.avi")
            else outFile

            val mjpgFourcc = fourcc('M','J','P','G')
            writer.release()
            writer = VideoWriter()
            Log.i(TAG, "Trying MJPG encoder → $aviFile")
            writer.open(
                aviFile.absolutePath,
                mjpgFourcc,
                fps,
                Size(width.toDouble(), height.toDouble()),
                true
            )
            opened = writer.isOpened
        }

        if (!opened) throw RuntimeException("OpenCV unable to open VideoWriter")

        videoWriter = writer
        isRecording = true

        /* timing state */
        frameInterval = (1000.0 / fps).toLong()
        nextFrameTime = 0L
        startTime     = System.currentTimeMillis()
        frameCount    = 0

        Log.i(
            TAG,
            "Recording started (${if (usedMJPG) "MJPG/AVI" else "H.264/MP4"}) – " +
                    "$width x $height @ $fps"
        )
    }

    /** Appends a processed frame to the queue and runs the scheduler. */
    private fun enqueueFrame(mat: Mat) {
        synchronized(this) {
            frameQueue.add(Pair(System.currentTimeMillis(), mat))
            lastFrame?.release()
            lastFrame = mat.clone()
            processFrameQueue()
        }
    }

    /** Keep output FPS stable by duplicating/ dropping frames as needed. */
    private fun processFrameQueue() {
        val writer = videoWriter ?: return
        if (!writer.isOpened) return

        val now = System.currentTimeMillis()
        if (nextFrameTime == 0L) nextFrameTime = now

        while (nextFrameTime <= now) {
            val frame = if (frameQueue.isNotEmpty())
                frameQueue.removeFirst().second
            else
                lastFrame?.clone()

            frame?.let {
                writer.write(it)
                frameCount++
                if (it !== lastFrame) it.release()
            }
            nextFrameTime += frameInterval

            /* safety cap – don’t try to catch up for >1 s of backlog */
            if (now - nextFrameTime > 1000) {
                nextFrameTime = now + frameInterval
                break
            }
        }
    }

    /** Apply current effect or pass through. */
    private fun applyEffectIfNeeded(src: Mat): Mat {
        if (effectType == EFFECT_NONE) return src

        val dst = Mat()
        when (effectType) {
            EFFECT_GRAYSCALE -> {
                Imgproc.cvtColor(src, dst, Imgproc.COLOR_BGR2GRAY)
                Imgproc.cvtColor(dst, dst, Imgproc.COLOR_GRAY2BGR)
            }
            EFFECT_CANNY_EDGE -> {
                val gray = Mat()
                Imgproc.cvtColor(src, gray, Imgproc.COLOR_BGR2GRAY)
                Imgproc.Canny(gray, gray, 50.0, 150.0)
                Imgproc.cvtColor(gray, dst, Imgproc.COLOR_GRAY2BGR)
                gray.release()
            }
            EFFECT_BLUR -> {
                Imgproc.GaussianBlur(src, dst, Size(15.0, 15.0), 0.0)
            }
            EFFECT_SEPIA -> {
                val kernel = Mat(3, 3, CvType.CV_32F).apply {
                    put(0, 0,
                        0.272, 0.534, 0.131,
                        0.349, 0.686, 0.168,
                        0.393, 0.769, 0.189
                    )
                }
                Core.transform(src, dst, kernel)
                kernel.release()
            }
        }
        src.release()
        return dst
    }

    /** Add “_a” if filename (sans extension) ends with digits. */
    private fun sanitizeFilename(path: String): String {
        val f = File(path)
        return if (f.nameWithoutExtension.matches(Regex("\\d+"))) {
            File(f.parent, f.nameWithoutExtension + "_a.${f.extension}").absolutePath
        } else {
            f.absolutePath
        }
    }
}
