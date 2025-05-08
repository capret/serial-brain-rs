package com.plugin.record_stream

import android.app.Activity
import android.util.Base64
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

/**
 * Request argument for configuring recording parameters
 */
@InvokeArg
class ConfigureRecordArgs {
    var width: Int? = null
    var height: Int? = null
    var fps: Double? = null
}

/**
 * Request argument for starting recording
 */
@InvokeArg
class StartRecordArgs {
    var file_path: String? = null
}

/**
 * Request argument for pushing a frame
 */
@InvokeArg
class PushFrameArgs {
    var b64_png: String? = null
}

/**
 * Request argument for setting video effect
 */
@InvokeArg
class SetEffectArgs {
    var effect_type: Int? = null
}

/**
 * Tauri plugin for video recording functionality
 */
@TauriPlugin
class TauriRecordStreamPlugin(private val activity: Activity) : Plugin(activity) {
    // Create an instance of our implementation
    private val recordStream = RecordStream()

    /**
     * Configure recording parameters
     */
    @Command
    fun configureRecord(invoke: Invoke) {
        val args = invoke.parseArgs(ConfigureRecordArgs::class.java)
        
        val width = args.width ?: 320
        val height = args.height ?: 240
        val fps = args.fps ?: 30.0
        
        val success = recordStream.configureRecord(width, height, fps)
        
        val result = JSObject()
        result.put("success", success)
        invoke.resolve(result)
    }

    /**
     * Start video recording
     */
    @Command
    fun startRecord(invoke: Invoke) {
        val args = invoke.parseArgs(StartRecordArgs::class.java)
        val filePath = args.file_path ?: ""
        
        if (filePath.isEmpty()) {
            val result = JSObject()
            result.put("success", false)
            invoke.resolve(result)
            return
        }
        
        // This will handle the async response in the RecordStream implementation
        val success = recordStream.startRecord(filePath)
        
        // Only return initial status - actual success will be resolved asynchronously
        val result = JSObject()
        result.put("success", success)
        invoke.resolve(result)
    }

    /**
     * Push a frame to the recording
     */
    @Command
    fun pushFrame(invoke: Invoke) {
        val args = invoke.parseArgs(PushFrameArgs::class.java)
        val b64Png = args.b64_png ?: ""
        
        if (b64Png.isEmpty()) {
            val result = JSObject()
            result.put("success", false)
            invoke.resolve(result)
            return
        }
        
        val success = recordStream.pushFrame(b64Png)
        
        val result = JSObject()
        result.put("success", success)
        invoke.resolve(result)
    }

    /**
     * Stop video recording
     */
    @Command
    fun stopRecord(invoke: Invoke) {
        val success = recordStream.stopRecord()
        
        val result = JSObject()
        result.put("success", success)
        invoke.resolve(result)
    }

    /**
     * Set video effect
     */
    @Command
    fun setEffect(invoke: Invoke) {
        val args = invoke.parseArgs(SetEffectArgs::class.java)
        val effectType = args.effect_type ?: 0
        
        // This would need to be implemented in the RecordStream class
        // recordStream.setEffect(effectType)
        
        val result = JSObject()
        result.put("success", true)
        invoke.resolve(result)
    }
}
