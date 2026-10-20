# Embed probe findings (2026-10-20)

Probed `proxer-stream` embed for /watch/45/1/engsub (embed `https://stream.proxer.me/embed-<code>-728x504.html`).

- Embed page: plain HTML, ~1.5 KB, Plyr player. Contains `<video id="plyr"><source type="video/mp4" src="https://s90.ps.proxer.me/files/<id>/video.mp4">`. No packed JS, no HLS.
- Media URL kind: **MP4**, directly in the HTML `<source src>`. Host varies (`sNN.ps.proxer.me`).
- Token/expiry: none. URL has no query string or signature.
- Required headers: none. Embed page and MP4 both return 200/206 with no Referer, no cookies, and a plain browser User-Agent. Range requests work (`Accept-Ranges: bytes`, 206).
- Framing: embed response has no `X-Frame-Options` and no `Content-Security-Policy`; framing is not blocked.
- CORS: MP4 sends `Access-Control-Allow-Origin: https://stream.proxer.me` only, so a non-proxer webview origin cannot fetch it via XHR; `<video src>` without `crossorigin` works, otherwise use the Rust media proxy.
- Embed fetch works anonymously; only the watch page (to get the stream code) needs a login.
- Other hoster types: not observed (only one stream entry on this episode).

DECISION: direct-extraction
