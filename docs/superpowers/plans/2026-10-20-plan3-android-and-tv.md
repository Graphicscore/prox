# Android and Android TV Implementation Plan (Plan 3 of 3)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers-extended-cc:subagent-driven-development (recommended) or superpowers-extended-cc:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The same app runs as an Android APK on phones and on Android TV: it builds, installs on the emulators, logs in with a saved password, navigates with a D-pad remote (arrow keys, Enter, Back), and plays video.

**Architecture:** One codebase. Tauri's Android target wraps the existing Rust core and Svelte UI in a WebView (`src-tauri/gen/android`, committed). Three small web modules add TV support: pure spatial navigation (`spatialNav.ts`), TV/touch mode detection (`tv.ts`), and a Back-button handler stack (`back.ts`), wired into `App.svelte`, `Info.svelte` and `Player.svelte`. On Android the credential store is an app-private file instead of the desktop keychain.

**Tech Stack:** Tauri v2 Android (Gradle, NDK, JDK 21), Rust Android targets, Svelte 5 + TypeScript, `node --test` for pure TS tests (Node 26 strips types natively).

**Spec:** `docs/superpowers/specs/2026-10-20-proxer-tauri-frontend-design.md` (Packaging, UI sections). Earlier plans: `2026-10-20-core-and-desktop-ui` and `2026-10-20-plan2-player-and-resolver`. Plan 2 findings that matter here: `<video src>` plays without a media proxy on desktop WebKitGTK; the Android WebView is untested.

## Global Constraints

- Machine facts (checked 2026-10-20): Android SDK at `~/Android/Sdk` with `cmdline-tools/latest`, platforms 30/34/36/36.1/37.0, build-tools up to 36.1.0, **no NDK installed**; `rustup` is **not installed** (Arch `rust` package, no Android targets); default `java` is JDK 27 but Tauri/Gradle need **JDK 21** (`/usr/lib/jvm/java-21-openjdk`); KVM works. AVDs in `~/.android/avd`: `Television_4K` (android-tv, API 36, **x86 32-bit** image, so Rust target `i686-linux-android`) and `Pixel_10_Pro_XL` (API 37 Google Play, x86_64, 16 KB pages). Real devices: none attached.
- Rust targets to build: `aarch64-linux-android` (real devices), `x86_64-linux-android` (phone emulator), `i686-linux-android` (TV emulator).
- Back button: Tauri 2.9+ exposes `onBackButtonPress` in `@tauri-apps/api/app` (installed `@tauri-apps/api` 2.12.4). Registering a handler replaces the default behavior, so the handler must close the app itself when nothing else handles Back.
- The existing desktop behavior must not regress: Escape handling, mouse/keyboard use, the CSP, `f` fullscreen, the 44 existing tests.
- Agents cannot enter the user's Proxer password or log in. Anything needing a login (playback on a device) is verified by the human; agents verify build, install, launch and the login screen through `adb` screenshots only.
- All remote data rules from Plans 1 and 2 stay (https-only embeds, `*.proxer.me`, no secrets in logs).
- Never commit build outputs: `src-tauri/gen/android/**/build/`, `.gradle/`, `local.properties`, `*.apk` stay ignored.

**User decisions (already made):** Tauri app for desktop, Android and Android TV (option 3); saved username and password; both stream types supported.

**Decision recorded for the user to confirm (default chosen, easy to change):** on Android the saved password goes into an app-private file (`<app data dir>/credentials`, mode 0600, readable only by this app on a non-rooted device). The desktop keychain crate has no Android backend. The stronger alternative is the Android Keystore through a Kotlin Tauri plugin (more work, extra Gradle dependency); `ponytail:` upgrade to it if the threat model needs it. Plain-text-in-private-storage is the common minimal approach, but it is weaker than the desktop keychain.

---

### Task 1: Android toolchain setup (human-assisted) and setup doc

**Goal:** A working Android build environment on this machine, documented, with a verification script.

**Files:**
- Create: `docs/android-setup.md`
- Create: `scripts/android-env.sh` (exports the environment variables, meant to be `source`d)

**Acceptance Criteria:**
- [ ] `docs/android-setup.md` lists the exact commands (below) and why each is needed, and states which ones need the user's `sudo`.
- [ ] After the user runs the setup, `source scripts/android-env.sh && rustup target list --installed` lists `aarch64-linux-android`, `x86_64-linux-android` and `i686-linux-android`, and `$NDK_HOME` points to an existing directory.
- [ ] `source scripts/android-env.sh && java -version` reports 21.

