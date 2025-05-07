package com.plugin.`record-stream`

import android.app.Activity
import android.util.Log
import android.graphics.BitmapFactory
import android.graphics.Bitmap
import android.util.Base64
import android.os.Handler
import android.os.Looper
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import org.opencv.android.OpenCVLoader
import org.opencv.android.Utils
import org.opencv.core.Mat
import org.opencv.core.Size
import org.opencv.imgproc.Imgproc
import org.opencv.videoio.VideoWriter
import org.opencv.videoio.VideoWriter.fourcc
import java.io.ByteArrayInputStream
import java.io.File
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import kotlin.Exception

@TauriPlugin
class RecordStreamPlugin(private val activity: Activity) : Plugin(activity) {
    private var videoWriter: VideoWriter? = null
    private var isRecording = false
    private var recordingThread: ExecutorService? = null
    private var mainHandler: Handler = Handler(Looper.getMainLooper())
    
    private var width: Int = 320
    private var height: Int = 240
    private var fps: Double = 30.0
    
    init {
        // Initialize OpenCV
        if (!OpenCVLoader.initDebug()) {
            Log.e("RecordStreamPlugin", "OpenCV initialization failed")
        } else {
            Log.i("RecordStreamPlugin", "OpenCV initialized successfully")
        }
    }
    
    @Command
    fun ping(options: JSObject): JSObject {
        val value = options.getString("value") ?: ""
        Log.i("RecordStreamPlugin", "ping: $value")
        val result = JSObject()
        result.put("value", value)
        return result
    }
    
    @Command
    fun startRecord(options: JSObject): JSObject {
        val filePath = options.getString("file_path") ?: ""
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
                    // Determine fourcc based on file extension
                    val extension = filePath.substringAfterLast('.', "").lowercase()
                    val fourccValue = when (extension) {
                        "mp4" -> fourcc('m', 'p', '4', 'v')
                        "avi" -> fourcc('M', 'J', 'P', 'G')
                        else -> fourcc('m', 'p', '4', 'v') // Default to MP4
                    }
                    
                    // Create VideoWriter
                    videoWriter = VideoWriter(
                        filePath,
                        fourccValue,
                        fps,
                        Size(width.toDouble(), height.toDouble()),
                        true
                    )
                    
                    if (videoWriter?.isOpened == true) {
                        isRecording = true
                        Log.i("RecordStreamPlugin", "Recording started successfully")
                        
                        // Update result on main thread
                        mainHandler.post {
                            result.put("success", true)
                        }
                    } else {
                        Log.e("RecordStreamPlugin", "Failed to open video writer")
                        videoWriter?.release()
                        videoWriter = null
                        
                        // Update result on main thread
                        mainHandler.post {
                            result.put("success", false)
                        }
                    }
                } catch (e: Exception) {
                    Log.e("RecordStreamPlugin", "Error initializing video writer", e)
                    videoWriter?.release()
                    videoWriter = null
                    
                    // Update result on main thread
                    mainHandler.post {
                        result.put("success", false)
                    }
                }
            }
            
            // Return immediately for async initialization
            result.put("success", true)
            return result
            
        } catch (e: Exception) {
            Log.e("RecordStreamPlugin", "Error starting recording", e)
            result.put("success", false)
            return result
        }
    }
    
    @Command
    fun configureRecord(options: JSObject): JSObject {
        width = options.getInt("width", 320)
        height = options.getInt("height", 240)
        fps = options.getDouble("fps", 30.0)
        
        Log.i("RecordStreamPlugin", "Configured recording: ${width}x${height} @ ${fps}fps")
        
        val result = JSObject()
        result.put("success", true)
        return result
    }
    
    @Command
    fun pushFrame(options: JSObject): JSObject {
        val result = JSObject()
        
        if (!isRecording || videoWriter == null) {
            result.put("success", false)
            return result
        }
        
        val b64Png = options.getString("b64_png") ?: ""
        if (b64Png.isEmpty()) {
            result.put("success", false)
            return result
        }
        
        recordingThread?.execute {
            try {
                // Decode base64 string to byte array
                val imageBytes = Base64.decode(b64Png, Base64.DEFAULT)
                
                // Convert to Bitmap
                val bitmap = BitmapFactory.decodeStream(ByteArrayInputStream(imageBytes))
                
                // Resize bitmap if needed
                val scaledBitmap = if (bitmap.width != width || bitmap.height != height) {
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
                
                // Write frame
                synchronized(this) {
                    videoWriter?.write(bgrMat)
                }
                
                // Clean up
                mat.release()
                bgrMat.release()
                if (scaledBitmap != bitmap) {
                    scaledBitmap.recycle()
                }
                bitmap.recycle()
                
                // Update result on main thread
                mainHandler.post {
                    result.put("success", true)
                }
            } catch (e: Exception) {
                Log.e("RecordStreamPlugin", "Error writing video frame", e)
                
                // Update result on main thread
                mainHandler.post {
                    result.put("success", false)
                }
            }
        }
        
        // Return success immediately for async processing
        result.put("success", true)
        return result
    }
    
    @Command
    fun stopRecord(options: JSObject): JSObject {
        Log.i("RecordStreamPlugin", "Stopping recording")
        
        val result = JSObject()
        
        if (!isRecording) {
            result.put("success", false)
            return result
        }
        
        recordingThread?.execute {
            try {
                synchronized(this) {
                    videoWriter?.release()
                    videoWriter = null
                    isRecording = false
                }
                
                Log.i("RecordStreamPlugin", "Recording stopped successfully")
                
                // Update result on main thread
                mainHandler.post {
                    result.put("success", true)
                }
            } catch (e: Exception) {
                Log.e("RecordStreamPlugin", "Error stopping recording", e)
                
                // Update result on main thread
                mainHandler.post {
                    result.put("success", false)
                }
            } finally {
                recordingThread?.shutdown()
                recordingThread = null
            }
        }
        
        // Return success immediately for async cleanup
        result.put("success", true)
        return result
    }
}
