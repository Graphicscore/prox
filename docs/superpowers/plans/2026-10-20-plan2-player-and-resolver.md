# Player and Stream Resolver Implementation Plan (Plan 2 of 3)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers-extended-cc:subagent-driven-development (recommended) or superpowers-extended-cc:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Pick a stream on the info screen and watch it in an in-app player, for both normal (`proxer-stream`) and airing (`ps-test`) streams, with an embedded-iframe fallback.

**Architecture:** `proxer-parse` extracts the media URL (`<video><source src>`) and its expiry from the embed page. `proxer-client` fetches embed pages anonymously from an allowlisted host (`https://*.proxer.me`, redirects disabled). A Tauri command `resolve_stream` ties them together. A Svelte `Player` plays the URL in a plain `<video>`, re-resolves once on error (airing URLs expire after about 3 h), and can switch to an `<iframe>` of the embed.

**Tech Stack:** Rust (scraper, reqwest, serde, thiserror), Tauri v2 commands, Svelte 5 + TypeScript.

**Spec:** `docs/superpowers/specs/2026-10-20-proxer-tauri-frontend-design.md`. Probe results: `docs/superpowers/specs/2026-10-20-embed-probe-findings.md` (normal) and `docs/superpowers/specs/2026-10-20-airing-embed-probe-findings.md` (airing).

**Out of this plan:** Android and Android TV builds, D-pad navigation, the Enter key and Back key remote handling, open-in-system-browser fallback (all Plan 3); other hoster types (shown as Unsupported); subtitles; downloads.

## Global Constraints

- Both probes concluded `DECISION: direct-extraction`: the embed HTML contains `<video><source type="video/mp4" src="https://...">`. Normal URLs have no query string and need no headers. Airing URLs (`.../fb/<id>?a=<22 chars>&t=<unix seconds>`) expire about 3 hours after scraping: scrape at play time, keep the query string exactly as scraped, and re-scrape once if playback fails.
- Deliberate deviation from the spec: **no local media proxy**. The probes showed `<video src>` without `crossorigin` plays from both hosts with no Referer or cookies. `ponytail:` add the 127.0.0.1 proxy only if playback fails in the Android or TV WebView (Plan 3).
- Embeds are fetched anonymously (no login cookies) and only from `https://` hosts ending in `.proxer.me`. Redirects are not followed (a redirect could leave the allowlist). Everything is validated in `proxer-client`, not in the UI.
- Never log or print media URLs (airing URLs carry a token) or credentials.
- Errors stay typed: the command returns the codes `NotLoggedIn`, `BadCredentials`, `Parse:<what>`, `Http:<n>`, `Network`, plus the new `Unsupported`.
- Max 2 concurrent requests to proxer hosts (the existing semaphore also gates embed fetches).
- `fixtures/embed-ps-test.html` is a reconstructed skeleton, not a raw capture; expiry parsing is tested with inline HTML instead.

**User decisions (already made):** Tauri app for desktop, Android and Android TV; both stream types (normal and airing) must be supported; saved username and password via the OS keychain.

---

### Task 1: proxer-parse: embed page parser and embed URL

**Goal:** `parse_embed(html) -> Result<MediaSource, EmbedError>` and `Stream::embed_url()`.

**Files:**
- Create: `crates/proxer-parse/src/embed.rs`
- Modify: `crates/proxer-parse/src/lib.rs`, `crates/proxer-parse/src/streams.rs`
- Test: `crates/proxer-parse/tests/embed.rs`

**Acceptance Criteria:**
- [ ] `parse_embed(fixtures/embed-proxer-stream.html)` gives an `https://` URL ending in `/video.mp4`, mime `video/mp4`, `expires_at == None`.
- [ ] `parse_embed(fixtures/embed-ps-test.html)` gives an `https://` URL containing `/fb/`.
- [ ] A `<source src="https://...?a=XyZ&amp;t=1790000000">` gives the URL with `&` decoded and `expires_at == Some(1790000000)`.
- [ ] Missing `<source>`, or a non-`https` src, gives `Err(EmbedError::NoSource)`.
- [ ] `Stream::embed_url()` replaces `#` in `replace` with `code`.

