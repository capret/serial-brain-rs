package com.plugin.record_stream

import android.app.Activity
import android.util.Log
import android.graphics.BitmapFactory
import android.graphics.Bitmap
import android.util.Base64
import android.os.Handler
import android.os.Looper
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.annotation.InvokeArg
import app.tauri.plugin.JSObject
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import org.opencv.android.OpenCVLoader
import org.opencv.android.Utils
import org.opencv.core.Mat
import org.opencv.core.Size
import org.opencv.imgproc.Imgproc
import org.opencv.videoio.VideoWriter
import org.opencv.videoio.VideoWriter.fourcc
import org.opencv.videoio.Videoio
import java.io.ByteArrayInputStream
import java.io.File
import java.util.LinkedList
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import kotlin.Exception

// Plugin definition for record stream functionality

@TauriPlugin
class RecordStreamPlugin(private val activity: Activity) : Plugin(activity) {
    // Constants for effect types
    companion object {
        const val EFFECT_NONE = 0
        const val EFFECT_GRAYSCALE = 1
        const val EFFECT_CANNY_EDGE = 2
        const val EFFECT_BLUR = 3
        const val EFFECT_SEPIA = 4
    }
    private var videoWriter: VideoWriter? = null
    private var isRecording = false
    private var recordingThread: ExecutorService? = null
    private var mainHandler: Handler = Handler(Looper.getMainLooper())
    
    // Frame buffering system
    private var lastFrame: Mat? = null
    private val frameQueue = LinkedList<Pair<Long, Mat>>()
    private var startTime: Long = 0
    private var frameCount: Int = 0
    private var nextFrameTime: Long = 0
    private var frameIntervalMs: Long = 0
    
    private var width: Int = 320
    private var height: Int = 240
    private var fps: Double = 30.0
    
    // Effect type to apply to video frames
    private var effectType: Int = EFFECT_NONE
    
    init {
        // Initialize OpenCV
        if (!OpenCVLoader.initDebug()) {
            Log.e("RecordStreamPlugin", "OpenCV initialization failed")
        } else {
            Log.i("RecordStreamPlugin", "OpenCV initialized successfully")
        }
    }
    
    @Command
    fun ping(invoke: Invoke): JSObject {
        // Get the arguments from the invoke object
        val args = invoke.getArgs()
        val value = args.optString("value", "")
        Log.i("RecordStreamPlugin", "ping: $value")
        val result = JSObject()
        result.put("value", value)
        return result
    }
    
