package me.louven.proxer

import android.app.UiModeManager
import android.content.Context
import android.content.res.Configuration
import android.os.Bundle
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge
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
    }, "AndroidTv")
  }
}
