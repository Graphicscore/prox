# Plan 2 manual verification

Run `env WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri dev` (Wayland workaround).

1. App loads, auto-login works, search `naruto` works (CSP did not break IPC). If the window is blank or invoke fails, revert `csp`/`devCsp` to `null` and report the console error.
2. Normal stream: open Naruto (`/info/45`), episode 1, a language, click the [Normal] stream. Video plays; seek with the bar and with the arrow keys; Space pauses; Escape closes and returns to the same episode list.
3. Airing stream: open Tensei shitara Ken deshita 2 (`/info/59782`), episode 1, click the [Airing] stream. Video plays.
4. "Use embedded player instead" shows the embed in an iframe and it plays (or note what is blocked).
5. An episode with no streams shows "No streams (yet)".
6. Leave an airing stream paused for more than 3 hours (optional), then resume: it should re-resolve once or show "Playback failed" with the fallback button.
7. Results: note anything that failed, with the message shown.