    @Command
    fun startRecord(invoke: Invoke): JSObject {
        // Extract the file_path parameter from the invoke object
        val args = invoke.getArgs()
        val filePath = args.optString("file_path", "")
        Log.i("RecordStreamPlugin", "Starting recording to $filePath")
        
        val result = JSObject()
        
        if (isRecording) {
            Log.w("RecordStreamPlugin", "Already recording")
            result.put("success", false)
            return result
        }
        
        try {
            // Ensure parent directory exists
            val file = File(filePath)
            file.parentFile?.mkdirs()
            
            // Create recording thread to avoid blocking UI
            recordingThread = Executors.newSingleThreadExecutor()
            
            // Initialize video writer on a background thread to avoid blocking UI
            recordingThread?.execute {
                try {
                    // Try H264 for MP4 first
                    val extension = filePath.substringAfterLast('.', "").lowercase()
                    var useH264 = extension == "mp4"
                    var useMJPG = extension == "avi"
                    
                    if (useH264) {
                    // Use Android-specific H264 encoder for MP4
                    try {
                        Log.i("RecordStreamPlugin", "Attempting to create H264 VideoWriter: ${width}x${height} @ ${fps}fps")
                        videoWriter = VideoWriter()
                        val fourccCode = fourcc('H', '2', '6', '4')
                        Log.d("RecordStreamPlugin", "H264 fourcc: $fourccCode")
                        
                        videoWriter?.open(
                            filePath,
                            fourccCode,
                            fps,
                            Size(width.toDouble(), height.toDouble()),
                            true  // isColor=true
                        )
                        
                        Log.d("RecordStreamPlugin", "H264 VideoWriter opened: ${videoWriter?.isOpened}")
                    } catch (e: Exception) {
                        Log.e("RecordStreamPlugin", "Failed to initialize H264 encoder", e)
                        useH264 = false
                        useMJPG = true
                    }
                        
                    // If H264 fails, fall back to MJPG in AVI container
                    if (videoWriter?.isOpened == false) {
                        Log.w("RecordStreamPlugin", "Failed to open H264 writer, trying MJPG")
                        useH264 = false
                        useMJPG = true
                        // Create new filename with .avi extension
                        val aviFilePath = filePath.replace(".mp4", ".avi")
                        Log.i("RecordStreamPlugin", "Switching to AVI format at: $aviFilePath")
                        videoWriter?.release()
                        videoWriter = null
                    }
                }
                    
                // Use MJPG if specified or if H264 failed
                if (useMJPG && (videoWriter?.isOpened == false || videoWriter == null)) {
                    try {
                        val aviFilePath = if (extension == "mp4") {
                            filePath.replace(".mp4", ".avi")
                        } else {
                            filePath
                        }
                        
                        Log.i("RecordStreamPlugin", "Attempting to create MJPG VideoWriter: ${width}x${height} @ ${fps}fps")
                        videoWriter = VideoWriter()
                        val fourccCode = fourcc('M', 'J', 'P', 'G')
                        Log.d("RecordStreamPlugin", "MJPG fourcc: $fourccCode")
                        
                        videoWriter?.open(
                            aviFilePath,
                            fourccCode,
                            fps,
                            Size(width.toDouble(), height.toDouble()),
                            true  // isColor=true
                        )
                        
                        Log.d("RecordStreamPlugin", "MJPG VideoWriter opened: ${videoWriter?.isOpened}")
                    } catch (e: Exception) {
                        Log.e("RecordStreamPlugin", "Failed to initialize MJPG encoder", e)
                    }
                }
                
                if (videoWriter?.isOpened == true) {
                        isRecording = true
                        
                        // Initialize frame buffering system
                        frameQueue.clear()
                        lastFrame = null
                        frameCount = 0
                        startTime = System.currentTimeMillis()
                        frameIntervalMs = (1000.0 / fps).toLong() // milliseconds between frames
                        nextFrameTime = startTime + frameIntervalMs
                        
                        Log.i("RecordStreamPlugin", "Recording started successfully with frame buffering at ${fps}fps, interval: ${frameIntervalMs}ms")
                        
                        // Resolve the invoke with success response
                        val response = JSObject()
                        response.put("success", true)
                        mainHandler.post {
                            invoke.resolve(response)
                        }
                    } else {
                        Log.e("RecordStreamPlugin", "Failed to open video writer")
                        videoWriter?.release()
                        videoWriter = null
                        
                        // Resolve the invoke with failure response
                        val response = JSObject()
                        response.put("success", false)
                        mainHandler.post {
                            invoke.resolve(response)
                        }
                    }
                } catch (e: Exception) {
                    Log.e("RecordStreamPlugin", "Error initializing video writer", e)
                    videoWriter?.release()
                    videoWriter = null
                    
                    // Resolve the invoke with error response
                    mainHandler.post {
                        invoke.reject("Failed to initialize video writer: ${e.message}")
                    }
                }
            }
            
            // Don't return anything since we're using invoke.resolve() asynchronously
            return JSObject()
            
        } catch (e: Exception) {
            Log.e("RecordStreamPlugin", "Error starting recording", e)
            invoke.reject("Error starting recording: ${e.message}")
            return JSObject()
        }
    }
    
    @Command
    fun configureRecord(invoke: Invoke): JSObject {
        // Extract parameters from the invoke object
        val args = invoke.getArgs()
        val w = args.optInt("width", 320)
        val h = args.optInt("height", 240)
        val frameRate = args.optDouble("fps", 30.0)
        
        width = w
        height = h
        fps = frameRate
        
        Log.i("RecordStreamPlugin", "Configured recording: ${width}x${height} @ ${fps}fps")
        
        val result = JSObject()
        result.put("success", true)
        return result
    }
    