**Verify:** `cargo test -p proxer-parse --test embed` → all tests pass

**Steps:**

- [ ] **Step 1: Write the failing tests `crates/proxer-parse/tests/embed.rs`.**

```rust
use proxer_parse::{parse_embed, EmbedError, Stream};

#[test]
fn normal_embed_fixture() {
    let html = std::fs::read_to_string("../../fixtures/embed-proxer-stream.html").unwrap();
    let m = parse_embed(&html).unwrap();
    assert!(m.url.starts_with("https://") && m.url.ends_with("/video.mp4"));
    assert_eq!(m.mime, "video/mp4");
    assert_eq!(m.expires_at, None);
}

#[test]
fn airing_embed_fixture_skeleton() {
    let html = std::fs::read_to_string("../../fixtures/embed-ps-test.html").unwrap();
    let m = parse_embed(&html).unwrap();
    assert!(m.url.starts_with("https://") && m.url.contains("/fb/"));
}

#[test]
fn airing_expiry_is_read_from_t() {
    let html = r#"<video><source type="video/mp4" src="https://s5.stream-service.proxer.me/fb/abc?a=XyZ&amp;t=1790000000"></video>"#;
    let m = parse_embed(html).unwrap();
    assert_eq!(m.url, "https://s5.stream-service.proxer.me/fb/abc?a=XyZ&t=1790000000");
    assert_eq!(m.expires_at, Some(1790000000));
}

#[test]
fn missing_or_insecure_source_is_an_error() {
    assert_eq!(parse_embed("<html></html>"), Err(EmbedError::NoSource));
    assert_eq!(parse_embed(r#"<video><source src="http://x.test/a.mp4"></video>"#), Err(EmbedError::NoSource));
}

#[test]
fn embed_url_substitutes_the_code() {
    let s: Stream = serde_json::from_str(
        r#"{"code":"abc","type":"proxer-stream","name":"n","replace":"https://stream.proxer.me/embed-#-728x504.html"}"#,
    )
    .unwrap();
    assert_eq!(s.embed_url(), "https://stream.proxer.me/embed-abc-728x504.html");
}
```

- [ ] **Step 2: Run to verify failure.**

Run: `cargo test -p proxer-parse --test embed`
Expected: FAIL to compile (`parse_embed`, `EmbedError` not found)

- [ ] **Step 3: Implement `crates/proxer-parse/src/embed.rs`.**

```rust
use scraper::{Html, Selector};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MediaSource {
    pub url: String,
    pub mime: String,
    /// Unix seconds from the `t` query parameter (airing streams); `None` when the URL has no expiry.
    pub expires_at: Option<u64>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EmbedError {
    #[error("no https <source src> in the embed page")]
    NoSource,
}

pub fn parse_embed(html: &str) -> Result<MediaSource, EmbedError> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("video source[src]").unwrap();
    let el = doc.select(&sel).next().ok_or(EmbedError::NoSource)?;
    let url = el.value().attr("src").unwrap_or_default().to_string();
    if !url.starts_with("https://") {
        return Err(EmbedError::NoSource);
    }
    let mime = el.value().attr("type").unwrap_or("video/mp4").to_string();
    Ok(MediaSource { expires_at: expiry(&url), url, mime })
}

fn expiry(url: &str) -> Option<u64> {
    let query = url.split_once('?')?.1;
    query.split('&').find_map(|kv| kv.strip_prefix("t=")).and_then(|v| v.parse().ok())
}
```

- [ ] **Step 4: Add `embed_url` to `Stream` in `crates/proxer-parse/src/streams.rs`** (next to `stream_kind`):

```rust
    /// The embed page for this stream: `replace` has a `#` placeholder for `code`.
    pub fn embed_url(&self) -> String {
        self.replace.replace('#', &self.code)
    }