**Verify:** `bash -c 'source scripts/android-env.sh && java -version 2>&1 | head -1 && rustup target list --installed && test -d "$NDK_HOME" && echo NDK_OK'` → `openjdk version "21...`, three Android targets, `NDK_OK`

**Steps:**

- [ ] **Step 1: Create `scripts/android-env.sh`** (NDK version is filled in at Step 3):

```bash
#!/usr/bin/env bash
# Source this before any `npm run tauri android ...` command: `source scripts/android-env.sh`
export JAVA_HOME=/usr/lib/jvm/java-21-openjdk   # Gradle/AGP do not support the default JDK 27
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
NDK_VERSION="$(ls "$ANDROID_HOME/ndk" 2>/dev/null | sort -V | tail -1)"
export NDK_HOME="$ANDROID_HOME/ndk/$NDK_VERSION"
export PATH="$JAVA_HOME/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$HOME/.cargo/bin:$PATH"
```

- [ ] **Step 2: Ask the user to run these (they need `sudo`/an interactive prompt; the user types them with the `!` prefix).** Write them into `docs/android-setup.md`:

```bash
# 1. rustup replaces Arch's `rust` package (pacman asks to remove `rust`; accept). Needed to add Android targets.
sudo pacman -S rustup
rustup default stable
rustup target add aarch64-linux-android x86_64-linux-android i686-linux-android
# 2. NDK (pick the newest stable that `sdkmanager --list` shows, e.g. 27.x or 28.x), using JDK 21 for sdkmanager
export JAVA_HOME=/usr/lib/jvm/java-21-openjdk
~/Android/Sdk/cmdline-tools/latest/bin/sdkmanager --licenses
~/Android/Sdk/cmdline-tools/latest/bin/sdkmanager "ndk;<newest stable>"
```

