# Plan 3 manual verification (human checklist)

Agents cannot log in to Proxer or play video; the human does these. Screenshots named below were taken by the agent run and are not committed.

## Setup

```bash
source scripts/android-env.sh
emulator -avd Television_4K            # with a window; keyboard arrows = D-pad, Enter = center, Esc/Backspace = Back
emulator -avd Pixel_10_Pro_XL          # phone (only one emulator at a time)
adb install -r src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk   # TV (i686)
adb install -r /tmp/claude-1000/x86_64.apk                                                          # phone (x86_64)
adb shell monkey -p me.louven.proxer -c android.intent.category.LEANBACK_LAUNCHER 1   # TV launch
adb shell monkey -p me.louven.proxer -c android.intent.category.LAUNCHER 1            # phone launch
adb logcat | grep -i -E 'chromium|RustStdoutStderr|FATAL'
```

Headless (what the agent used): add `-no-window -no-audio -gpu swiftshader_indirect -no-snapshot -no-boot-anim`, poll `adb shell getprop sys.boot_completed`, stop with `adb emu kill`.

## Checklist

- [ ] TV: log in with "Remember me" ticked, force-stop (`adb shell am force-stop me.louven.proxer`), relaunch: the login form is skipped (Android file store).
- [ ] TV, D-pad only: login (Up/Down between username, password, checkbox, Log in; Center activates) -> search -> info -> player. Focus ring visible at every step.
- [ ] Note: focusing a text field opens the soft keyboard, and D-pad then moves inside the keyboard; Back hides it first. Decide whether that is acceptable (typing on a TV needs it anyway).
- [ ] Back: closes fullscreen, then the player, then Info, then (on the login/home screen) exits the app.
- [ ] Playback of a normal stream and of an airing stream. Enter toggles play/pause; seek keys work.
- [ ] Fullscreen button works.
- [ ] Autoplay: if video does not start without pressing Enter/Play, add to `src-tauri/gen/android/app/src/main/java/me/louven/proxer/MainActivity.kt` (class extends `TauriActivity`; confirm the base class in the generated code first):
  ```kotlin
  override fun onWebViewCreate(webView: android.webkit.WebView) {
      super.onWebViewCreate(webView)
      webView.settings.mediaPlaybackRequiresUserGesture = false
  }
  ```
  Rebuild, reinstall, note the result here. If the WebView cannot play the MP4 at all: that is the trigger for the media proxy (spec); stop.
- [ ] TV class: on a real TV (no touchscreen) the `tv` class should give 24px text and 5vh/5vw margin. The stock emulator advertises a touchscreen, so it does NOT apply there; force with `?tv=1` or localStorage `tv=1` to inspect.
- [ ] Phone: touch targets are comfortably tappable (44 px), nothing overlaps the status bar (see Results).

## Results (agent run)

Both runs: install Success, app in foreground (`mCurrentFocus=me.louven.proxer/.MainActivity`), 0 `FATAL EXCEPTION`, no `Refused to`, no CSP messages, no Rust panics.

TV (`Television_4K`, 3840x2160, density 640):
- `tv-login.png`: Login screen renders (dark background; "Proxer" heading, Username and Password inputs, checked "Remember me", Log in button). No focus ring at first; the layout is small, top-left, with no overscan margin: the `tv` class did not take effect because the emulator reports `android.hardware.touchscreen` (maxTouchPoints != 0).
- `tv-seq1-DPAD_DOWN.png`: first Down puts a blue focus ring on Username. `tv-seq3-DPAD_DOWN.png`: Password focused (ring) and the soft keyboard opens; further Down/Right/Up (`tv-seq4..6`) then move the IME key highlight, not the page focus. So page D-pad focus can reach Username and Password, but checkbox/Log in cannot be reached after the keyboard opens without first pressing Back (not verified further, no credentials).
- `tv-back1.png`: Back hides the keyboard, app stays. `tv-back2.png`: second Back leaves the app (launcher focused). Relaunch OK.
- Logcat: only `chromium ... variations_seed_loader.cc: Seed missing signature` (benign WebView noise).

Phone (`Pixel_10_Pro_XL`, 1344x2992, density 480):
- `phone-login.png`: Login renders; inputs ~66 dp tall, button large (touch sizing fine). Concern: edge-to-edge, the status bar clock/icons overlap the "Proxer" heading (no top inset padding).
- Logcat: only `cc/tiles/tile_manager.cc: tile memory limits exceeded` (software GL emulator noise).

Final tested emulator command: `emulator -avd <name> -no-window -no-audio -gpu swiftshader_indirect -no-snapshot -no-boot-anim` (boot < 4 min each, KVM). Both emulators shut down (`adb emu kill`, no qemu left).