```

- [ ] **Step 5: Export from `lib.rs`.**

```rust
mod embed;
pub use embed::{parse_embed, EmbedError, MediaSource};
```

- [ ] **Step 6: Run to verify pass, then commit.**

Run: `cargo test -p proxer-parse`
Expected: all pass

```bash
git add crates/proxer-parse && git commit -m "feat(parse): embed page parser and Stream::embed_url"
```

---

### Task 2: proxer-client: allowlisted anonymous embed fetch

**Goal:** `is_allowed_embed_url` and `Client::get_embed(url)`: anonymous, https-only, `*.proxer.me` only, no redirects.

**Files:**
- Modify: `crates/proxer-client/src/lib.rs`
- Test: `crates/proxer-client/tests/embed.rs`

**Acceptance Criteria:**
- [ ] `is_allowed_embed_url` accepts `https://stream.proxer.me/embed-a-1.html` and `https://abc.proxer.me/embed-a.html`.
- [ ] It rejects `http://stream.proxer.me/x`, `https://evilproxer.me/x`, `https://proxer.me.evil.test/x`, `https://stream.proxer.me@evil.test/x`, `https://proxer.me/x` (no subdomain), and `not a url`.
- [ ] `Client::get_embed("https://evil.test/x")` returns `Err(Error::Parse("embed host not allowed"))` without sending a request.
- [ ] The embed HTTP client sends no cookies and does not follow redirects; 5xx/429 map to `Error::Http`.
- [ ] All 12 existing client tests stay green.

**Verify:** `cargo test -p proxer-client` → all tests pass

**Steps:**

- [ ] **Step 1: Write the failing tests `crates/proxer-client/tests/embed.rs`.**

```rust
use proxer_client::{is_allowed_embed_url, Client, Error};

#[test]
fn allowlist() {
    assert!(is_allowed_embed_url("https://stream.proxer.me/embed-a-1.html"));
    assert!(is_allowed_embed_url("https://abc.proxer.me/embed-a.html"));
    assert!(!is_allowed_embed_url("http://stream.proxer.me/x"));
    assert!(!is_allowed_embed_url("https://evilproxer.me/x"));
    assert!(!is_allowed_embed_url("https://proxer.me.evil.test/x"));
    assert!(!is_allowed_embed_url("https://stream.proxer.me@evil.test/x"));
    assert!(!is_allowed_embed_url("https://proxer.me/x"));
    assert!(!is_allowed_embed_url("not a url"));
}

#[tokio::test]
async fn foreign_host_is_refused_without_a_request() {
    let c = Client::new("https://proxer.me");
    assert!(matches!(c.get_embed("https://evil.test/x").await, Err(Error::Parse("embed host not allowed"))));
}
```

- [ ] **Step 2: Run to verify failure.**

Run: `cargo test -p proxer-client --test embed`
Expected: FAIL to compile (`is_allowed_embed_url` not found)

- [ ] **Step 3: Implement in `crates/proxer-client/src/lib.rs`.** Add the field, builder, method and function:

```rust
// in struct Client:
    embed_http: reqwest::Client, // anonymous: no cookies, no redirects
```

```rust
// in Client::new, inside the struct literal:
            embed_http: reqwest::Client::builder()
                .user_agent(UA)
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(20))
                .build()
                .expect("embed http client"),
```

```rust
// in impl Client:
    /// GET an embed page anonymously. Only `https://*.proxer.me` is allowed and redirects are not followed.
    pub async fn get_embed(&self, url: &str) -> Result<String, Error> {
        if !is_allowed_embed_url(url) {
            return Err(Error::Parse("embed host not allowed"));
        }
        let _p = self.gate.acquire().await.expect("semaphore");
        let res = self.embed_http.get(url).send().await?;
        check_status(res.status())?;
        Ok(res.text().await?)
    }
