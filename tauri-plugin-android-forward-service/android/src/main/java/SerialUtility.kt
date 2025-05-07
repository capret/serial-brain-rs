package com.plugin.android_forward_service

import android.util.Log

/**
 * Utility class for serial operations
 */
class SerialUtility {
    fun processMessage(value: String): String {
        Log.i("SerialUtility", value)
        return value
    }
}
