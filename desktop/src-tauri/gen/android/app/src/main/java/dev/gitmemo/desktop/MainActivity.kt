package dev.gitmemo.desktop

import android.os.Bundle
import android.util.Log
import androidx.activity.enableEdgeToEdge
import java.io.File
import java.io.PrintWriter
import java.io.StringWriter
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

class MainActivity : TauriActivity() {
  companion object {
    private const val TAG = "GitMemo"
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    installCrashHandler()
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }

  private fun installCrashHandler() {
    val defaultHandler = Thread.getDefaultUncaughtExceptionHandler()
    Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
      try {
        val sw = StringWriter()
        throwable.printStackTrace(PrintWriter(sw))
        val stackTrace = sw.toString()

        Log.e(TAG, "FATAL EXCEPTION on thread [${thread.name}]:\n$stackTrace")

        // Write crash log to app-private storage
        val crashDir = File(filesDir, "crash_logs")
        crashDir.mkdirs()
        val timestamp = SimpleDateFormat("yyyy-MM-dd_HH-mm-ss", Locale.US).format(Date())
        val logFile = File(crashDir, "crash_${timestamp}.log")
        logFile.writeText(buildString {
          appendLine("=== GitMemo Android Crash Log ===")
          appendLine("Time: ${SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ssZ", Locale.US).format(Date())}")
          appendLine("Kind: java_uncaught_exception")
          appendLine("Thread: ${thread.name}")
          appendLine("Exception: ${throwable.javaClass.name}: ${throwable.message}")
          appendLine()
          appendLine(stackTrace)
        })
      } catch (e: Exception) {
        Log.e(TAG, "Failed to write crash log", e)
      }

      // Let the default handler finish the process
      defaultHandler?.uncaughtException(thread, throwable)
    }
  }
}
