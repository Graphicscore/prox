# src-tauri/ - Tauri shell, commands, Android project

## OVERVIEW
Tauri v2 app crate `proxer-app` (lib `proxer_app_lib`): IPC commands over the crates, credential storage, self-updater, plus the tracked Android Gradle project in `gen/android`. Earned its own file: distinct domain (IPC contract, platform split, release plumbing).

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Add/change an IPC command | `src/commands.rs` + `generate_handler!` in `src/lib.rs` | Then `src/lib/api.ts` (+ `demo.ts`) on the UI side |
| Error codes to the UI | `commands.rs` `err()` | Strings: `NotLoggedIn`, `BadCredentials`, `Parse:<what>`, `Http:<n>`, `Network`, `Captcha`, `Unsupported`, `Proxer:<msg>`, `Io` |
| Write actions (note/favor/finish, reminder, mark watched) | `commands.rs` `write()` / `post_action()` | GET page for token, POST JSON endpoint |
| Saved login | `src/creds.rs` | `KeyringStore` (desktop), `FileStore` (Android, 0600, atomic rename), `MemStore` (tests) |
| Self-update | `src/update.rs` | GitHub `releases/latest`, tag `build-<N>`, app version `0.1.<N>` |
| Android JS bridge | `gen/android/app/src/main/java/me/louven/proxer/MainActivity.kt` | `AndroidTv.isTv/immersive/installUpdate` |
| Permissions | `capabilities/default.json` | Only `core:default`, app exit, window fullscreen |
| CSP | `tauri.conf.json` | frames only `*.proxer.me`; media/img any https |

## CONVENTIONS
- Commands return `Result<T, String>`; the string is a machine code the UI maps in `describeError`. Adding a code means updating both sides.
- User-controlled path pieces are allowlisted, never interpolated raw: `list_path`, `action_kind`, `reminder_title`, `valid_lang`, `url_encode`, lang filtered to ASCII alnum.
- Platform split by `#[cfg(target_os = "android")]`, with `allow(dead_code)` on the unused side; keep both branches compiling.
- Unit tests sit in `#[cfg(test)] mod tests` in the same file.
- `gen/android` is tracked except `gen/schemas`, `.gradle`, `build/`, `local.properties`; `keystore.properties` is written by CI only.

## ANTI-PATTERNS
- Do not remove `#![cfg_attr(..., windows_subsystem = "windows")]` in `main.rs` (marked DO NOT REMOVE).
- Do not log secrets: only page titles (`page_title`) are logged on a stream-list failure.
- Do not accept download URLs outside `https://github.com/Graphicscore/prox/releases/download/` (`DOWNLOAD_PREFIX` check in both parse and download).
- Do not move the Android password to Keystore or add backup of it without a decision; plain 0600 app-private file is deliberate.
- Proguard/minify must keep the `AndroidTv` `@JavascriptInterface` methods if ever enabled.
- `login(remember=false)` clears any older saved account; keep that, or the old account auto-logs in next start.

## COMMANDS
```bash
cargo test -p proxer-app
source scripts/android-env.sh && npm run tauri android build -- --apk --split-per-abi --target aarch64
```