- [ ] **Step 3: After the user confirms, run the Verify command, fix `NDK_VERSION` handling if the directory layout differs, and also run** `source scripts/android-env.sh && cargo build -p proxer-app` **(the desktop build must still work with rustup's toolchain) and `cargo test --workspace`.**

- [ ] **Step 4: Commit.**

```bash
git add docs/android-setup.md scripts/android-env.sh && git commit -m "docs: android toolchain setup"
```

---

### Task 2: Spatial navigation core (pure, tested)

**Goal:** `pickNext` chooses the best focus target for an arrow press; `moveFocus` applies it to the DOM.

**Files:**
- Create: `src/lib/spatialNav.ts`
- Test: `src/lib/spatialNav.test.ts`
- Modify: `package.json` (add a `test` script)

**Acceptance Criteria:**
- [ ] In a 2×2 grid, Right from top-left picks top-right and Down picks bottom-left.
- [ ] A far but aligned candidate beats a near but vertically offset one.
- [ ] With nothing in that direction `pickNext` returns -1; candidates behind the origin are ignored.
- [ ] `npm test` passes; `npm run check` reports 0 errors and 0 warnings; `npm run build` passes.

**Verify:** `npm test && npm run check` → all tests pass, 0 errors

**Steps:**

- [ ] **Step 1: Add the script** to `package.json` `scripts`: `"test": "node --test src/lib/*.test.ts"`. (If `npm run check` rejects the `.ts` import extension used by the tests, add `"allowImportingTsExtensions": true` to `tsconfig.app.json` `compilerOptions`.)

- [ ] **Step 2: Write the failing test `src/lib/spatialNav.test.ts`.**

```ts
import { test } from "node:test";
import assert from "node:assert/strict";
import { pickNext, type Box } from "./spatialNav.ts";

const box = (left: number, top: number, w = 100, h = 50): Box => ({ left, top, right: left + w, bottom: top + h });

test("grid: right and down from top-left", () => {
  const from = box(0, 0);
  const cands = [box(150, 0), box(0, 100), box(150, 100)]; // TR, BL, BR
  assert.equal(pickNext(from, cands, "right"), 0);
  assert.equal(pickNext(from, cands, "down"), 1);
});

test("aligned far candidate beats near offset candidate", () => {
  const from = box(0, 0);
  const far = box(300, 0);          // aligned, 200 away
  const near = box(120, 200);       // 20 away along x but 200 across
  assert.equal(pickNext(from, [near, far], "right"), 1);
});

test("nothing in direction gives -1 and candidates behind are ignored", () => {
  const from = box(300, 300);
  assert.equal(pickNext(from, [box(0, 300), box(300, 0)], "right"), -1);
  assert.equal(pickNext(from, [box(0, 300)], "up"), -1);
  assert.equal(pickNext(from, [], "left"), -1);
});

test("left and up pick the nearest aligned box", () => {
  const from = box(300, 300);
  assert.equal(pickNext(from, [box(0, 300), box(150, 300)], "left"), 1);
  assert.equal(pickNext(from, [box(300, 0), box(300, 150)], "up"), 1);
});
```

- [ ] **Step 3: Run to verify failure.**

Run: `npm test`
Expected: FAIL (cannot find module `./spatialNav.ts`)

- [ ] **Step 4: Implement `src/lib/spatialNav.ts`.**

```ts
export type Dir = "up" | "down" | "left" | "right";
export type Box = { left: number; top: number; right: number; bottom: number };

const cx = (b: Box) => (b.left + b.right) / 2;
const cy = (b: Box) => (b.top + b.bottom) / 2;

/** Index of the best candidate in direction `dir` from `from`, or -1. A candidate must lie beyond the
 *  origin's center in that direction; score = gap along the axis + 2 x offset across it (lower wins). */
export function pickNext(from: Box, cands: Box[], dir: Dir): number {
  let best = -1, bestScore = Infinity;
  cands.forEach((c, i) => {
    let along: number, across: number;
    if (dir === "right") { if (cx(c) <= cx(from)) return; along = Math.max(0, c.left - from.right); across = Math.abs(cy(c) - cy(from)); }
    else if (dir === "left") { if (cx(c) >= cx(from)) return; along = Math.max(0, from.left - c.right); across = Math.abs(cy(c) - cy(from)); }
    else if (dir === "down") { if (cy(c) <= cy(from)) return; along = Math.max(0, c.top - from.bottom); across = Math.abs(cx(c) - cx(from)); }
    else { if (cy(c) >= cy(from)) return; along = Math.max(0, from.top - c.bottom); across = Math.abs(cx(c) - cx(from)); }
    const score = along + 2 * across;
    if (score < bestScore) { bestScore = score; best = i; }
  });
  return best;
}

const FOCUSABLE = 'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])';

const boxOf = (e: Element): Box => { const r = e.getBoundingClientRect(); return { left: r.left, top: r.top, right: r.right, bottom: r.bottom }; };
const visible = (e: HTMLElement) => { const r = e.getBoundingClientRect(); return r.width > 0 && r.height > 0; };

/** Move focus one step in `dir`. Returns true if focus moved (the caller then prevents the default scroll). */
export function moveFocus(dir: Dir, root: ParentNode = document): boolean {
  const cur = document.activeElement as HTMLElement | null;
  const all = [...root.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(visible);
  if (!cur || cur === document.body) { all[0]?.focus(); return all.length > 0; }
  const els = all.filter((e) => e !== cur);
  const i = pickNext(boxOf(cur), els.map(boxOf), dir);
  if (i < 0) return false;
  els[i].focus();
  els[i].scrollIntoView({ block: "nearest" });
  return true;
}
```

- [ ] **Step 5: Run to verify pass, then commit.**

Run: `npm test && npm run check && npm run build`
Expected: 4 tests pass, 0 errors, 0 warnings

```bash
git add src/lib/spatialNav.ts src/lib/spatialNav.test.ts package.json tsconfig.app.json && git commit -m "feat(ui): spatial navigation core"
```

---

### Task 3: TV mode, D-pad, Back stack, and TV/touch layout

**Goal:** Arrow keys move focus everywhere, Back closes the top-most layer (or exits the app), and a TV mode switches to 10-foot sizing.

**Files:**
- Create: `src/lib/tv.ts`, `src/lib/back.ts`, `src/lib/dpad.ts`
- Test: `src/lib/tv.test.ts`, `src/lib/back.test.ts`
- Modify: `src/App.svelte`, `src/lib/Info.svelte`, `src/lib/Player.svelte`

**Acceptance Criteria:**
- [ ] `detectTv`: a `?tv=1`/`tv=on` override forces TV mode on, `0`/`off` forces it off; otherwise TV only when the user agent contains "Android" and there are no touch points.
- [ ] `back.ts`: handlers run most-recent first; the first returning `true` wins; `handleBack()` returns `false` when none handles it; an unregistered handler is not called.
- [ ] On desktop nothing regresses: Escape still closes the player (fullscreen first), `f` still toggles fullscreen, arrows seek inside the player.
- [ ] Arrow keys move focus between buttons/inputs on the login, search, info and player screens (D-pad); Left/Right still move the caret inside text inputs and seek in the player; Enter or Space on the focused `<video>` toggles play/pause.
- [ ] Android Back (`onBackButtonPress`): leaves fullscreen, then closes the player, then returns from Info to Search, otherwise exits the app. On desktop (no such plugin) the registration failing is ignored.
- [ ] In TV mode `<html>` has class `tv`: base font 24 px, a 5% screen margin and a thicker focus ring; with a coarse pointer buttons/inputs are at least 44 px high.
- [ ] `npm test`, `npm run check` (0 errors, 0 warnings) and `npm run build` pass.

**Verify:** `npm test && npm run check && npm run build` → exit 0

**Steps:**

- [ ] **Step 1: Write the failing tests.**

`src/lib/tv.test.ts`:
```ts
import { test } from "node:test";
import assert from "node:assert/strict";
import { detectTv } from "./tv.ts";

const TV_UA = "Mozilla/5.0 (Linux; Android 14; Google TV) AppleWebKit/537.36 Chrome/120 Mobile Safari/537.36";
const DESKTOP_UA = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 Version/17 Safari/605.1.15";

test("override wins", () => {
  assert.equal(detectTv(DESKTOP_UA, 0, "1"), true);
  assert.equal(detectTv(DESKTOP_UA, 0, "on"), true);
  assert.equal(detectTv(TV_UA, 0, "0"), false);
  assert.equal(detectTv(TV_UA, 0, "off"), false);
});

test("auto: Android without touch is a TV", () => {
  assert.equal(detectTv(TV_UA, 0, null), true);
  assert.equal(detectTv(TV_UA, 5, null), false);   // phone
  assert.equal(detectTv(DESKTOP_UA, 0, null), false);
});
```

`src/lib/back.test.ts`:
```ts
import { test } from "node:test";
import assert from "node:assert/strict";
import { handleBack, pushBack } from "./back.ts";

test("most recent handler first, first true wins", () => {
  const calls: string[] = [];
  const off1 = pushBack(() => { calls.push("a"); return true; });
  const off2 = pushBack(() => { calls.push("b"); return true; });
  assert.equal(handleBack(), true);
  assert.deepEqual(calls, ["b"]);
  off2();
  assert.equal(handleBack(), true);
  assert.deepEqual(calls, ["b", "a"]);
  off1();
});

test("falls through handlers that decline; false when nobody handles", () => {
  const calls: string[] = [];
  const off1 = pushBack(() => { calls.push("a"); return true; });
  const off2 = pushBack(() => { calls.push("b"); return false; });
  assert.equal(handleBack(), true);
  assert.deepEqual(calls, ["b", "a"]);
  off1(); off2();
  assert.equal(handleBack(), false);
});
```

- [ ] **Step 2: Run to verify failure.**

Run: `npm test`
Expected: FAIL (cannot find `./tv.ts` / `./back.ts`)

- [ ] **Step 3: Implement `src/lib/tv.ts`.**

```ts
/** TV mode: explicit override ("1"/"on"/"0"/"off" from `?tv=` or localStorage `tv`), else Android with no touchscreen. */
export function detectTv(ua: string, maxTouchPoints: number, override: string | null): boolean {
  if (override === "1" || override === "on") return true;
  if (override === "0" || override === "off") return false;
  return /Android/i.test(ua) && maxTouchPoints === 0;
}

export function currentTv(): boolean {
  let stored: string | null = null;
  try { stored = localStorage.getItem("tv"); } catch { /* storage unavailable */ }
  const q = new URLSearchParams(location.search).get("tv");
  return detectTv(navigator.userAgent, navigator.maxTouchPoints, q ?? stored);
}
```

- [ ] **Step 4: Implement `src/lib/back.ts`.**

```ts
// Layers register a handler while they are open; Back asks the most recent first.
const stack: (() => boolean)[] = [];

export function pushBack(fn: () => boolean): () => void {
  stack.push(fn);
  return () => { const i = stack.lastIndexOf(fn); if (i >= 0) stack.splice(i, 1); };
}

export function handleBack(): boolean {
  for (let i = stack.length - 1; i >= 0; i--) if (stack[i]()) return true;
  return false;
}
```

- [ ] **Step 5: Implement `src/lib/dpad.ts`.**

```ts
import { moveFocus, type Dir } from "./spatialNav";

let seekMode = false;
/** The player turns this on so Left/Right seek instead of moving focus. */
export const setSeekMode = (on: boolean) => { seekMode = on; };

const dirs: Record<string, Dir> = { ArrowUp: "up", ArrowDown: "down", ArrowLeft: "left", ArrowRight: "right" };

export function installDpad(): () => void {
  const onKey = (e: KeyboardEvent) => {
    const dir = dirs[e.key];
    if (!dir || e.defaultPrevented || e.altKey || e.ctrlKey || e.metaKey) return;
    const tag = (e.target as HTMLElement | null)?.tagName;
    const horizontal = dir === "left" || dir === "right";
    if (horizontal && (tag === "INPUT" || tag === "TEXTAREA")) return; // caret movement
    if (horizontal && seekMode) return;                                 // the player seeks
    if (moveFocus(dir)) e.preventDefault();
  };
  window.addEventListener("keydown", onKey);
  return () => window.removeEventListener("keydown", onKey);
}
```

- [ ] **Step 6: Wire `src/App.svelte`.** Add imports and an `onMount` extension (keep the existing auto-login code):

```svelte
  import { exit, onBackButtonPress } from "@tauri-apps/api/app";
  import { currentTv } from "./lib/tv";
  import { handleBack } from "./lib/back";
  import { installDpad } from "./lib/dpad";
```

```ts
  onMount(() => {
    document.documentElement.classList.toggle("tv", currentTv());
    const offDpad = installDpad();
    let offBack: (() => void) | undefined;
    // Android Back: close the top-most layer, else leave the app. Not available on desktop: ignore.
    onBackButtonPress(() => { if (!handleBack()) void exit(0); }).then((l) => (offBack = () => void l.unregister())).catch(() => {});
    return () => { offDpad(); offBack?.(); };
  });
```

Add to the `<style>` block of App.svelte:

```css
  :global(html.tv) { font-size: 24px; }
  :global(html.tv body) { margin: 5vh 5vw; }
  :global(html.tv button:focus-visible, html.tv input:focus-visible) { outline-width: 5px; }
  @media (pointer: coarse) { :global(button), :global(input) { min-height: 44px; } }
```

- [ ] **Step 7: `src/lib/Info.svelte`:** register Back while Info is open. In `<script>` add `import { onMount } from "svelte"; import { pushBack } from "./back";` and

```ts
  onMount(() => pushBack(() => { onback(); return true; }));
```

- [ ] **Step 8: `src/lib/Player.svelte`:** (a) `import { pushBack } from "./back"; import { setSeekMode } from "./dpad";` (b) in `onMount` add `setSeekMode(true); const offBack = pushBack(() => { if (full) setFull(false); else onback(); return true; });` and return a cleanup `() => { offBack(); }` merged with the existing onMount body; in `onDestroy` add `setSeekMode(false);` (c) give the video `tabindex="0"` and focus it once playback starts: `onplaying={() => { retried = false; video?.focus(); }}`; (d) in `key()` extend the Space branch so Enter on the video toggles too:

```ts
    else if ((e.key === " " && tag !== "BUTTON") || (e.key === "Enter" && tag === "VIDEO")) { if (video.paused) video.play().catch(() => {}); else video.pause(); e.preventDefault(); }
```

Because `onMount` already returns nothing today, restructure it as: `onMount(() => { closeBtn?.focus(); section?.scrollIntoView({ block: "center" }); resolve(); setSeekMode(true); const offBack = pushBack(...); return offBack; });`.

- [ ] **Step 9: Verify and commit.**

Run: `npm test && npm run check && npm run build`
Expected: 8 tests pass (4 + 2 + 2), 0 errors, 0 warnings

Run: `cargo build -p proxer-app && cargo test --workspace`
Expected: green (44 tests)

```bash
git add src && git commit -m "feat(ui): TV mode, D-pad navigation, Back handling, touch sizing"
```

---

### Task 4: Android credential store

**Goal:** `FileStore` (app-private file, mode 0600) used on Android, keychain unchanged on desktop.

**Files:**
- Modify: `src-tauri/src/creds.rs`, `src-tauri/src/lib.rs`

**Acceptance Criteria:**
- [ ] `FileStore::new(dir)` saves, loads and clears `user`/`pass` (password may contain newlines; username may not), and `load` returns `None` for a missing or malformed file.
- [ ] On unix the saved file's mode is 0600.
- [ ] `lib.rs` selects `KeyringStore` (desktop) or `FileStore` in `app_data_dir()` (`target_os = "android"`), registering `App` inside `.setup(...)`.
- [ ] Desktop behavior is unchanged: `cargo test --workspace` green, `cargo build -p proxer-app` has no new warnings, and the existing `commands.rs` code still compiles unchanged.

**Verify:** `cargo test -p proxer-app && cargo build -p proxer-app` → exit 0

**Steps:**

- [ ] **Step 1: Write the failing tests** in the `tests` module of `src-tauri/src/creds.rs`:

```rust
    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("proxer-creds-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn file_store_round_trip_and_mode() {
        let dir = temp_dir("rt");
        let s = FileStore::new(&dir);
        assert!(s.load().is_none());
        s.save("user", "pa\nss").unwrap();
        assert_eq!(s.load(), Some(("user".into(), "pa\nss".into())));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(dir.join("credentials")).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
        s.clear();
        assert!(s.load().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_store_malformed_file_is_none() {
        let dir = temp_dir("bad");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("credentials"), "no newline here").unwrap();
        assert!(FileStore::new(&dir).load().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
```

- [ ] **Step 2: Run to verify failure.**

Run: `cargo test -p proxer-app creds`
Expected: FAIL to compile (`FileStore` not found)

- [ ] **Step 3: Implement in `src-tauri/src/creds.rs`.** Rename nothing; add:

```rust
/// App-private credential file for Android, where the `keyring` crate has no backend.
/// ponytail: plain text in the app's private directory (only this app can read it on a non-rooted device);
/// upgrade to the Android Keystore (Kotlin plugin) if the threat model needs it.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub struct FileStore {
    path: std::path::PathBuf,
}

#[cfg_attr(not(target_os = "android"), allow(dead_code))]
impl FileStore {
    pub fn new(dir: &std::path::Path) -> Self {
        Self { path: dir.join("credentials") }
    }
}

impl CredStore for FileStore {
    fn save(&self, user: &str, pass: &str) -> Result<(), String> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(&self.path, format!("{user}\n{pass}")).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.path, std::fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    fn load(&self) -> Option<(String, String)> {
        let s = std::fs::read_to_string(&self.path).ok()?;
        let (u, p) = s.split_once('\n')?;
        Some((u.into(), p.into()))
    }
    fn clear(&self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
```

(Do not log the contents. Create the file with 0600 from the start if the executor prefers `OpenOptions::mode(0o600)`; the test only checks the final mode.)

- [ ] **Step 4: Select the store in `src-tauri/src/lib.rs`.** Replace the `.manage(...)` call with a `.setup(...)`:

```rust
use std::sync::Arc;
use tauri::Manager;

#[cfg(target_os = "android")]
fn make_store(app: &tauri::App) -> Result<Arc<dyn creds::CredStore>, Box<dyn std::error::Error>> {
    Ok(Arc::new(creds::FileStore::new(&app.path().app_data_dir()?)))
}

#[cfg(not(target_os = "android"))]
fn make_store(_app: &tauri::App) -> Result<Arc<dyn creds::CredStore>, Box<dyn std::error::Error>> {
    Ok(Arc::new(creds::KeyringStore))
}
```

and in `run()`:

```rust
        .setup(|app| {
            app.manage(commands::App { client: proxer_client::Client::new("https://proxer.me"), store: make_store(app)? });
            Ok(())
        })
```

(Keep the existing `invoke_handler(...)`, `generate_context!()` and the `#[cfg_attr(mobile, tauri::mobile_entry_point)]` attribute. The existing struct name for the keychain store is whatever `creds.rs` calls it today; use that name.)

- [ ] **Step 5: Verify and commit.**

Run: `cargo test --workspace && cargo build -p proxer-app`
Expected: all pass (44 + 2 new), no new warnings

```bash
git add src-tauri && git commit -m "feat(app): file-based credential store for Android"
```

---

### Task 5: Android project, TV manifest, and debug APKs

**Goal:** `tauri android init`, the manifest declares phone and TV launchers, and debug APKs build for the emulators.

**Files:**
- Create: `src-tauri/gen/android/**` (generated by `tauri android init`, committed minus build outputs), `src-tauri/gen/android/app/src/main/res/drawable-xhdpi/tv_banner.png` (320×180)
- Modify: `src-tauri/gen/android/app/src/main/AndroidManifest.xml`, `.gitignore`
- Create: `docs/android-setup.md` section "Build and install"

**Acceptance Criteria:**
- [ ] Depends on Task 1: `source scripts/android-env.sh && npm run tauri android init` succeeds.
- [ ] The manifest has `<uses-feature android:name="android.software.leanback" android:required="false"/>`, `<uses-feature android:name="android.hardware.touchscreen" android:required="false"/>`, a launcher activity with both `android.intent.category.LAUNCHER` and `android.intent.category.LEANBACK_LAUNCHER`, and `android:banner="@drawable/tv_banner"` on the `<application>`; the `INTERNET` permission is present.
- [ ] `source scripts/android-env.sh && npm run tauri android build -- --debug --apk --target i686` produces an APK (TV emulator) and `--target x86_64` one (phone emulator); the produced APK paths are noted in the doc.
- [ ] Build outputs are git-ignored; `cargo test --workspace` and the desktop build still pass.

**Verify:** `bash -c 'source scripts/android-env.sh && npm run tauri android build -- --debug --apk --target i686 && find src-tauri/gen/android -name "*.apk" | head'` → an APK path is printed

**Steps:**

- [ ] **Step 1: Init.** `source scripts/android-env.sh && npm run tauri android init`. If it asks for anything, use the defaults; the identifier stays `me.louven.proxer`.

- [ ] **Step 2: Ignore build outputs.** Append to `.gitignore` (adjust to what `src-tauri/gen/android/.gitignore` does not already cover): `src-tauri/gen/android/.gradle/`, `src-tauri/gen/android/**/build/`, `src-tauri/gen/android/local.properties`, `*.apk`, `*.aab`.

- [ ] **Step 3: Edit the manifest** `src-tauri/gen/android/app/src/main/AndroidManifest.xml`: inside `<manifest>` (before `<application>`) add

```xml
    <uses-feature android:name="android.software.leanback" android:required="false" />
    <uses-feature android:name="android.hardware.touchscreen" android:required="false" />
```

add `android:banner="@drawable/tv_banner"` to `<application ...>`, and in the main `<activity>`'s `<intent-filter>` add next to the existing LAUNCHER category:

```xml
                <category android:name="android.intent.category.LEANBACK_LAUNCHER" />
```

- [ ] **Step 4: Banner.** Create a 320×180 PNG at `src-tauri/gen/android/app/src/main/res/drawable-xhdpi/tv_banner.png` (any tool: ImageMagick `magick`, Python PIL, or Inkscape; dark background with the app name "Proxer" centered is enough). If no image tool is available, copy `src-tauri/icons/icon.png` there so the build passes and say so in the report.

- [ ] **Step 5: Build both APKs.**

Run: `source scripts/android-env.sh && npm run tauri android build -- --debug --apk --target i686`
Expected: BUILD SUCCESSFUL, an APK path in `src-tauri/gen/android/app/build/outputs/apk/...`

Run: `source scripts/android-env.sh && npm run tauri android build -- --debug --apk --target x86_64`
Expected: same for x86_64

(If Gradle downloads fail or the AGP/JDK versions clash, report the exact error instead of working around it blindly.)

- [ ] **Step 6: Document** the exact build commands, APK paths and `adb install -r <apk>` in `docs/android-setup.md`, then commit.

```bash
git add src-tauri/gen/android .gitignore docs && git commit -m "feat(app): android project with leanback manifest"
```

---

### Task 6: Emulator run, autoplay check, and manual verification

**Goal:** The app installs and launches on both emulators; the login screen renders; the human verifies login, navigation and playback; an autoplay fix is added only if the WebView needs it.

**Files:**
- Create: `docs/superpowers/plans/2026-10-20-plan3-manual-verification.md`
- Modify (only if autoplay is blocked): `src-tauri/gen/android/app/src/main/java/me/louven/proxer/MainActivity.kt`

**Acceptance Criteria:**
- [ ] An agent starts `Television_4K` (and later `Pixel_10_Pro_XL`) headless (`emulator -avd <name> -no-window -no-audio -gpu swiftshader_indirect`), waits for `adb shell getprop sys.boot_completed` = 1, installs the matching APK, launches it, and saves a screenshot (`adb exec-out screencap -p > file.png`) that shows the Login screen (the agent views it with the Read tool and describes it).
- [ ] `adb logcat` for the app shows no crash (`AndroidRuntime: FATAL`) and no CSP errors for the first screen.
- [ ] The manual checklist document exists and covers: login with saved password and relaunch skipping the form (Android file store), D-pad navigation on the TV emulator (arrow keys sent with `adb shell input keyevent KEYCODE_DPAD_DOWN/RIGHT/CENTER`, Back with `KEYCODE_BACK`), Back closing fullscreen then the player then Info then exiting, playback of a normal and an airing stream, the Fullscreen button, and phone touch sizing.
- [ ] If playback does not start by itself on the device, the WebView's `mediaPlaybackRequiresUserGesture` is turned off in `MainActivity.kt` (see Step 4) and the checklist notes it; if the WebView cannot play the MP4 at all, record that as the trigger for adding the media proxy (spec) and stop.

**Verify:** screenshot of the Login screen on `Television_4K` described by the agent; `adb logcat -d | grep -c 'FATAL EXCEPTION'` → `0`

**Steps:**

- [ ] **Step 1: Start the TV emulator headless and wait for boot.**

```bash
source scripts/android-env.sh
nohup emulator -avd Television_4K -no-window -no-audio -gpu swiftshader_indirect -no-snapshot > /tmp/claude-1000/emulator-tv.log 2>&1 &
adb wait-for-device
until [ "$(adb shell getprop sys.boot_completed | tr -d '\r')" = "1" ]; do sleep 3; done
```

- [ ] **Step 2: Install and launch** (APK from Task 5, `i686` build for the TV image; package id `me.louven.proxer`).

```bash
adb install -r <path to the i686 debug apk>
adb shell monkey -p me.louven.proxer -c android.intent.category.LEANBACK_LAUNCHER 1
sleep 8
adb exec-out screencap -p > /tmp/claude-1000/tv-login.png
adb logcat -d | grep -E 'FATAL EXCEPTION|Refused to|CSP' | head
```

Read `/tmp/claude-1000/tv-login.png` and describe what is on screen. Then repeat Steps 1-2 for `Pixel_10_Pro_XL` with the `x86_64` APK and the normal launcher (`monkey -p me.louven.proxer 1`), shutting the TV emulator down first (`adb emu kill`).

- [ ] **Step 3: Write `docs/superpowers/plans/2026-10-20-plan3-manual-verification.md`** with the checklist above (the human logs in; agents never enter the password) and a "Results" section.

- [ ] **Step 4 (only if needed): autoplay.** If, during the human's test, a stream's video does not start without pressing Enter/Play, add to `MainActivity.kt` inside the activity class (check the generated class and base class name first, then use the matching hook):

```kotlin
    override fun onWebViewCreate(webView: android.webkit.WebView) {
        super.onWebViewCreate(webView)
        webView.settings.mediaPlaybackRequiresUserGesture = false
    }
```

Rebuild the APK, reinstall, and note the result in the checklist.

- [ ] **Step 5: Commit.**

```bash
git add docs src-tauri/gen/android && git commit -m "docs: plan 3 manual verification; android launch checked on emulators"
```

---

## Self-review

- **Spec coverage:** Android and Android TV APK (Tasks 1, 5, 6), leanback manifest, `touchscreen required=false`, banner (Task 5), D-pad navigation and focus ring (Tasks 2, 3), Back key (Task 3), Enter toggles play (Task 3), TV 10-foot sizing and 5% overscan margin (Task 3), touch 44 px targets (Task 3), credential storage on Android (Task 4, with the recorded security trade-off), media proxy only if the WebView needs it (Task 6 trigger). Cleartext loopback is not needed (no proxy). Not covered, by design: Apple platforms, store signing, release APK signing, a Kotlin Keystore plugin, open-in-system-browser fallback.
- **Placeholders:** `<newest stable>` NDK version and `<path to the i686 debug apk>` are values only known at execution and are determined by commands shown in the same steps.
- **Type consistency:** `Dir`/`Box`/`pickNext`/`moveFocus` (Task 2) are imported by `dpad.ts` (Task 3); `pushBack`/`handleBack` (Task 3) are used by `Info.svelte`, `Player.svelte` and `App.svelte`; `setSeekMode` (Task 3) is used only by the player; `FileStore` (Task 4) implements the `CredStore` trait defined in Plan 1.

---

### Task 7: Android polish found on the emulators (added after Task 6)

**Goal:** Fix the three problems the emulator run found: TV mode not detected on a TV, the soft keyboard stealing the D-pad on TV, and the status bar overlapping the page on phones.

**Files:**
- Modify: `src-tauri/gen/android/app/src/main/java/me/louven/proxer/MainActivity.kt`
- Modify: `src/lib/tv.ts`, `src/lib/tv.test.ts`, `src/App.svelte`
- Create: `src/lib/tvInput.ts`

**Acceptance Criteria:**
- [ ] `MainActivity.kt` exposes `AndroidTv.isTv()` to the page through a JavaScript interface backed by `UiModeManager.currentModeType == UI_MODE_TYPE_TELEVISION`; `detectTv` takes a 4th argument (native answer, `boolean | null`) and the order is: explicit override, then the native answer when it is a boolean, then the old heuristic (Android UA and no touch points). Tests cover all three tiers.
- [ ] In TV mode text and password inputs are `readonly` while focused by navigation, so the soft keyboard does not open; Enter/Select unlocks the field and opens the keyboard; the lock comes back when focus leaves the field. Not active outside TV mode.
- [ ] On the phone emulator the "Proxer" heading is no longer covered by the status bar (system-bar insets are applied as padding, by CSS `env(safe-area-inset-*)` with `viewport-fit=cover` or by a window-insets listener in `MainActivity.kt`, whichever demonstrably works).
- [ ] Rebuilt debug APKs install and launch on both emulators; screenshots show TV mode active on the TV emulator (large text, margin), no IME after D-pad focus on a field, IME only after Enter, and no status-bar overlap on the phone.
- [ ] `npm test`, `npm run check` (0 errors, 0 warnings), `npm run build`, `cargo test --workspace` (50 tests) all pass; no desktop regression.

**Verify:** `npm test && npm run check && npm run build && cargo test --workspace` → all green, plus the emulator screenshots