```

```rust
// free function at the end of the file:
/// `https://` and a host that ends in `.proxer.me` (the bare domain and look-alikes like `evilproxer.me` are refused).
pub fn is_allowed_embed_url(url: &str) -> bool {
    let Ok(u) = reqwest::Url::parse(url) else { return false };
    u.scheme() == "https" && u.host_str().is_some_and(|h| h.ends_with(".proxer.me"))
}
```

- [ ] **Step 4: Run to verify pass, then commit.**

Run: `cargo test -p proxer-client`
Expected: all pass (existing 12 + 2 new)

```bash
git add crates/proxer-client && git commit -m "feat(client): allowlisted anonymous embed fetch"
```

---

### Task 3: Tauri command `resolve_stream`

**Goal:** `resolve_stream(stream)` returns a `MediaSource` for normal and airing streams and `Unsupported` for other types.

**Files:**
- Modify: `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`

**Acceptance Criteria:**
- [ ] `resolve_stream` is registered in `generate_handler!` and takes the `Stream` object the UI received from `get_streams`.
- [ ] A stream of kind `Other` returns `Err("Unsupported")` without any network request; Normal and Airing build their embed URL from `Stream::embed_url()`.
- [ ] An embed page without a usable `<source>` returns `Err("Parse:embed")`; other errors use the existing `err()` codes.
- [ ] A unit test covers the three kinds through a pure helper.

**Verify:** `cargo test -p proxer-app && cargo build -p proxer-app` → exit 0

**Steps:**

- [ ] **Step 1: Write the failing test** in the `tests` module of `src-tauri/src/commands.rs`.

```rust
    fn stream(kind: &str) -> Stream {
        serde_json::from_str(&format!(
            r#"{{"code":"abc","type":"{kind}","name":"n","replace":"https://stream.proxer.me/embed-#-728x504.html"}}"#
        ))
        .unwrap()
    }

    #[test]
    fn embed_target_by_kind() {
        let want = "https://stream.proxer.me/embed-abc-728x504.html";
        assert_eq!(embed_target(&stream("proxer-stream")).as_deref(), Ok(want));
        assert_eq!(embed_target(&stream("ps-test")).as_deref(), Ok(want));
        assert_eq!(embed_target(&stream("other-host")), Err("Unsupported".to_string()));
    }
```

(`serde_json` is needed by this test: add `serde_json = "1"` under `[dev-dependencies]` in `src-tauri/Cargo.toml` if it is not there.)

- [ ] **Step 2: Run to verify failure.**

Run: `cargo test -p proxer-app embed_target`
Expected: FAIL to compile (`embed_target` not found)

- [ ] **Step 3: Implement in `src-tauri/src/commands.rs`.** Extend the `proxer_parse` import with `parse_embed, MediaSource, StreamKind`, then add:

```rust
/// The embed page to fetch for a stream, or "Unsupported" for hosters we cannot play.
fn embed_target(stream: &Stream) -> Result<String, String> {
    match stream.stream_kind() {
        StreamKind::Normal | StreamKind::Airing => Ok(stream.embed_url()),
        StreamKind::Other => Err("Unsupported".into()),
    }
}

#[tauri::command]
pub async fn resolve_stream(app: State<'_, App>, stream: Stream) -> Result<MediaSource, String> {
    let url = embed_target(&stream)?;
    let html = app.client.get_embed(&url).await.map_err(err)?;
    parse_embed(&html).map_err(|_| "Parse:embed".to_string())
}
```

- [ ] **Step 4: Register in `src-tauri/src/lib.rs`:** add `commands::resolve_stream` to the `generate_handler!` list.

- [ ] **Step 5: Run to verify pass, then commit.**

Run: `cargo test -p proxer-app && cargo build -p proxer-app`
Expected: exit 0

```bash
git add src-tauri Cargo.lock && git commit -m "feat(app): resolve_stream command"
```

---

### Task 4: Svelte Player and stream selection

**Goal:** Clicking a supported stream on the info screen opens an in-app player that plays it, retries once, and offers the embedded player as a fallback.

**Files:**
- Create: `src/lib/Player.svelte`
- Modify: `src/lib/api.ts`, `src/lib/Info.svelte`

**Acceptance Criteria:**
- [ ] `api.resolve(stream)` calls `resolve_stream` and returns `{ url, mime, expires_at }`.
- [ ] On the info screen each stream is a button labelled with its name and `[Normal]`/`[Airing]`; `Unsupported` streams are disabled buttons.
- [ ] The Player resolves the stream on open, plays it in a `<video controls autoplay playsinline>`, and shows `describeError` messages in a `role="alert"`; `Unsupported` shows "This stream type is not supported yet."; `NotLoggedIn` calls `onauth`.
- [ ] A video error triggers exactly one re-resolve; a second error shows "Playback failed" and keeps the fallback button.
- [ ] "Use embedded player instead" swaps the video for an `<iframe title="Embedded player" src={embed url}>`.
- [ ] Keys: Escape closes the player, ArrowLeft/ArrowRight seek 10 s, Space toggles play/pause (not while focus is on a button or input).
- [ ] Closing the player returns to the info screen with its episode page, selected episode and streams still shown.
- [ ] `npm run check` reports 0 errors and 0 warnings; `npm run build` succeeds.

**Verify:** `npm run build && npm run check` → exit 0; then the human walk-through in Task 5

**Steps:**

- [ ] **Step 1: `src/lib/api.ts`.** Add the type and the call:

```ts
export type MediaSource = { url: string; mime: string; expires_at: number | null };
```

```ts
  resolve: (stream: Stream) => invoke<MediaSource>("resolve_stream", { stream }),