    @Command
    fun pushFrame(invoke: Invoke): JSObject {
        val result = JSObject()
        
        if (!isRecording || videoWriter == null) {
            result.put("success", false)
            return result
        }
        
        // Extract the b64_png parameter from the invoke object
        val args = invoke.getArgs()
        val b64Png = args.optString("b64_png", "")
        if (b64Png.isEmpty()) {
            result.put("success", false)
            return result
        }
        
        // Process frame in a separate thread to avoid blocking UI
        recordingThread?.execute {
            try {
                // Decode base64 string to byte array
                Log.d("RecordStreamPlugin", "Decoding frame of length: ${b64Png.length}")
                val imageBytes = Base64.decode(b64Png, Base64.DEFAULT)
                
                // Convert to Bitmap
                val bitmap = BitmapFactory.decodeStream(ByteArrayInputStream(imageBytes))
                if (bitmap == null) {
                    Log.e("RecordStreamPlugin", "Failed to decode bitmap from base64")
                    mainHandler.post {
                        val response = JSObject()
                        response.put("success", false)
                        invoke.resolve(response)
                    }
                    return@execute
                }
                
                Log.d("RecordStreamPlugin", "Decoded bitmap: ${bitmap.width}x${bitmap.height}")
                
                // Resize bitmap if needed
                val scaledBitmap = if (bitmap.width != width || bitmap.height != height) {
                    Log.d("RecordStreamPlugin", "Resizing bitmap from ${bitmap.width}x${bitmap.height} to ${width}x${height}")
                    Bitmap.createScaledBitmap(bitmap, width, height, true)
                } else {
                    bitmap
                }
                
                // Convert Bitmap to Mat
                val mat = Mat()
                Utils.bitmapToMat(scaledBitmap, mat)
                
                // Convert RGB to BGR for OpenCV
                val bgrMat = Mat()
                Imgproc.cvtColor(mat, bgrMat, Imgproc.COLOR_RGBA2BGR)
                
                Log.d("RecordStreamPlugin", "Converted mat: ${bgrMat.width()}x${bgrMat.height()}, channels: ${bgrMat.channels()}")
                
                // Apply effect if needed
                val processedMat = if (effectType != EFFECT_NONE) {
                    applyEffect(bgrMat, effectType)
                } else {
                    bgrMat
                }
                
                // Add frame to queue with timestamp
                val now = System.currentTimeMillis()
                synchronized(this) {
                    if (videoWriter?.isOpened == true) {
                        // Store the processed frame in our queue with current timestamp
                        frameQueue.add(Pair(now, processedMat.clone()))
                        
                        // Keep a copy of the last frame for frame duplication if needed
                        if (lastFrame != null) {
                            lastFrame?.release()
                        }
                        lastFrame = processedMat.clone()
                        
                        // Process frame queue to maintain consistent FPS
                        processFrameQueue()
                    } else {
                        Log.e("RecordStreamPlugin", "VideoWriter is not opened")
                        // Clean up the mat since we're not using it
                        processedMat.release()
                    }
                }
                
                // Clean up temporary resources
                // Note: We don't release processedMat here since it's now in our queue
                // We'll release frames when they're removed from the queue
                mat.release()
                bgrMat.release()
                if (scaledBitmap != bitmap) {
                    scaledBitmap.recycle()
                }
                bitmap.recycle()
                
                // Return success response
                val response = JSObject()
                response.put("success", true)
                mainHandler.post {
                    invoke.resolve(response)
                }
            } catch (e: Exception) {
                Log.e("RecordStreamPlugin", "Error writing video frame", e)
                
                // Return error response
                mainHandler.post {
                    invoke.reject("Error writing video frame: ${e.message}")
                }
            }
        }
        
        // Return an empty response since we're using async resolution
        return JSObject()
    }
    
