package com.plugin.record_stream

import android.app.Activity
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

@InvokeArg
class StartRecordArgs {
  var filePath: String? = null
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

    @Command
    fun startRecord(invoke: Invoke) {
        val args = invoke.parseArgs(StartRecordArgs::class.java)
        val success = implementation.startRecord(args.filePath ?: "")
        val ret = JSObject()
        ret.put("success", success)
        invoke.resolve(ret)
    }
}
