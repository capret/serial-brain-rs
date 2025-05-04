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

@InvokeArg
class PingArgs {
  var value: String? = null
}

@TauriPlugin
class ExamplePlugin(private val activity: Activity): Plugin(activity) {
    private val implementation = Example()

    @Command
    fun ping(invoke: Invoke) {
        val args = invoke.parseArgs(PingArgs::class.java)

        val ret = JSObject()
        ret.put("value", implementation.pong(args.value ?: "default value :("))
        invoke.resolve(ret)
    }

    /**
     * Starts the foreground recording service so the process can stay alive when the
     * application is backgrounded. On Android 8.0+ we must use startForegroundService.
     */
    @Command
    fun startService(invoke: Invoke) {
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
    fun stopService(invoke: Invoke) {
        val ctx = activity.applicationContext
        val intent = Intent(ctx, RecordingForegroundService::class.java)
        ctx.stopService(intent)

        invoke.resolve()
    }
}