```

In `describeError`, add before the final return: `if (c === "Unsupported") return "This stream type is not supported yet.";`

- [ ] **Step 2: Create `src/lib/Player.svelte`.**

```svelte
<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { api, describeError, streamLabel, type MediaSource, type Stream } from "./api";
  let { stream, onback, onauth }: { stream: Stream; onback: () => void; onauth: () => void } = $props();
  let src = $state<MediaSource | null>(null), error = $state(""), frame = $state(false);
  let video = $state<HTMLVideoElement | null>(null);
  let retried = false, alive = true;
  const embedUrl = () => stream.replace.replace("#", stream.code);

  async function resolve() {
    error = "";
    try {
      const m = await api.resolve(stream);
      if (alive) src = m;
    } catch (x) {
      if (!alive) return;
      if (x === "NotLoggedIn") onauth(); else error = describeError(x);
    }
  }
  function onVideoError() {
    if (!retried) { retried = true; src = null; resolve(); } // airing URLs expire: scrape again once
    else error = "Playback failed";
  }
  function key(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement | null)?.tagName;
    if (e.key === "Escape") { onback(); return; }
    if (!video || tag === "INPUT") return;
    if (e.key === "ArrowLeft") { video.currentTime = Math.max(0, video.currentTime - 10); e.preventDefault(); }
    else if (e.key === "ArrowRight") { video.currentTime += 10; e.preventDefault(); }
    else if (e.key === " " && tag !== "BUTTON") { if (video.paused) video.play(); else video.pause(); e.preventDefault(); }
  }
  onMount(resolve);
  onDestroy(() => { alive = false; });
</script>

<svelte:window onkeydown={key} />

