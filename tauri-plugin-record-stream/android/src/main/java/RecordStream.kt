package com.plugin.record_stream

import android.util.Log
import org.opencv.core.CvType
import org.opencv.core.Mat
import org.opencv.core.Size
import org.opencv.imgproc.Imgproc
import org.opencv.videoio.VideoWriter
import java.io.File
import java.util.LinkedList
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors

/**
 * Implementation of video recording functionality using OpenCV
 */
class RecordStream {
    // Default values
    private val defaultWidth = 320
    private val defaultHeight = 240
    private val defaultFps = 30.0
    
    // Plugin state
    private var isRecording = false
    private var videoWriter: VideoWriter? = null
    private var recordingThread: ExecutorService? = null
    private var width = defaultWidth
    private var height = defaultHeight
    private var fps = defaultFps
    
    // Frame buffering system
    private var lastFrame: Mat? = null
    private val frameQueue = LinkedList<Pair<Long, Mat>>()
    private var startTime: Long = 0
    private var frameCount: Int = 0
    private var nextFrameTime: Long = 0
    private var frameIntervalMs: Long = 0
    
    // Effect constants
    companion object {
        const val EFFECT_NONE = 0
        const val EFFECT_GRAYSCALE = 1
        const val EFFECT_CANNY_EDGE = 2
        const val EFFECT_BLUR = 3
    }
    
    private var effectType = EFFECT_NONE
    
    /**
     * Configure recording parameters
     */
    fun configureRecord(width: Int, height: Int, fps: Double): Boolean {
        this.width = width
        this.height = height
        this.fps = fps
        
        Log.i("RecordStream", "Configured recording: ${this.width}x${this.height} @ ${this.fps}fps")
        return true
    }
    
    /**
     * Start recording to the specified file path
     */
    fun startRecord(filePath: String): Boolean {
        Log.i("RecordStream", "Starting recording to $filePath")
        
        if (isRecording) {
            Log.w("RecordStream", "Already recording")
            return false
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
                            Log.i("RecordStream", "Attempting to create H264 VideoWriter: ${width}x${height} @ ${fps}fps")
                            videoWriter = VideoWriter()
                            val fourccCode = fourcc('H', '2', '6', '4')
                            Log.d("RecordStream", "H264 fourcc: $fourccCode")
                            
                            videoWriter?.open(
                                filePath,
                                fourccCode,
                                fps,
                                Size(width.toDouble(), height.toDouble()),
                                true  // isColor=true
                            )
                            
                            Log.d("RecordStream", "H264 VideoWriter opened: ${videoWriter?.isOpened}")
                        } catch (e: Exception) {
                            Log.e("RecordStream", "Failed to initialize H264 encoder", e)
                            useH264 = false
                            useMJPG = true
                        }
                        
                        // If H264 fails, fall back to MJPG in AVI container
                        if (videoWriter?.isOpened() == false) {
                            Log.w("RecordStream", "Failed to open H264 writer, trying MJPG")
                            useH264 = false
                            useMJPG = true
                            // Create new filename with .avi extension
                            val aviFilePath = filePath.replace(".mp4", ".avi")
                            Log.i("RecordStream", "Switching to AVI format at: $aviFilePath")
                            videoWriter?.release()
                            videoWriter = null
                        }
                    }
                    
