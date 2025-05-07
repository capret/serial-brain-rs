package com.plugin.android_forward_service

import android.app.Activity
import android.content.Intent
import android.os.Build
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

/**
 * Arguments for the sendMessage command
 */
@InvokeArg
class MessageArgs {
  var value: String? = null
}

/**
 * Tauri plugin for Android forward service functionality
 * This plugin enables background recording and serial data handling on Android devices
 */
@TauriPlugin
class SerialForwardPlugin(private val activity: Activity): Plugin(activity) {
    private val serialUtility = SerialUtility()

    /**
     * Sends a message to the serial utility
     */
    @Command
    fun sendMessage(invoke: Invoke) {
        val args = invoke.parseArgs(MessageArgs::class.java)

        val response = JSObject()
        response.put("value", serialUtility.processMessage(args.value ?: "No message provided"))
        invoke.resolve(response)
    }

    /**
     * Starts the foreground recording service so the process can stay alive when the
     * application is backgrounded. On Android 8.0+ we must use startForegroundService.
     */
    @Command
    fun startRecordingService(invoke: Invoke) {
        val ctx = activity.applicationContext
        val intent = Intent(ctx, RecordingForegroundService::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            ctx.startForegroundService(intent)
        } else {
            ctx.startService(intent)
        }

        invoke.resolve()
    }

    /**
     * Stops the foreground recording service.
     */
    @Command
    fun stopRecordingService(invoke: Invoke) {
        val ctx = activity.applicationContext
        val intent = Intent(ctx, RecordingForegroundService::class.java)
        ctx.stopService(intent)
        
        invoke.resolve()
    }
}
