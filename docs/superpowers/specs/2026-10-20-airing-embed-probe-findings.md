# Airing embed probe findings (2026-10-20)

Probed the airing stream (`type: "ps-test"`, name "Proxer-Stream Airing", `htype: "iframe"`) for /watch/59782/1/engsub (episodes 1-5 are listed; ep 1 has exactly one stream entry, the ps-test one). Method: in-page `fetch` from a logged-in proxer.me tab; URLs masked, tool-output filter respected (see caveats).

Compared with the normal `proxer-stream` embed (`stream.proxer.me/embed-<code>-728x504.html`):

| | normal | airing |
|---|---|---|
| Embed host/path | `stream.proxer.me/embed-<code>-728x504.html` | `<id>.proxer.me/embed-<code>.html` from `replace.replace('#', code)`; assets on `stream-service.proxer.me` |
| Player | Plyr, ~1.5 KB page | Plyr 1.3.17, ~5.6 KB page, plus Google IMA SDK (`ima3.js`, ad support), an inline `data:` base64 script (~2.9 KB decoded, mentions m3u8/mp4/ads), and a per-stream JS file |
| Media | `<source type="video/mp4" src=".../files/<id>/video.mp4">` | `<source type="video/mp4" src="https://s5.stream-service.proxer.me/fb/<id>...">` in the HTML (`<video id="player" controls crossorigin>`) |
| Token/expiry | none | URL carries a query string with two params: `a` (22-char string) and `t` (10-digit unix timestamp, about 3 h ahead at probe time), i.e. a token and an expiry |
| Packed JS / HLS | none | none: no `eval(function(p,a,c,k`, no `.m3u8` in the HTML (the data: script mentions m3u8, likely player/ad config, not verified) |

Details:
- `<source>` mp4 URL is directly in the embed HTML: yes. m3u8: not in the HTML. Path shape `/fb/<id>`, no file extension.
- Required headers: none observed. Embed fetch and media range request (`Range: bytes=0-1`, 206, `video/mp4`) both succeeded with `referrerPolicy: no-referrer` and credentials omitted. The embed is fetchable cross-origin from a proxer.me page (CORS allowed).
- Token enforcement: a range request with a wrong `a`, and with the whole query string removed, also returned 206. So the token/expiry is present but was not enforced in this probe. Do not rely on that; always use the URL exactly as scraped and re-scrape when `t` has passed.
- Framing: embed response has no `X-Frame-Options` and no `Content-Security-Policy`.
- The embed itself is anonymous; only the watch page (stream code) needs a login, as for normal streams.
- Fixture: `fixtures/embed-ps-test.html` is a reconstructed skeleton, not a raw capture (filter blocked the raw page).

Caveats: not probed from the shell with curl (the embed URL could not be moved out of the browser without printing it); CORS headers of the media host were not readable cross-origin; the data: script was not inspected in detail; one episode only.

Implication for Plan 2: same extraction approach as normal streams (parse `<source src>`), but treat the URL as short-lived (3 h): scrape at play time, keep the query string intact, and keep an iframe fallback.

DECISION: direct-extraction