                    // Use MJPG if specified or if H264 failed
                    if (useMJPG && (videoWriter?.isOpened() == false || videoWriter == null)) {
                        try {
                            val aviFilePath = if (extension == "mp4") {
                                filePath.replace(".mp4", ".avi")
                            } else {
                                filePath
                            }
                            
                            Log.i("RecordStream", "Attempting to create MJPG VideoWriter: ${width}x${height} @ ${fps}fps")
                            videoWriter = VideoWriter()
                            val fourccCode = fourcc('M', 'J', 'P', 'G')
                            Log.d("RecordStream", "MJPG fourcc: $fourccCode")
                            
                            videoWriter?.open(
                                aviFilePath,
                                fourccCode,
                                fps,
                                Size(width.toDouble(), height.toDouble()),
                                true  // isColor=true
                            )
                            
                            Log.d("RecordStream", "MJPG VideoWriter opened: ${videoWriter?.isOpened}")
                        } catch (e: Exception) {
                            Log.e("RecordStream", "Failed to initialize MJPG encoder", e)
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
                        
                        Log.i("RecordStream", "Recording started successfully with frame buffering at ${fps}fps, interval: ${frameIntervalMs}ms")
                        return@execute
                    } else {
                        Log.e("RecordStream", "Failed to open video writer")
                        videoWriter?.release()
                        videoWriter = null
                    }
                } catch (e: Exception) {
                    Log.e("RecordStream", "Error initializing video writer", e)
                    videoWriter?.release()
                    videoWriter = null
                }
            }
            
            return true
        } catch (e: Exception) {
            Log.e("RecordStream", "Error starting recording", e)
            return false
        }
    }
    
    /**
     * Push a frame to the video recording - directly with RGB data
     */
    fun pushFrame(rgbBytes: ByteArray, width: Int, height: Int): Boolean {
        if (!isRecording || videoWriter == null) {
            return false
        }
        
        if (rgbBytes.isEmpty()) {
            return false
        }
        
        // Process frame in a separate thread to avoid blocking UI
        recordingThread?.execute {
            try {
                // Convert raw RGB bytes to Mat
                val rgbMat = Mat(height, width, CvType.CV_8UC3)
                rgbMat.put(0, 0, rgbBytes)
                
                // Convert RGB to BGR for OpenCV
                val bgrMat = Mat()
                Imgproc.cvtColor(rgbMat, bgrMat, Imgproc.COLOR_RGB2BGR)
                
                // If incoming frame size doesn't match video size, resize
                if (width != this.width || height != this.height) {
                    val resized = Mat()
                    Imgproc.resize(bgrMat, resized, Size(this.width.toDouble(), this.height.toDouble()))
                    bgrMat.release()
                    rgbMat.release()
                    
                    // Add frame to queue with timestamp
                    val now = System.currentTimeMillis()
                    frameQueue.add(Pair(now, resized))
                    lastFrame = resized
                } else {
                    rgbMat.release()
                    // Add frame to queue with timestamp
                    val now = System.currentTimeMillis()
                    frameQueue.add(Pair(now, bgrMat))
                    lastFrame = bgrMat
                }
                
                synchronized(this) {
                    if (videoWriter?.isOpened == true) {
                        // Process frame queue to maintain consistent FPS
                        processFrameQueue()
                    }
                }
            } catch (e: Exception) {
                Log.e("RecordStream", "Error processing RGB frame", e)
            }
        }
        
        return true
    }
    
    /**
     * Stop recording
     */
    fun stopRecord(): Boolean {
        Log.i("RecordStream", "Stopping recording")
        
        if (!isRecording) {
            Log.w("RecordStream", "Not recording, nothing to stop")
            return false
        }
        
        recordingThread?.execute {
            try {
                synchronized(this) {
                    // Process any remaining frames in the queue
                    if (videoWriter?.isOpened == true) {
                        Log.d("RecordStream", "Processing remaining frames in queue before stopping")
                        
                        // Process any remaining frames in the queue
                        while (frameQueue.isNotEmpty()) {
                            try {
                                val (_, frame) = frameQueue.removeFirst()
                                videoWriter?.write(frame)
                                frame.release()
                            } catch (e: Exception) {
                                Log.e("RecordStream", "Error processing remaining frame", e)
                            }
                        }
                        
                        // Calculate actual recorded FPS
                        val elapsed = (System.currentTimeMillis() - startTime) / 1000.0
                        val actualFps = if (elapsed > 0) frameCount / elapsed else 0.0
                        Log.i("RecordStream", "Recording stopped. Frames: ${frameCount}, Duration: ${elapsed}s, Actual FPS: ${actualFps}")
                        
                        Log.d("RecordStream", "Releasing video writer")
                        videoWriter?.release()
                        Log.d("RecordStream", "Video writer released")
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
            } catch (e: Exception) {
                Log.e("RecordStream", "Error stopping recording", e)
            } finally {
                try {
                    recordingThread?.shutdown()
                    recordingThread = null
                } catch (e: Exception) {
                    Log.e("RecordStream", "Error shutting down recording thread", e)
                }
            }
        }
        
        return true
    }
    
    /**
     * Helper function to create a fourcc code for video codec
     */
    private fun fourcc(c1: Char, c2: Char, c3: Char, c4: Char): Int {
        return VideoWriter.fourcc(c1, c2, c3, c4)
    }
    
    /**
     * Process the frame queue to maintain consistent frame rate
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
                            Log.d("RecordStream", "Frame arrived ${delay}ms late")
                        }
                    }
                    
                    frameToWrite = frame
                } else if (lastFrame != null) {
                    // No new frames available, duplicate the last one
                    Log.d("RecordStream", "No new frame available, duplicating previous frame")
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
                                Log.d("RecordStream", "Frame ${frameCount}: Target FPS: ${targetFps}, Actual average FPS: ${actualFps}")
                            }
                        } else {
                            Log.e("RecordStream", "Failed to write frame")
                        }
                    } catch (e: Exception) {
                        Log.e("RecordStream", "Error writing frame", e)
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
                    Log.w("RecordStream", "Over 1 second behind in frame processing. Skipping ahead.")
                    nextFrameTime = now + frameIntervalMs
                    break
                }
            }
        } else {
            // Initialize next frame time if not set
            nextFrameTime = now + frameIntervalMs
        }
    }
    
    /**
     * Apply an effect to a frame using OpenCV
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
                Imgproc.Canny(inputMat, outputMat, 100.0, 200.0)
                // Convert back to BGR for video writer
                Imgproc.cvtColor(outputMat, outputMat, Imgproc.COLOR_GRAY2BGR)
            }
            EFFECT_BLUR -> {
                // Apply Gaussian blur
                Imgproc.GaussianBlur(inputMat, outputMat, Size(15.0, 15.0), 0.0)
            }
            else -> {
                // No effect, just clone the input
                inputMat.copyTo(outputMat)
            }
        }
        
        return outputMat
    }
}
