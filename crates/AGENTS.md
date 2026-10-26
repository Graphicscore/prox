# crates/ - Proxer scraping core

## OVERVIEW
Two pure-Rust crates with no Tauri dependency: `proxer-parse` (HTML/JSON-in-HTML -> structs) and `proxer-client` (logged-in HTTP session). Earned its own file: separate domain, fixture-driven tests, most security rules live here.

## STRUCTURE
```
crates/
├── proxer-parse/src/   # one module per page type; lib.rs re-exports everything
│   ├── search.rs       # /search and list pages (same tr.entry table)
│   ├── info.rs, episodes.rs, streams.rs, calendar.rs
│   ├── mylist.rs       # /ucp list (JSON var ucp_list_items), page token, write replies, plan_watched
│   ├── embed.rs        # hoster embed page -> MediaSource (url, mime, expires_at)
│   └── session.rs      # login form action, is_logged_in marker
└── proxer-client/src/lib.rs  # Client: get/post/get_embed/login/logout, with_tls
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Proxer changed its HTML | `proxer-parse/src/<page>.rs` + `fixtures/<page>.html` | Refresh the fixture, then fix the selector |
| Logged-in detection | `session.rs` `is_logged_in` | Marker: `id="mod_logout_submit"` |
| Re-login / rate limit | `proxer-client` `Client::get` | Single-flight re-login, Semaphore(2) |
| Which hosts may be fetched | `is_allowed_embed_url` | https, `*.proxer.me` only, no port/userinfo |
| Android TLS | `with_tls` | Static webpki roots on Android; also used by the updater |

## CONVENTIONS
- Parsers are total: garbage input gives an empty result or a typed error, never a panic. Add a garbage/empty-input test with each new parser (search, calendar, mylist, streams, episodes have one).
- Tests live in `<crate>/tests/*.rs` and read fixtures via `../../fixtures/<name>.html` (path relative to the crate dir; cargo runs tests there).
- Client tests use `wiremock`; no test may hit proxer.me.
- Stream kinds: `proxer-stream` = Normal, `ps-test` = Airing (media URL has `t` expiry ~3 h); everything else is `Other`/Unsupported. Both playable kinds are required.
- Write endpoints need the page token (`token(html)`), obtained by GET of the page first.

## ANTI-PATTERNS
- NEVER let a request URL reach logs: embed URLs and token-bearing POST URLs use `e.without_url()`.
- NEVER follow redirects or send cookies for embed fetches (`embed_http` is anonymous by design).
- Do not re-POST a password after `BadCredentials`: credentials are cleared to avoid account lockout.
- Do not raise concurrency above 2 requests (politeness to the site).
- Do not treat 5xx/429 as "logged out": `check_status` runs before the login check.
- Do not use the official Proxer API or keys taken from the official app; it is HTML scraping with the user's own account.
- 2FA (`secretkey`) accounts are unsupported; do not pretend otherwise.
- Fixtures are scrubbed pages: never commit a fixture with a real user id, username or token.