    /**
     * Apply an effect to a frame using OpenCV
     * 
     * @param inputMat The input Mat in BGR format
     * @param effect The effect to apply
     * @return A new Mat with the effect applied, or the input Mat if no effect
     */
    private fun applyEffect(inputMat: Mat, effect: Int): Mat {
        // Create output mat
        val outputMat = Mat()
        
        when (effect) {
            EFFECT_GRAYSCALE -> {
                // Convert to grayscale
                Imgproc.cvtColor(inputMat, outputMat, Imgproc.COLOR_BGR2GRAY)
                // Convert back to BGR for video writer
                Imgproc.cvtColor(outputMat, outputMat, Imgproc.COLOR_GRAY2BGR)
            }
            EFFECT_CANNY_EDGE -> {
                // Apply Canny edge detection
                val grayMat = Mat()
                Imgproc.cvtColor(inputMat, grayMat, Imgproc.COLOR_BGR2GRAY)
                Imgproc.Canny(grayMat, grayMat, 50.0, 150.0)
                Imgproc.cvtColor(grayMat, outputMat, Imgproc.COLOR_GRAY2BGR)
                grayMat.release()
            }
            EFFECT_BLUR -> {
                // Apply Gaussian blur
                Imgproc.GaussianBlur(inputMat, outputMat, Size(15.0, 15.0), 0.0)
            }
            EFFECT_SEPIA -> {
                // Apply sepia filter using a transformation matrix
                val sepiaMat = Mat(3, 3, org.opencv.core.CvType.CV_32F)
                sepiaMat.put(0, 0, 
                    0.272, 0.534, 0.131,
                    0.349, 0.686, 0.168,
                    0.393, 0.769, 0.189
                )
                
                // First convert BGR to RGB
                val rgbMat = Mat()
                Imgproc.cvtColor(inputMat, rgbMat, Imgproc.COLOR_BGR2RGB)
                
                // Apply transformation
                org.opencv.core.Core.transform(rgbMat, outputMat, sepiaMat)
                
                // Convert back to BGR
                Imgproc.cvtColor(outputMat, outputMat, Imgproc.COLOR_RGB2BGR)
                
                rgbMat.release()
                sepiaMat.release()
            }
            else -> {
                // No effect, return the input
                inputMat.copyTo(outputMat)
            }
        }
        
        return outputMat
    }
    
    @Command
    fun setEffect(invoke: Invoke): JSObject {
        // Extract the effect_id parameter from the invoke object
        val args = invoke.getArgs()
        val effectId = args.optInt("effect_id", EFFECT_NONE)
        
        // Validate effect ID
        effectType = when (effectId) {
            EFFECT_GRAYSCALE, EFFECT_CANNY_EDGE, EFFECT_BLUR, EFFECT_SEPIA -> effectId
            else -> EFFECT_NONE
        }
        
        Log.i("RecordStreamPlugin", "Set video effect: $effectType")
        
        val result = JSObject()
        result.put("success", true)
        result.put("current_effect", effectType)
        return result
    }
    
    @Command
    fun stopRecord(invoke: Invoke): JSObject {
        Log.i("RecordStreamPlugin", "Stopping recording")
        
        if (!isRecording) {
            Log.w("RecordStreamPlugin", "Not recording, nothing to stop")
            val response = JSObject()
            response.put("success", false)
            invoke.resolve(response)
            return JSObject()
        }
        
        // Make sure any pending frame is written
        try {
            synchronized(this) {
                if (videoWriter?.isOpened == true) {
                    Log.d("RecordStreamPlugin", "Preparing to finalize video file")
                }
            }
        } catch (e: Exception) {
            Log.e("RecordStreamPlugin", "Error checking video writer state", e)
        }
        
        recordingThread?.execute {
            try {
                synchronized(this) {
                    // Ensure all buffered frames are written
                    if (videoWriter?.isOpened == true) {
                        Log.d("RecordStreamPlugin", "Processing remaining frames in queue before stopping")
                        
                        // Process any remaining frames in the queue
                        while (frameQueue.isNotEmpty()) {
                            try {
                                val (_, frame) = frameQueue.removeFirst()
                                videoWriter?.write(frame)
                                frame.release()
                            } catch (e: Exception) {
                                Log.e("RecordStreamPlugin", "Error processing remaining frame", e)
                            }
                        }
                        
                        // Calculate actual recorded FPS
                        val elapsed = (System.currentTimeMillis() - startTime) / 1000.0
                        val actualFps = if (elapsed > 0) frameCount / elapsed else 0.0
                        Log.i("RecordStreamPlugin", "Recording stopped. Frames: ${frameCount}, Duration: ${elapsed}s, Actual FPS: ${actualFps}")
                        
                        Log.d("RecordStreamPlugin", "Releasing video writer")
                        videoWriter?.release()
                        Log.d("RecordStreamPlugin", "Video writer released")
                    } else {
                        Log.w("RecordStreamPlugin", "VideoWriter was already closed")
                    }
                    
                    // Clean up resources
                    if (lastFrame != null) {
                        lastFrame?.release()
                        lastFrame = null
                    }
                    
                    // Clear frame queue and release all mats
                    while (frameQueue.isNotEmpty()) {
                        val (_, frame) = frameQueue.removeFirst()
                        frame.release()
                    }
                    
                    videoWriter = null
                    isRecording = false
                }
                
                Log.i("RecordStreamPlugin", "Recording stopped successfully")
                
                // Return success response
                val response = JSObject()
                response.put("success", true)
                mainHandler.post {
                    invoke.resolve(response)
                }
            } catch (e: Exception) {
                Log.e("RecordStreamPlugin", "Error stopping recording", e)
                
                // Return error response
                mainHandler.post {
                    invoke.reject("Error stopping recording: ${e.message}")
                }
            } finally {
                try {
                    Log.d("RecordStreamPlugin", "Shutting down recording thread")
                    recordingThread?.shutdown()
                    recordingThread = null
                } catch (e: Exception) {
                    Log.e("RecordStreamPlugin", "Error shutting down recording thread", e)
                }
            }
        }
        
        // Return an empty response since we're using async resolution
        return JSObject()
    }
    
