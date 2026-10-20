# Plan 3 manual verification (human checklist)

Agents cannot log in to Proxer or play video; the human does these. Screenshots named below were taken by the agent run and are not committed.

## Setup

```bash
source scripts/android-env.sh
emulator -avd Television_4K            # with a window; keyboard arrows = D-pad, Enter = center, Esc/Backspace = Back
emulator -avd Pixel_10_Pro_XL          # phone (only one emulator at a time)
# Build + copy each APK to a named file (both targets write the same path; see docs/android-setup.md)
npm run tauri android build -- --debug --apk --target i686     # TV
cp src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk proxer-tv-x86.apk
npm run tauri android build -- --debug --apk --target x86_64   # phone
cp src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk proxer-phone-x86_64.apk
adb install -r proxer-tv-x86.apk          # TV
adb install -r proxer-phone-x86_64.apk    # phone
adb shell monkey -p me.louven.proxer -c android.intent.category.LEANBACK_LAUNCHER 1   # TV launch
adb shell monkey -p me.louven.proxer -c android.intent.category.LAUNCHER 1            # phone launch
adb logcat | grep -i -E 'chromium|RustStdoutStderr|FATAL'
```

Headless (what the agent used): add `-no-window -no-audio -gpu swiftshader_indirect -no-snapshot -no-boot-anim`, poll `adb shell getprop sys.boot_completed`, stop with `adb emu kill`.

## Checklist

- [ ] TV: log in with "Remember me" ticked, force-stop (`adb shell am force-stop me.louven.proxer`), relaunch: the login form is skipped (Android file store).
- [ ] TV, D-pad only: login (Up/Down between username, password, checkbox, Log in; Center activates) -> search -> info -> player. Focus ring visible at every step.
- [ ] TV text fields: D-pad focus does NOT open the soft keyboard; Enter/Select on the field opens it; Back hides it and D-pad navigation works again. Submitting an empty username/password shows "Enter your username and password".
- [ ] D-pad reaches Fullscreen: Left/Right move focus along the player bar (Close player <-> Fullscreen), also during "Loading..." and in the embedded player; with the video focused Left/Right seek.
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
- [ ] TV class: the native `AndroidTv.isTv()` check (UiModeManager) applies the `tv` class (24px text, 5vh/5vw margin) on the TV emulator and real TVs even though the emulator advertises a touchscreen; it must NOT apply on the phone.
- [ ] Phone: touch targets are comfortably tappable (44 px), nothing overlaps the status bar, and the soft keyboard does not cover the focused input (ime inset).

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

## Results (Task 7 fixes, agent run)

- TV detection: `MainActivity` exposes `AndroidTv.isTv()` (UiModeManager) via `onWebViewCreate`; `detectTv` order is override, native, heuristic. On `Television_4K` the `tv` class is now active (`t7-tv-login.png`: large text, 5vh/5vw margin) although the emulator advertises a touchscreen.
- TV input lock (`src/lib/tvInput.ts`, TV mode only): D-pad onto Username and onto Password: `mInputShown=false`; DPAD_CENTER / ENTER on the field: `mInputShown=true`; BACK: false. Down x2 more reaches Remember me and Log in (focus ring, `t7-tv-button.png`), so D-pad navigation is no longer trapped.
- Phone (`Pixel_10_Pro_XL`): window-insets listener pads the content view; `t7-phone-login.png` shows the status bar above the page and the "Proxer" heading fully visible.
- Logcat both: 0 `FATAL EXCEPTION`, no CSP/"Refused to", no panics. Both emulators shut down (`adb devices` empty, no qemu).
- Note: the phone AVD was nearly out of storage; the old app was uninstalled before installing the new APK.
- Manifest: the built APK's merged manifest sets `android:allowBackup="false"` and `android:dataExtractionRules` (`aapt2 dump xmltree --file AndroidManifest.xml <apk>`).

## Results (device run with a logged-in account, 2026-10-20)

Phone emulator (Pixel_10_Pro_XL, API 37) and TV emulator (Television_4K, API 36), debug builds from `--split-per-abi` APKs.

Passed on the phone: saved login survives relaunch and an app update that keeps data; search; info page with episode pager; a Normal stream and an Airing stream play with picture, subtitles and sound; fullscreen; Back closes fullscreen, then the player, then Info, then exits the app; an episode without streams shows "No streams (yet)".

Passed on the TV (remote keys sent through `adb`): TV mode applied (`<html class="tv">`, native TV check true); the soft keyboard stays closed when focus arrives on a field and opens only after Select; Back closes the keyboard first; the remote reaches episode 1 and the Normal stream; focus lands on the video once playback starts; Left/Right seek ±10 s and Select pauses/resumes with the video focused; Up from the video reaches the Fullscreen button, Select enters fullscreen, Left/Right still seek there; Back steps fullscreen, player, Info, exit.

Fixed during this run: the remote could not reach Close/Fullscreen because Up from the video jumped to an episode button; the open player now keeps D-pad focus to itself.

Observations, not fixed: on a phone the streams list sits below all 50 episodes; tapping the same stream again while its player is open does not reopen it; one transient HTTP 404 from the airing embed was not retried automatically; screenshots of the emulator can show a stale video frame until the screen repaints (read the player state through WebView DevTools instead).