<section aria-label="Player">
  <button onclick={onback}>Close player</button>
  <span>{stream.name} [{streamLabel(stream)}]</span>
  {#if error}<p role="alert">{error}</p>{/if}
  {#if frame}
    <iframe title="Embedded player" src={embedUrl()} allowfullscreen style="width:100%;aspect-ratio:16/9;border:0"></iframe>
  {:else if src}
    <!-- svelte-ignore a11y_media_has_caption -->
    <video bind:this={video} src={src.url} controls autoplay playsinline onerror={onVideoError} style="width:100%;max-height:80vh"></video>
  {/if}
  {#if !frame}<button onclick={() => (frame = true)}>Use embedded player instead</button>{/if}
</section>
```

- [ ] **Step 3: `src/lib/Info.svelte`.** Import `Player`, add `let playing = $state<Stream | null>(null);`, make each stream a button, and render the player inside the info screen (so its state survives closing):

```svelte
      {#each streams as s}
        <li>
          <button disabled={streamLabel(s) === "Unsupported"} onclick={() => (playing = s)}>{s.name}</button>
          <small>[{streamLabel(s)}]</small>
        </li>
      {:else}<li>No streams (yet)</li>{/each}
```

```svelte
  {#if playing}
    {#key playing}
      <Player stream={playing} onback={() => (playing = null)} {onauth} />
    {/key}
  {/if}
```

Also reset `playing = null` at the start of `pick()`.

- [ ] **Step 4: Verify and commit.**

Run: `npm run build && npm run check`
Expected: exit 0, 0 errors, 0 warnings

Run: `cargo build -p proxer-app`
Expected: exit 0

```bash
git add src && git commit -m "feat(ui): in-app player with retry and embedded fallback"
```

---

### Task 5: Content Security Policy and manual walk-through

**Goal:** Replace the template's `csp: null` with a restrictive policy that still allows the player, and document the manual playback checks.

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Create: `docs/superpowers/plans/2026-10-20-plan2-manual-verification.md`

**Acceptance Criteria:**
- [ ] `app.security.csp` and `app.security.devCsp` are set (no longer `null`); `media-src` and `img-src` allow `https:`, `frame-src` allows only `https://*.proxer.me`, `connect-src` allows Tauri IPC (`ipc:` and `http://ipc.localhost`), and `devCsp` additionally allows the Vite dev server and its websocket on port 1420.
- [ ] The walk-through document lists the checks below and records the result for each.
- [ ] `cargo build -p proxer-app` and `npm run build` succeed.
- [ ] **Human-verified:** in `npm run tauri dev` the app still loads, logs in and searches (the CSP must not break IPC), and a normal and an airing stream both play.

**Verify:** `cargo build -p proxer-app && npm run build` → exit 0; manual walk-through for the rest

**Steps:**

- [ ] **Step 1: Edit `src-tauri/tauri.conf.json`:** replace `"csp": null` with:

```json
      "csp": "default-src 'self'; img-src 'self' https: data:; media-src 'self' https:; frame-src https://*.proxer.me; style-src 'self' 'unsafe-inline'; connect-src ipc: http://ipc.localhost",
      "devCsp": "default-src 'self'; img-src 'self' https: data:; media-src 'self' https:; frame-src https://*.proxer.me; style-src 'self' 'unsafe-inline'; connect-src ipc: http://ipc.localhost http://localhost:1420 ws://localhost:1420"
```

- [ ] **Step 2: Create `docs/superpowers/plans/2026-10-20-plan2-manual-verification.md`** with these checks (the human fills in results; agents cannot launch the GUI):

```markdown
# Plan 2 manual verification

Run `env WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri dev` (Wayland workaround).

1. App loads, auto-login works, search `naruto` works (CSP did not break IPC). If the window is blank or invoke fails, revert `csp`/`devCsp` to `null` and report the console error.
2. Normal stream: open Naruto (`/info/45`), episode 1, a language, click the [Normal] stream. Video plays; seek with the bar and with the arrow keys; Space pauses; Escape closes and returns to the same episode list.
3. Airing stream: open Tensei shitara Ken deshita 2 (`/info/59782`), episode 1, click the [Airing] stream. Video plays.
4. "Use embedded player instead" shows the embed in an iframe and it plays (or note what is blocked).
5. An episode with no streams shows "No streams (yet)".
6. Leave an airing stream paused for more than 3 hours (optional), then resume: it should re-resolve once or show "Playback failed" with the fallback button.
7. Results: note anything that failed, with the message shown.
```

- [ ] **Step 3: Build and commit.**

Run: `cargo build -p proxer-app && npm run build`
Expected: exit 0

```bash
git add src-tauri docs && git commit -m "feat(app): restrictive CSP; docs: plan 2 manual verification"
```

---

## Self-review

- **Spec coverage:** direct extraction (Tasks 1-3), airing tokens and expiry (Task 1, retry in Task 4), iframe fallback (Task 4), playback controls (Task 4: Escape, arrows, Space; Enter and Back remote keys are Plan 3), unsupported hosters (Tasks 3, 4), CSP (Task 5). Not covered, by design: media proxy (recorded deviation), open-in-system-browser fallback, Android/TV (Plan 3).
- **Placeholders:** none.
- **Type consistency:** `MediaSource {url, mime, expires_at}` matches in Rust, `api.ts` and `Player.svelte`; command `resolve_stream(stream)` matches `api.resolve`; `Stream` serializes `type` (kind renamed) and deserializes it back, so the object round-trips from `get_streams` to `resolve_stream`.