    /**
     * Helper function to create a fourcc code for video codec
     */
    private fun fourcc(c1: Char, c2: Char, c3: Char, c4: Char): Int {
        return VideoWriter.fourcc(c1, c2, c3, c4)
    }
    
    /**
     * Process the frame queue to maintain consistent frame rate
     * Modeled after the desktop Rust implementation
     */
    private fun processFrameQueue() {
        if (videoWriter?.isOpened != true) {
            return
        }
        
        val now = System.currentTimeMillis()
        
        // Only process if we have a reference time
        if (nextFrameTime > 0) {
            // Use a mutable copy of the next frame time
            var currentFrameTime = nextFrameTime
            
            // If it's time to write a frame (or past time)
            while (currentFrameTime <= now) {
                // Get a frame to write - either from queue or repeat last frame
                var frameToWrite: Mat? = null
                
                if (frameQueue.isNotEmpty()) {
                    // Take the oldest frame from the queue
                    val (frameTime, frame) = frameQueue.removeFirst()
                    
                    // Calculate how far behind schedule we are
                    if (frameTime > nextFrameTime) {
                        val delay = frameTime - nextFrameTime
                        if (delay > 100) { // Only log significant delays
                            Log.d("RecordStreamPlugin", "Frame arrived ${delay}ms late")
                        }
                    }
                    
                    frameToWrite = frame
                    
                } else if (lastFrame != null) {
                    // No new frames available, duplicate the last one
                    Log.d("RecordStreamPlugin", "No new frame available, duplicating previous frame")
                    frameToWrite = lastFrame?.clone()
                }
                
                // Write the frame if we have one
                if (frameToWrite != null) {
                    try {
                        val success = videoWriter?.write(frameToWrite) ?: false
                        if (success == true) {
                            frameCount++
                            if (frameCount % 30 == 0) {
                                // Print overall stats periodically
                                val elapsed = (now - startTime) / 1000.0
                                val targetFps = 1000.0 / frameIntervalMs
                                val actualFps = if (elapsed > 0) frameCount / elapsed else 0.0
                                Log.d("RecordStreamPlugin", "Frame ${frameCount}: Target FPS: ${targetFps}, Actual average FPS: ${actualFps}")
                            }
                        } else {
                            Log.e("RecordStreamPlugin", "Failed to write frame")
                        }
                    } catch (e: Exception) {
                        Log.e("RecordStreamPlugin", "Error writing frame", e)
                    } finally {
                        // Make sure to release the frame after writing
                        if (frameToWrite != lastFrame) {
                            frameToWrite.release()
                        }
                    }
                }
                
                // Calculate the next frame time based on the current one
                currentFrameTime += frameIntervalMs
                
                // Update next frame time
                nextFrameTime = currentFrameTime
                
                // Safety check: don't get stuck in the loop if we're very far behind
                // Only write up to 10 frames at once to catch up
                if (now - currentFrameTime > 1000) {
                    Log.w("RecordStreamPlugin", "Over 1 second behind in frame processing. Skipping ahead.")
                    nextFrameTime = now + frameIntervalMs
                    break
                }
            }
        } else {
            // Initialize next frame time if not set
            nextFrameTime = now + frameIntervalMs
        }
    }
}
