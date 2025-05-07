package com.plugin.record-stream

import android.util.Log

class Example {
    fun pong(value: String): String {
        Log.i("Pong", value)
        return value
    }

    fun startRecord(filePath: String): Boolean {
        // TODO: Implement OpenCV-based video recording to the specified file.
        Log.i("StartRecord", "Recording to $filePath")
        return true
    }
}
