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
import androidx.activity.enableEdgeToEdge
import androidx.core.content.FileProvider
import java.io.File
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    // Edge-to-edge draws under the status bar: pad the content by the system-bar/cutout insets.
    ViewCompat.setOnApplyWindowInsetsListener(findViewById(android.R.id.content)) { v, insets ->
      val i = insets.getInsets(WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout() or WindowInsetsCompat.Type.ime())
      v.setPadding(i.left, i.top, i.right, i.bottom)
      insets
    }
  }

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
