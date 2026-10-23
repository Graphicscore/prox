package me.louven.proxer

import android.app.UiModeManager
import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.content.res.Configuration
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.provider.Settings
import android.webkit.JavascriptInterface
import android.webkit.WebView
import android.view.View
import androidx.activity.enableEdgeToEdge
import androidx.core.content.FileProvider
import java.io.File
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    // Edge-to-edge draws under the status bar: pad the content by the system-bar/cutout insets —
    // unless the player switched immersive mode on (then the video goes under the hidden bars too).
    ViewCompat.setOnApplyWindowInsetsListener(findViewById(android.R.id.content)) { v, insets ->
      // True fullscreen video goes under the hidden bars AND the cutout: player-style full bleed.
      val i = if (immersive) insets.getInsets(WindowInsetsCompat.Type.ime())
              else insets.getInsets(WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout() or WindowInsetsCompat.Type.ime())
      v.setPadding(i.left, i.top, i.right, i.bottom)
      insets
    }
  }

  // Immersive video mode: while it is on, the insets listener above drops the system-bar padding.
  private var immersive = false

  // The TV emulator advertises a touchscreen, so the page cannot tell it is a TV: ask UiModeManager.
  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)
    // Visible to every frame (including embedded players): only harmless getters belong here.
    webView.addJavascriptInterface(object {
      @JavascriptInterface
      fun isTv(): Boolean =
        (getSystemService(Context.UI_MODE_SERVICE) as UiModeManager).currentModeType == Configuration.UI_MODE_TYPE_TELEVISION

      // Self-update: open the system installer for an APK the app downloaded into its own cache. Any frame can call
      // this, so only .apk files inside cacheDir are accepted; Android also refuses an update with another signing key.
      // Returns "started", "permission" (user must allow installs for this app, then retry), or an error code.
      // True fullscreen video: hide status and navigation bars while the player overlay covers the app.
      // The insets listener above drops the system-bar/cutout padding while this is on; transient
      // bars come back on a system-gesture swipe.
      @JavascriptInterface
      fun immersive(on: Boolean) {
        runOnUiThread {
          immersive = on
          val controller = androidx.core.view.WindowCompat.getInsetsController(window, window.decorView)
          controller.systemBarsBehavior = androidx.core.view.WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
          if (on) controller.hide(androidx.core.view.WindowInsetsCompat.Type.systemBars())
          else controller.show(androidx.core.view.WindowInsetsCompat.Type.systemBars())
          android.util.Log.d("ProxPlayer", "immersive(on=$on) requested")
          // Re-run the insets listener: the hide/show itself may not re-dispatch insets (or only after the
          // bar animation), but the content must move under the hidden bars immediately. The listener drops
          // the system-bar padding while `immersive` is on, so both re-applies land without the bars.
          val content = findViewById<View>(android.R.id.content)
          ViewCompat.requestApplyInsets(content)
          content.postDelayed({ ViewCompat.requestApplyInsets(content) }, 400)
        }
      }

      @JavascriptInterface
      fun installUpdate(path: String): String {
        val f = File(path).canonicalFile
        if (!f.path.startsWith(cacheDir.canonicalPath + File.separator) || !f.name.endsWith(".apk") || !f.isFile) return "invalid"
        if (Build.VERSION.SDK_INT >= 26 && !packageManager.canRequestPackageInstalls()) {
          return try {
            startActivity(Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES, Uri.parse("package:$packageName")).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
            "permission"
          } catch (e: ActivityNotFoundException) { "permission-unavailable" }
        }
        val uri = FileProvider.getUriForFile(this@MainActivity, "$packageName.fileprovider", f)
        return try {
          startActivity(Intent(Intent.ACTION_VIEW).setDataAndType(uri, "application/vnd.android.package-archive")
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_ACTIVITY_NEW_TASK))
          "started"
        } catch (e: ActivityNotFoundException) { "installer-unavailable" }
      }
    }, "AndroidTv")
  }
}
