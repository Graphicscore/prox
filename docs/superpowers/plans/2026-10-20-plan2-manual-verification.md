# Plan 2 manual verification

Run `env WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri dev` (Wayland workaround).

Note: on desktop, `tauri dev` loads `devUrl` (http://localhost:1420) directly and applies neither `csp` nor `devCsp`; `devCsp` is kept for later mobile dev. The real CSP is only active in a built app, so step 7 is the one that tests it.

1. App loads, auto-login works, search `naruto` works (dev mode: no CSP applied, so this checks functionality only).
2. Normal stream: open Naruto (`/info/45`), episode 1, a language, click the [Normal] stream. Video plays; seek with the bar and with the arrow keys; Space pauses; Escape closes and returns to the same episode list.
3. Airing stream: open Tensei shitara Ken deshita 2 (`/info/59782`), episode 1, click the [Airing] stream. Video plays.
4. "Use embedded player instead" shows the embed in an iframe and it plays (or note what is blocked).
5. An episode with no streams shows "No streams (yet)".
6. Leave an airing stream paused for more than 3 hours (optional), then resume: it should re-resolve once or show "Playback failed" with the fallback button.
7. Production build (the only place the CSP is enforced). Build with `npm run tauri build -- --debug --no-bundle`, then run `env WEBKIT_DISABLE_DMABUF_RENDERER=1 /home/asteria/Projects/proxer/target/debug/proxer-app` (i.e. `target/debug/proxer-app`). Repeat login, search `naruto`, the normal stream (step 2), the airing stream (step 3) and the embedded iframe (step 4) there. If the CSP is wrong you will see:
   - IPC blocked: blank window stuck on boot, console "Refused to connect to ipc://localhost".
   - Scripts blocked: blank window.
   - Media blocked: "Playback failed" after one retry.
   - Frame blocked: empty iframe.

   To revert, set `csp` to `null` in `src-tauri/tauri.conf.json`, rebuild, and report the console error.

8. Results: note anything that failed, with the message shown.
