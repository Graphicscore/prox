# PROJECT KNOWLEDGE BASE

**Generated:** 2026-10-26
**Commit:** 6853f04
**Branch:** master

## OVERVIEW
Unofficial Proxer.Me anime streaming app for Android TV, Android phones and Linux desktop. Tauri v2 (Rust core scraping proxer.me HTML) + Svelte 5 UI in the system WebView; TV remote (D-pad) first.

## STRUCTURE
```
proxer/
├── crates/
│   ├── proxer-parse/   # HTML -> structs, one module per Proxer page (see crates/AGENTS.md)
│   └── proxer-client/  # cookie session, re-login, max 2 concurrent requests
├── src-tauri/          # Tauri commands, creds, updater; gen/android = tracked Gradle project (see src-tauri/AGENTS.md)
├── src/                # Svelte 5 UI (see src/AGENTS.md)
├── fixtures/           # saved, scrubbed proxer.me pages for parser/client tests (~700 KB HTML)
├── docs/superpowers/   # design specs + implementation plans (*.md + *.tasks.json); history, not instructions
├── docs/android-setup.md  # SDK/NDK/JDK + release signing
├── scripts/android-env.sh # source before any `tauri android` command
└── .forgejo/workflows/build.yml  # the only CI (Forgejo Actions)
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| New feature end to end | parser -> client/command -> `src/lib/api.ts` -> screen | Plus `demo.ts` and a fixture test |
| Proxer page changed | `crates/proxer-parse/src/*.rs`, `fixtures/` | |
| New IPC command | `src-tauri/src/commands.rs`, `lib.rs` handler list | |
| UI screen / TV navigation | `src/lib/*.svelte`, `spatialNav.ts`, `dpad.ts`, `back.ts` | |
| Release / build numbers | `.forgejo/workflows/build.yml`, `src-tauri/src/update.rs` | |
| Why something was built a way | `docs/superpowers/specs/`, `plans/` | |

## CODE MAP
| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `Client` | struct | crates/proxer-client/src/lib.rs | All proxer.me HTTP; `get` re-logs in once |
| `is_allowed_embed_url` | fn | crates/proxer-client/src/lib.rs | Host allowlist (mirrored by `safeEmbedUrl` in TS) |
| `parse_*` | fns | crates/proxer-parse/src/ | Page parsers re-exported from lib.rs |
| `plan_watched` | fn | crates/proxer-parse/src/mylist.rs | Steps to mark an episode watched |
| `commands::App` | struct | src-tauri/src/commands.rs | Managed state: `Client` + `CredStore` |
| `err` | fn | src-tauri/src/commands.rs | Rust error -> UI string code |
| `api` / `Api` | const/type | src/lib/api.ts | Typed `invoke` wrappers; demo swap in dev |
| `describeError` | fn | src/lib/api.ts | UI text for error codes |
| `pushBack` / `handleBack` | fns | src/lib/back.ts | Back stack (Android Back, Esc, mouse back) |
| `pickNext` / `moveFocus` | fns | src/lib/spatialNav.ts | D-pad spatial navigation |

## CONVENTIONS
- IPC contract = command name + serde field names (snake_case on both sides) + string error codes. Change Rust and TS together.
- Comments explain WHY in one line; code is dense (short names, one-line matches). Match the surrounding density.
- Rust edition 2021; workspace release profile only in root `Cargo.toml` (CI overrides LTO/codegen via `CARGO_PROFILE_RELEASE_*`).
- App version is `0.1.<N>` stamped by CI from the `build-<N>` tag (`--config {"version":...}`); local and untagged builds are `0.1.0` = build 0.
- UI copy is English; Proxer data (list states, messages) stays German (`Am Schauen`, `Wird noch geschaut`, ...).

## ANTI-PATTERNS (THIS PROJECT)
- NEVER use the official Proxer API or keys extracted from the official app.
- NEVER put real account data (user id, username, tokens, password) in fixtures, tests, logs or commits; history was scrubbed once.
- Never push the local tag `pre-scrub-backup`.
- Agents never type the user's Proxer password; the user logs in in the app/emulator.
- No server, no telemetry: the app talks to proxer.me (and GitHub releases for updates) only.

## COMMANDS
```bash
npm ci
npm test                      # node --test src/lib/*.test.ts src/lib/ui/*.test.ts
npm run check                 # svelte-check (tsgo) + tsc for node config
cargo test --workspace        # CI: TMPDIR=/dev/shm (creds tests check 0600)
npm run dev                   # UI only, http://localhost:1420/?demo&screen=home
npm run tauri dev             # desktop app
source scripts/android-env.sh && npm run tauri android build -- --apk --split-per-abi --target aarch64
```

## NOTES
- CI runs on Forgejo Actions (git.asteria.dog, `origin`) for master, PRs and tags. Only a pushed tag `build-<N>` publishes release `build-<N>` (app version `0.1.<N>`); master/PR builds are version `0.1.0` and never publish. The in-app updater reads Forgejo releases; they are mirrored to GitHub `Graphicscore/prox` for older installs and README links.
- Desktop on KDE Wayland + NVIDIA: run with `env WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri dev`; video needs `gst-plugins-good`. WebKit native video fullscreen renders black, so the player uses its own fullscreen (Tauri `setFullscreen`).
- `tauri dev` does not enforce the CSP; only a built app does.
- Android: use `--split-per-abi` APKs (the universal one is too big for the phone AVD). Debug and release builds have different signatures: uninstall before switching.
- Emulator QA: drive with adb keyevents and read state through WebView DevTools (`adb forward tcp:9222 localabstract:webview_devtools_remote_<pid>`); emulator screenshots can show stale video frames.
