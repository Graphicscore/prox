# Proxer Tauri Frontend: Design

Date: 2026-10-20
Status: approved in chat, pending written-spec review

## Goal

An unofficial third-party frontend for https://proxer.me/ as a Tauri v2 app for desktop (Windows, macOS, Linux), Android (phone and tablet) and Android TV. One codebase, D-pad usable on TV.

## Constraints and findings (spike, 2026-10-20)

- The official API v1 is deprecated and no new keys are issued. Not usable. Keys extracted from the official app must not be used (key sharing is prohibited).
- Plain server-side fetch of proxer.me works: no Cloudflare challenge. Search pages are anonymous-readable.
- No CORS headers (`OPTIONS` returns 405) and `SameSite=strict` cookies: a browser-only site cannot call proxer.me. Tauri's Rust HTTP layer avoids this.
- `/info/<id>` and `/watch/<id>/<ep>/<lang>` require a logged-in session. Anonymous requests return a 404 page or an empty stub.
- Info page: server-rendered HTML, details table (titles, genres, season, status, adaptation). Parseable with selectors.
- Watch page: inline `var streams = [...]` JSON. Entry keys: `id, code, type, htype, name, replace, img, parts, ssl, text, legal, public, points, uploader, username, tid, tname`. Only `type: "proxer-stream"` observed; its embed URL template is `https://stream.proxer.me/embed-#-728x504.html`.
- **Not verified:** embed internals (direct media URL extractable? token or expiry protection? framing allowed?), episode list markup, other hoster types, rate limiting behaviour, TV WebView codec support.

## Architecture

```
UI (Svelte SPA in webview) --invoke--> Rust core --> proxer.me / stream.proxer.me
```

Units, each with one purpose:

1. **`proxer-client` (Rust crate).** HTTP, persisted cookie jar, login. `reqwest` with a browser-like User-Agent. Detects a lost session (404 or login page where content is expected) and returns `NotLoggedIn`. No UI knowledge.
2. **`proxer-parse` (Rust crate).** Pure functions, HTML to typed structs: search results, info, episode list, `streams` JSON. No network. Tested against saved HTML fixtures.
3. **`stream-resolver` + media proxy (Rust).** Turns a stream entry into a playable URL. A `Resolver` trait with one implementation per hoster (`resolve(stream) -> MediaSource`). A local `127.0.0.1:<random port>` proxy forwards range requests with the needed `Referer`/cookies and rewrites HLS playlists so segments also pass through it.
4. **Tauri commands.** Thin typed API: `login`, `search`, `get_info`, `get_episodes`, `get_streams`, `resolve_stream`.
5. **UI (Svelte + TypeScript, Vite).** Screens: Login, Search, Info (details and episodes), Player, Settings (logout, clear saved credentials).

## Login and credentials

- Login form: username, password, "remember me" (default on), show-password toggle.
- Username and password are stored in the OS secure store: the `keyring` crate on desktop (Credential Manager, Keychain, Secret Service), Android Keystore via a Tauri plugin on Android and Android TV (`keyring` does not support Android; needs a small plugin or an existing one).
- On `NotLoggedIn`, `proxer-client` logs in again with stored credentials and retries the request once. If that fails, show the login form with the error. No retry loops on `BadCredentials`.
- Settings "logout" clears the session cookie and the stored credentials.

## Data flow

1. App start: load credentials, log in, show Search.
2. `search(q)` -> `proxer-client` GET -> `proxer-parse` -> `Vec<SearchHit>` to UI.
3. Open hit: `get_info(id)` returns details and episode list.
4. Pick episode: `get_streams(id, ep, lang)` parses the inline `streams` JSON.
5. `resolve_stream(stream)` returns a loopback media URL; UI plays it in `<video>`.

## Errors (typed results, not strings)

| Error | Handling |
|---|---|
| `NotLoggedIn` | Auto re-login, retry once, then login form. |
| `BadCredentials` | Login form with message; no retry. |
| `ParseFailed(page)` | "Proxer changed something" message with a report action; never crash. |
| `StreamUnsupported(hoster)` | Disable that stream, offer the others. |
| `Network` / `Blocked` | Retry button; hint when Cloudflare or an IP block is suspected. |

Politeness: at most 2 concurrent requests, short delay between pages, fetch only on user navigation, in-memory cache of search/info results for a few minutes.

## Stream resolver and playback

Strategy chain, tried in order:

1. **Direct extraction.** Rust fetches the embed with `Referer: https://proxer.me/`, extracts the media URL (HLS or MP4) from HTML or inline JS.
2. **Iframe fallback.** If extraction fails but the embed allows framing, play it in an `<iframe>`. Degraded D-pad control on TV.
3. **Open externally.** Open the embed in the system browser.

Playback: webview plays the proxied URL. HLS via `hls.js` where the WebView lacks native HLS, MP4 natively. Plain `<video>` with custom controls: language and stream picker, subtitles if provided. Remote keys: Enter play/pause, Left/Right seek 10 s, Back exits the player.

v1 supports `proxer-stream` plus the iframe fallback. Other hoster types show as unsupported until observed in practice.

Decision point after implementation task 1 (embed probe): if the embed is directly extractable, build the full chain. If it is protected, v1 ships with the iframe fallback on desktop and mobile, and TV uses open-externally. Do not reverse-engineer a protection.

## UI

Three input modes, one layout:

| Mode | Detected by | Behaviour |
|---|---|---|
| Touch | Android phone or tablet, coarse pointer | Bottom tab bar, 44px+ targets. |
| Mouse and keyboard | Desktop | Sidebar nav, hover states, `/` focuses search. |
| TV (D-pad) | Android TV leanback feature, or `?tv` override | ~24px base font, everything focusable, strong focus ring. |

- Spatial navigation through one small focus manager (library such as `norigin-spatial-navigation`, or about 100 lines of our own). Back goes up one screen; on the home screen it exits after a confirm.
- Login on TV uses the system on-screen keyboard, large input targets, show-password toggle.
- CSS grid with `clamp()` sizing, `auto-fill` poster grids (2 columns on phone to 6+ on TV), 5% overscan safe margin. Dark theme only in v1.

## Packaging

- Desktop: Tauri bundles (`.msi`, `.dmg`, `.AppImage`).
- Android and Android TV: one APK. Manifest declares `LEANBACK_LAUNCHER`, a TV banner, `touchscreen required="false"`. Cleartext loopback allowed for the media proxy. Sideload for TV testing, no Play Store submission planned.

## Testing

- `proxer-parse`: fixture tests from real saved pages (search, info, watch), collected from a logged-in session.
- `stream-resolver`: unit test of URL extraction from a saved embed page.
- Manual smoke checklist per platform (TV cannot be automated), including playback on a real Android TV box.

## Out of scope (v1)

Manga reader, messenger, forum, notifications, user list sync, downloads, casting, iOS and Apple TV, light theme. Each can become its own spec.

## Risks

- Embed may be protected (tokens, expiring URLs, obfuscation).
- Old TV WebViews: HLS and codec support varies.
- Android Keystore access needs a plugin.
- IP blocking or rate limiting under heavy use (untested).
- Scraping may violate Proxer's terms; acceptable for personal use, higher risk if distributed publicly.
- Markup changes break parsers; mitigated by fixtures and the `ParseFailed` path.

## Implementation order (for the plan)

1. Embed probe (decides the resolver path).
2. `proxer-client` and login.
3. `proxer-parse` with fixtures.
4. Tauri commands and desktop UI (Search, Info).
5. Stream resolver, media proxy and player.
6. Android build, then TV focus navigation and manifest.
