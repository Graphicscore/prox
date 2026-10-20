# Proxer Core and Desktop UI Implementation Plan (Plan 1 of 3)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers-extended-cc:subagent-driven-development (recommended) or superpowers-extended-cc:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A desktop Tauri app that logs in to proxer.me with saved credentials, searches anime, and shows an info page with its episode streams listed (no playback yet).

**Architecture:** Cargo workspace with two pure-ish crates (`proxer-parse`: HTML to typed structs; `proxer-client`: HTTP, cookies, login, auto re-login) used by Tauri commands. Svelte + TypeScript UI calls the commands. Credentials live in the OS keychain via `keyring`.

**Tech Stack:** Rust (reqwest, scraper, serde, regex, keyring, tokio, wiremock for tests), Tauri v2, Svelte 5 + TypeScript + Vite.

**Spec:** `docs/superpowers/specs/2026-10-20-proxer-tauri-frontend-design.md`

**Out of this plan:** player, stream resolver, media proxy (Plan 2, shaped by Task 0's probe); Android, Android TV, D-pad navigation (Plan 3).

## Global Constraints

- Never log or print passwords, session cookies, or stream URLs containing query strings.
- All proxer.me access goes through `proxer-client`; the UI never fetches proxer.me.
- Parsers are pure (`&str` in, typed out) and tested against fixtures in `fixtures/`. Fixtures must never contain the user's cookies, username, or email (scrub before commit).
- Max 2 concurrent requests to proxer.me; no background polling.
- Errors are typed (`NotLoggedIn`, `BadCredentials`, `Parse`, `Network`); no string-matching on messages.
- Deliberate simplification vs the spec: the cookie jar is in-memory only. The app re-logs-in at start from stored credentials. (`ponytail:` upgrade to a persisted jar only if login becomes rate-limited.)

**User decisions (already made):** "I want it as an Tauri app"; platforms "3" (desktop + Android + Android TV); "it should have a login form and save username password"; spec approved ("looks goof"); "make sure to incorperate both different types of proxer streams, there is normal and there is airing" (airing example: https://proxer.me/info/59782/details). Normal streams have `type: "proxer-stream"`; airing streams have `type: "ps-test"`, name "Proxer-Stream Airing"; both `htype: "iframe"`; airing episodes are listed at `/info/<id>/list`.

---

### Task 0: Embed probe (spike, decides Plan 2)

**Goal:** Learn whether `stream.proxer.me` embeds expose a directly extractable media URL and whether they can be framed, and record the answer.

**Files:**
- Create: `docs/superpowers/specs/2026-10-20-embed-probe-findings.md`
- Create: `fixtures/embed-proxer-stream.html` (scrubbed)

**Acceptance Criteria:**
- [ ] Findings doc states, for one `proxer-stream` embed: media URL kind (HLS, MP4, or none found), whether URL carries a token or expiry, whether `X-Frame-Options`/`frame-ancestors` block framing, and required request headers (Referer, cookies).
- [ ] Findings doc ends with one line: `DECISION: direct-extraction` or `DECISION: iframe-fallback`.
- [ ] Fixture file contains no cookies, usernames, or emails.

**Verify:** `grep -E '^DECISION: (direct-extraction|iframe-fallback)$' docs/superpowers/specs/2026-10-20-embed-probe-findings.md` → one matching line

**Steps:**

- [ ] **Step 1: Get an embed URL from a logged-in session.** In the user's logged-in Chrome (Claude extension), open `https://proxer.me/watch/45/1/engsub` and run in the page:

```js
const w = await (await fetch(location.href)).text();
const s = JSON.parse(w.match(/var\s+streams\s*=\s*(\[[\s\S]*?\]);/)[1]);
s[0].replace.replace('#', s[0].code)
```

- [ ] **Step 2: Inspect the embed.** From the same page: `fetch(embedUrl)` and print the response headers `x-frame-options`, `content-security-policy`, plus the HTML with query strings redacted. Look for `.m3u8`, `.mp4`, `<source`, `file:` or packed JS (`eval(function(p,a,c,k`).

- [ ] **Step 3: Test headers from outside the browser.**

Run: `curl -sS -D - -o /dev/null -A 'Mozilla/5.0' -H 'Referer: https://proxer.me/' '<embedUrl>' | head -20`
Expected: record status and headers; repeat without the Referer to see whether it is required.

- [ ] **Step 4: Write the findings doc and the `DECISION:` line, save the scrubbed embed HTML to `fixtures/embed-proxer-stream.html`, commit.**

```bash
git add docs fixtures && git commit -m "docs: embed probe findings"
```

---

### Task 0b: Airing embed probe (spike, decides Plan 2's airing path)

**Goal:** Learn whether the airing stream (`type: "ps-test"`) embed exposes a directly extractable media URL, like the normal stream, and whether its embed differs (host, player, tokens, framing).

**Files:**
- Create: `docs/superpowers/specs/2026-10-20-airing-embed-probe-findings.md`
- Create: `fixtures/embed-ps-test.html` (scrubbed; query strings and tokens replaced with `<redacted>`)

**Acceptance Criteria:**
- [ ] Findings doc states, for one airing episode of `https://proxer.me/info/59782` (episode list at `/info/59782/list`): the embed host, whether the page has a `<source>`/m3u8/mp4 URL, whether the URL carries a token or expiry, whether the embed can be framed (`X-Frame-Options`/CSP), and required request headers.
- [ ] Findings doc ends with one line: `DECISION: direct-extraction` or `DECISION: iframe-only` for airing streams.
- [ ] Fixture contains no cookies, usernames, tokens, or query strings.

**Verify:** `grep -E '^DECISION: (direct-extraction|iframe-only)$' docs/superpowers/specs/2026-10-20-airing-embed-probe-findings.md` → one matching line

**Steps:**

- [ ] **Step 1: Get an airing embed URL** in the logged-in Chrome: fetch `/info/59782/list`, take the first `/watch/59782/<n>/engsub` link, fetch it, read `var streams`, and build `replace.replace('#', code)` for the entry with `type` `ps-test`. Print only the host and the path with long ids masked.
- [ ] **Step 2: Inspect the embed** as in Task 0 (headers and `<source>` or `.m3u8`/`.mp4`, packed JS, query-string tokens). Note differences from the normal `proxer-stream` embed.
- [ ] **Step 3: Write the findings doc with its `DECISION:` line, save the scrubbed embed HTML, commit.**

```bash
git add docs fixtures && git commit -m "docs: airing embed probe findings"
```

---

### Task 1: Scaffold Tauri v2 + Svelte workspace

**Goal:** A buildable Tauri v2 app with a Cargo workspace containing empty `proxer-parse` and `proxer-client` crates.

**Files:**
- Create: `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`, `src/main.ts`, `src/App.svelte`
- Create: `src-tauri/` (Tauri v2 app crate named `proxer-app`)
- Create: `crates/proxer-parse/{Cargo.toml,src/lib.rs}`, `crates/proxer-client/{Cargo.toml,src/lib.rs}`
- Create: `Cargo.toml` (workspace root), `.gitignore`

**Acceptance Criteria:**
- [ ] `cargo test --workspace` succeeds (zero tests is fine).
- [ ] `npm run build` succeeds.
- [ ] `npm run tauri dev` opens a window showing the text "Proxer".

**Verify:** `cargo test --workspace && npm run build` → both exit 0

**Steps:**

- [ ] **Step 1: Scaffold.** Run `npm create tauri-app@latest . -- --template svelte-ts --manager npm --identifier me.louven.proxer` (answer prompts to keep the current directory). Move `src-tauri` crate name to `proxer-app` in `src-tauri/Cargo.toml`.

- [ ] **Step 2: Workspace root `Cargo.toml`.**

```toml
[workspace]
resolver = "2"
members = ["src-tauri", "crates/proxer-parse", "crates/proxer-client"]
```

- [ ] **Step 3: `crates/proxer-parse/Cargo.toml`.**

```toml
[package]
name = "proxer-parse"
version = "0.1.0"
edition = "2021"

[dependencies]
scraper = "0.20"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
regex = "1"
thiserror = "1"
```

- [ ] **Step 4: `crates/proxer-client/Cargo.toml`.**

```toml
[package]
name = "proxer-client"
version = "0.1.0"
edition = "2021"

[dependencies]
proxer-parse = { path = "../proxer-parse" }
reqwest = { version = "0.12", default-features = false, features = ["cookies", "rustls-tls"] }
tokio = { version = "1", features = ["sync", "macros", "rt-multi-thread"] }
thiserror = "1"

[dev-dependencies]
wiremock = "0.6"
```

- [ ] **Step 5: Empty `lib.rs` in both crates, `.gitignore` with `target/ node_modules/ dist/ .remember/`, then verify.**

Run: `cargo test --workspace && npm run build`
Expected: both exit 0

- [ ] **Step 6: Commit.**

```bash
git add -A && git commit -m "chore: scaffold Tauri v2 + Svelte workspace"
```

---

### Task 2: proxer-parse: search results

**Goal:** `parse_search(html) -> Vec<SearchHit>` from the search result table.

**Files:**
- Create: `crates/proxer-parse/src/search.rs`
- Modify: `crates/proxer-parse/src/lib.rs`
- Create: `fixtures/search-naruto.html` (scrubbed copy of `https://proxer.me/search?s=search&name=naruto&type=all-anime`, saved from a browser or curl; anonymous-readable)
- Test: `crates/proxer-parse/tests/search.rs`

**Acceptance Criteria:**
- [ ] On the fixture, the result contains a hit with `id == 12092` titled `NARUTO外伝 七代目火影と緋色の花つ月`, kind `Mangaserie`, genres including `Action` and `Shounen`, rating `7.55`.
- [ ] A row without an `/info/` link is skipped, not a panic.
- [ ] Empty or unrelated HTML returns an empty `Vec`.

**Verify:** `cargo test -p proxer-parse search` → all tests pass

**Steps:**

- [ ] **Step 1: Save the fixture.**

Run: `curl -sSL -A 'Mozilla/5.0' 'https://proxer.me/search?s=search&name=naruto&type=all-anime' -o fixtures/search-naruto.html && grep -c 'entry12092' fixtures/search-naruto.html`
Expected: `1` or more. Note: if the saved cookie header is in the file, remove it (the HTML body holds none).

- [ ] **Step 2: Write the failing test `crates/proxer-parse/tests/search.rs`.**

```rust
use proxer_parse::{parse_search, SearchHit};

#[test]
fn parses_known_row() {
    let html = std::fs::read_to_string("../../fixtures/search-naruto.html").unwrap();
    let hits = parse_search(&html);
    let h: &SearchHit = hits.iter().find(|h| h.id == 12092).expect("row 12092");
    assert_eq!(h.title, "NARUTO外伝 七代目火影と緋色の花つ月");
    assert_eq!(h.kind, "Mangaserie");
    assert!(h.genres.contains(&"Action".to_string()));
    assert!(h.genres.contains(&"Shounen".to_string()));
    assert_eq!(h.rating, Some(7.55));
}

#[test]
fn garbage_gives_empty() {
    assert!(parse_search("<html><body>nothing</body></html>").is_empty());
    assert!(parse_search("").is_empty());
}

#[test]
fn row_without_info_link_is_skipped() {
    let html = r#"<table><tr class="entry1"><td></td><td>no link</td></tr></table>"#;
    assert!(parse_search(html).is_empty());
}
```

- [ ] **Step 3: Run to verify failure.**

Run: `cargo test -p proxer-parse search`
Expected: FAIL (`parse_search` not found)

- [ ] **Step 4: Implement `crates/proxer-parse/src/search.rs`.** Cell layout seen in the live page: `[status img, title link, genres, kind, flag, rating "7.55 (427)"]`.

```rust
use scraper::{Html, Selector};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchHit {
    pub id: u32,
    pub title: String,
    pub genres: Vec<String>,
    pub kind: String,
    pub rating: Option<f32>,
}

fn text(e: scraper::ElementRef) -> String {
    e.text().collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn parse_search(html: &str) -> Vec<SearchHit> {
    let doc = Html::parse_document(html);
    let row = Selector::parse("tr[class^=entry]").unwrap();
    let td = Selector::parse("td").unwrap();
    let link = Selector::parse("a[href^='/info/']").unwrap();
    doc.select(&row)
        .filter_map(|r| {
            let a = r.select(&link).next()?;
            let id = a.value().attr("href")?.trim_start_matches("/info/").split('#').next()?.parse().ok()?;
            let cells: Vec<_> = r.select(&td).collect();
            let genres = cells.get(2).map(|c| text(*c).split(' ').filter(|s| !s.is_empty()).map(String::from).collect()).unwrap_or_default();
            let kind = cells.get(3).map(|c| text(*c)).unwrap_or_default();
            let rating = cells.get(5).and_then(|c| text(*c).split(' ').next().and_then(|n| n.parse().ok()));
            Some(SearchHit { id, title: text(a), genres, kind, rating })
        })
        .collect()
}
```

- [ ] **Step 5: Export from `lib.rs`.**

```rust
mod search;
pub use search::{parse_search, SearchHit};
```

- [ ] **Step 6: Run to verify pass, then commit.**

Run: `cargo test -p proxer-parse search`
Expected: 3 passed

```bash
git add -A && git commit -m "feat(parse): search results parser with fixture test"
```

---

### Task 3: proxer-parse: info, streams (normal + airing), episodes, login helpers

**Goal:** `parse_info`, `parse_streams` (with `Stream::stream_kind()` distinguishing Normal and Airing), `parse_episodes`, `login_action`, `is_logged_in` in `proxer-parse`.

**Files:**
- Create: `crates/proxer-parse/src/{info.rs,streams.rs,episodes.rs,session.rs}`
- Modify: `crates/proxer-parse/src/lib.rs`
- Create: `fixtures/info-45.html`, `fixtures/watch-45-1.html` (saved from the logged-in browser via Save Page / `copy(document.documentElement.outerHTML)`, then scrubbed of username and cookies)
- Create: `fixtures/home-anon.html` (anonymous `curl https://proxer.me/`)
- Test: `crates/proxer-parse/tests/{info.rs,streams.rs,episodes.rs,session.rs}`

**Acceptance Criteria:**
- [ ] `parse_info(info-45)` gives `title == "Naruto"` and `fields["Status"] == "Abgeschlossen"`.
- [ ] `parse_streams(watch-45-1)` returns at least one stream with `kind == "proxer-stream"` and `replace == "https://stream.proxer.me/embed-#-728x504.html"`.
- [ ] `parse_streams` on a page without a `streams` variable returns `Err(ParseError::NoStreams)`.
- [ ] A stream with `type: "proxer-stream"` has `stream_kind() == StreamKind::Normal`; `type: "ps-test"` gives `StreamKind::Airing`; any other type gives `StreamKind::Other`.
- [ ] `parse_episodes(html, 59782)` on a list page containing links `/watch/59782/1/engsub`, `/watch/59782/2/engsub`, `/watch/59782/2/gersub` returns episode 1 with langs `["engsub"]` and episode 2 with langs `["engsub","gersub"]`, sorted by number; a page with no matching links returns an empty `Vec`.
- [ ] `login_action(home-anon)` returns a path starting with `/login?`; `is_logged_in(home-anon) == false`; `is_logged_in(info-45) == true`.

**Verify:** `cargo test -p proxer-parse` → all tests pass

**Steps:**

- [ ] **Step 1: Save fixtures.** Anonymous home: `curl -sSL -A 'Mozilla/5.0' https://proxer.me/ -o fixtures/home-anon.html`. For the two logged-in pages, in the logged-in Chrome run `copy(document.documentElement.outerHTML)` on `https://proxer.me/info/45` and `https://proxer.me/watch/45/1/engsub`, paste into the fixture files, then `grep -il 'asteria' fixtures/*` must print nothing (replace the username with `testuser` if found).

- [ ] **Step 2: Write the failing tests.**

`crates/proxer-parse/tests/info.rs`:
```rust
use proxer_parse::parse_info;

#[test]
fn parses_info_fixture() {
    let html = std::fs::read_to_string("../../fixtures/info-45.html").unwrap();
    let info = parse_info(&html);
    assert_eq!(info.title, "Naruto");
    assert_eq!(info.fields.get("Status").map(String::as_str), Some("Abgeschlossen"));
}
```

`crates/proxer-parse/tests/streams.rs`:
```rust
use proxer_parse::{parse_streams, ParseError, StreamKind};

#[test]
fn parses_streams_fixture() {
    let html = std::fs::read_to_string("../../fixtures/watch-45-1.html").unwrap();
    let s = parse_streams(&html).unwrap();
    assert!(s.iter().any(|x| x.kind == "proxer-stream"
        && x.replace == "https://stream.proxer.me/embed-#-728x504.html"));
}

#[test]
fn missing_streams_is_error() {
    assert!(matches!(parse_streams("<html></html>"), Err(ParseError::NoStreams)));
}

fn page(kind: &str, name: &str) -> String {
    format!(r#"<script>var streams = [{{"id":"1","code":"abc","type":"{kind}","htype":"iframe","name":"{name}","replace":"https://example.test/embed-#-728x504.html"}}];</script>"#)
}

#[test]
fn normal_and_airing_kinds() {
    let n = parse_streams(&page("proxer-stream", "Proxer-Stream")).unwrap();
    let a = parse_streams(&page("ps-test", "Proxer-Stream Airing")).unwrap();
    let o = parse_streams(&page("something-else", "X")).unwrap();
    assert_eq!(n[0].stream_kind(), StreamKind::Normal);
    assert_eq!(a[0].stream_kind(), StreamKind::Airing);
    assert_eq!(o[0].stream_kind(), StreamKind::Other);
}

#[test]
fn empty_streams_array_is_ok() {
    let s = parse_streams("<script>var streams = [];</script>").unwrap();
    assert!(s.is_empty());
}
```

`crates/proxer-parse/tests/episodes.rs`:
```rust
use proxer_parse::{parse_episodes, Episode};

#[test]
fn groups_languages_per_episode() {
    let html = r#"<a href="/watch/59782/2/gersub">x</a><a href="/watch/59782/1/engsub">x</a>
                  <a href="/watch/59782/2/engsub">x</a><a href="/watch/99/1/engsub">other anime</a>"#;
    assert_eq!(
        parse_episodes(html, 59782),
        vec![
            Episode { number: 1, langs: vec!["engsub".into()] },
            Episode { number: 2, langs: vec!["engsub".into(), "gersub".into()] },
        ]
    );
}

#[test]
fn no_links_is_empty() {
    assert!(parse_episodes("<html></html>", 1).is_empty());
}
```

`crates/proxer-parse/tests/session.rs`:
```rust
use proxer_parse::{is_logged_in, login_action};

#[test]
fn anon_home_has_login_form() {
    let html = std::fs::read_to_string("../../fixtures/home-anon.html").unwrap();
    assert!(login_action(&html).unwrap().starts_with("/login?"));
    assert!(!is_logged_in(&html));
}

#[test]
fn info_fixture_is_logged_in() {
    let html = std::fs::read_to_string("../../fixtures/info-45.html").unwrap();
    assert!(is_logged_in(&html));
}
```

- [ ] **Step 3: Run to verify failure.**

Run: `cargo test -p proxer-parse`
Expected: FAIL (functions not defined)

- [ ] **Step 4: Implement `info.rs`.** Label/value pairs come from table rows with two cells; the title comes from `<title>` ("Naruto (Anime) - Details - Proxer.Me").

```rust
use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Info {
    pub title: String,
    pub fields: BTreeMap<String, String>,
}

fn clean(s: String) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn parse_info(html: &str) -> Info {
    let doc = Html::parse_document(html);
    let title_sel = Selector::parse("title").unwrap();
    let raw = doc.select(&title_sel).next().map(|t| t.text().collect::<String>()).unwrap_or_default();
    let title = raw.split(" (").next().unwrap_or("").trim().to_string();
    let tr = Selector::parse("tr").unwrap();
    let td = Selector::parse("td").unwrap();
    let mut fields = BTreeMap::new();
    for row in doc.select(&tr) {
        let cells: Vec<_> = row.select(&td).collect();
        if let [k, v] = cells.as_slice() {
            let key = clean(k.text().collect());
            let val = clean(v.text().collect());
            if !key.is_empty() && !val.is_empty() {
                fields.entry(key).or_insert(val);
            }
        }
    }
    Info { title, fields }
}
```

> If Step 6 shows the label cell is a `<th>` or the key text carries a trailing colon, adjust the selector (`td, th`) and `trim_end_matches(':')` here, then re-run. The acceptance test is the arbiter.

- [ ] **Step 5: Implement `streams.rs` and `session.rs`.**

`streams.rs`:
```rust
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("no streams variable on page")]
    NoStreams,
    #[error("streams JSON invalid: {0}")]
    BadJson(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stream {
    pub code: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    pub replace: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum StreamKind {
    /// `proxer-stream`: finished anime hosted by Proxer.
    Normal,
    /// `ps-test` ("Proxer-Stream Airing"): currently airing anime.
    Airing,
    Other,
}

impl Stream {
    pub fn stream_kind(&self) -> StreamKind {
        match self.kind.as_str() {
            "proxer-stream" => StreamKind::Normal,
            "ps-test" => StreamKind::Airing,
            _ => StreamKind::Other,
        }
    }
}

pub fn parse_streams(html: &str) -> Result<Vec<Stream>, ParseError> {
    let re = Regex::new(r"var\s+streams\s*=\s*(\[[\s\S]*?\]);").unwrap();
    let caps = re.captures(html).ok_or(ParseError::NoStreams)?;
    Ok(serde_json::from_str(&caps[1])?)
}
```

`episodes.rs` (episode links live on `/info/<id>/list`, shaped `/watch/<id>/<n>/<lang>`):
```rust
use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Episode {
    pub number: u32,
    pub langs: Vec<String>,
}

pub fn parse_episodes(html: &str, id: u32) -> Vec<Episode> {
    let re = Regex::new(&format!(r"/watch/{id}/(\d+)/([a-z]+)")).unwrap();
    let mut by_ep: BTreeMap<u32, BTreeSet<String>> = BTreeMap::new();
    for c in re.captures_iter(html) {
        if let Ok(n) = c[1].parse() {
            by_ep.entry(n).or_default().insert(c[2].to_string());
        }
    }
    by_ep.into_iter().map(|(number, langs)| Episode { number, langs: langs.into_iter().collect() }).collect()
}
```

`session.rs`:
```rust
use scraper::{Html, Selector};

pub fn login_action(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("form[action^='/login']").unwrap();
    doc.select(&sel).next()?.value().attr("action").map(String::from)
}

pub fn is_logged_in(html: &str) -> bool {
    html.contains("id=\"mod_logout_submit\"")
}
```

- [ ] **Step 6: Export from `lib.rs`, run, fix selectors against the fixtures, commit.**

```rust
mod episodes;
mod info;
mod search;
mod session;
mod streams;
pub use episodes::{parse_episodes, Episode};
pub use info::{parse_info, Info};
pub use search::{parse_search, SearchHit};
pub use session::{is_logged_in, login_action};
pub use streams::{parse_streams, ParseError, Stream, StreamKind};
```

Run: `cargo test -p proxer-parse`
Expected: all pass

```bash
git add -A && git commit -m "feat(parse): info, streams, session helpers"
```

---

### Task 4: proxer-client: login, fetch, auto re-login

**Goal:** `Client` that logs in, fetches pages, and transparently re-logs-in once when the session is lost.

**Files:**
- Modify: `crates/proxer-client/src/lib.rs`
- Test: `crates/proxer-client/tests/client.rs`

**Acceptance Criteria:**
- [ ] `login` with correct credentials against a mocked server returns `Ok`; wrong credentials return `Err(Error::BadCredentials)`.
- [ ] `get` on a page that first returns a logged-out body then, after re-login, a logged-in body returns the logged-in body and issues exactly one login POST.
- [ ] `get` after a failed re-login returns `Err(Error::NotLoggedIn)` and does not loop.
- [ ] `get` without any stored credentials and a logged-out body returns `Err(Error::NotLoggedIn)`.

**Verify:** `cargo test -p proxer-client` → all tests pass

**Steps:**

- [ ] **Step 1: Write the failing tests `crates/proxer-client/tests/client.rs`.**

```rust
use proxer_client::{Client, Error};
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const LOGIN_PAGE: &str = r#"<form method="post" action="/login?abc123=1"><input name="username"><input name="password"></form>"#;
const IN: &str = r#"<a id="mod_logout_submit">out</a>"#;

#[tokio::test]
async fn login_ok_and_bad() {
    let s = MockServer::start().await;
    Mock::given(method("GET")).and(path("/")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).mount(&s).await;
    Mock::given(method("POST")).and(path_regex("^/login")).respond_with(ResponseTemplate::new(200).set_body_string(IN)).up_to_n_times(1).mount(&s).await;
    let c = Client::new(&s.uri());
    assert!(c.login("u", "p").await.is_ok());
    Mock::given(method("POST")).and(path_regex("^/login")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).mount(&s).await;
    assert!(matches!(c.login("u", "bad").await, Err(Error::BadCredentials)));
}

#[tokio::test]
async fn get_relogins_once() {
    let s = MockServer::start().await;
    Mock::given(method("GET")).and(path("/")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).mount(&s).await;
    Mock::given(method("POST")).and(path_regex("^/login")).respond_with(ResponseTemplate::new(200).set_body_string(IN)).expect(1).mount(&s).await;
    Mock::given(method("GET")).and(path("/info/1")).respond_with(ResponseTemplate::new(200).set_body_string("<html>404</html>")).up_to_n_times(1).mount(&s).await;
    Mock::given(method("GET")).and(path("/info/1")).respond_with(ResponseTemplate::new(200).set_body_string(IN)).mount(&s).await;
    let c = Client::new(&s.uri());
    c.set_credentials("u", "p").await;
    assert_eq!(c.get("/info/1").await.unwrap(), IN);
}

#[tokio::test]
async fn get_without_credentials_is_not_logged_in() {
    let s = MockServer::start().await;
    Mock::given(method("GET")).and(path("/info/1")).respond_with(ResponseTemplate::new(200).set_body_string("<html></html>")).mount(&s).await;
    let c = Client::new(&s.uri());
    assert!(matches!(c.get("/info/1").await, Err(Error::NotLoggedIn)));
}
```

- [ ] **Step 2: Run to verify failure.**

Run: `cargo test -p proxer-client`
Expected: FAIL (`Client` not defined)

- [ ] **Step 3: Implement `crates/proxer-client/src/lib.rs`.**

```rust
use tokio::sync::{Mutex, Semaphore};

const UA: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not logged in")]
    NotLoggedIn,
    #[error("bad credentials")]
    BadCredentials,
    #[error("page structure unexpected: {0}")]
    Parse(&'static str),
    #[error("network: {0}")]
    Network(#[from] reqwest::Error),
}

pub struct Client {
    http: reqwest::Client,
    base: String,
    creds: Mutex<Option<(String, String)>>,
    gate: Semaphore, // max 2 concurrent requests (spec: politeness)
}

impl Client {
    pub fn new(base: &str) -> Self {
        Self {
            http: reqwest::Client::builder().cookie_store(true).user_agent(UA).build().expect("http client"),
            base: base.trim_end_matches('/').to_string(),
            creds: Mutex::new(None),
            gate: Semaphore::new(2),
        }
    }

    pub async fn set_credentials(&self, user: &str, pass: &str) {
        *self.creds.lock().await = Some((user.into(), pass.into()));
    }

    pub async fn clear_credentials(&self) {
        *self.creds.lock().await = None;
    }

    pub async fn login(&self, user: &str, pass: &str) -> Result<(), Error> {
        let _p = self.gate.acquire().await.expect("semaphore");
        let home = self.http.get(&self.base).send().await?.text().await?;
        let action = proxer_parse::login_action(&home).ok_or(Error::Parse("login form"))?;
        let res = self
            .http
            .post(format!("{}{}", self.base, action))
            .form(&[("username", user), ("password", pass)])
            .send()
            .await?
            .text()
            .await?;
        if proxer_parse::is_logged_in(&res) {
            self.set_credentials(user, pass).await;
            Ok(())
        } else {
            Err(Error::BadCredentials)
        }
    }


    async fn fetch(&self, path: &str) -> Result<String, Error> {
        let _p = self.gate.acquire().await.expect("semaphore");
        Ok(self.http.get(format!("{}{}", self.base, path)).send().await?.text().await?)
    }

    /// GET a page; if it comes back logged out, log in again once and retry once.
    pub async fn get(&self, path: &str) -> Result<String, Error> {
        let body = self.fetch(path).await?;
        if proxer_parse::is_logged_in(&body) {
            return Ok(body);
        }
        let creds = self.creds.lock().await.clone();
        let (u, p) = creds.ok_or(Error::NotLoggedIn)?;
        self.login(&u, &p).await.map_err(|e| match e {
            Error::BadCredentials => Error::NotLoggedIn,
            other => other,
        })?;
        let body = self.fetch(path).await?;
        if proxer_parse::is_logged_in(&body) { Ok(body) } else { Err(Error::NotLoggedIn) }
    }
}
```

> Note: the `login` helper acquires the semaphore, and `get` must not hold a permit while calling it (it does not: `fetch` releases its permit on return).

- [ ] **Step 4: Run to verify pass, then commit.**

Run: `cargo test -p proxer-client`
Expected: 3 passed

```bash
git add -A && git commit -m "feat(client): login, fetch, auto re-login"
```

---

### Task 5: Tauri commands and credential store

**Goal:** Commands `login`, `logout`, `auto_login`, `search`, `get_info`, `get_episodes`, `get_streams`, with credentials in the OS keychain.

**Files:**
- Modify: `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`
- Create: `src-tauri/src/creds.rs`, `src-tauri/src/commands.rs`
- Test: `src-tauri/src/creds.rs` (`#[cfg(test)]`, uses an in-memory trait object)

**Acceptance Criteria:**
- [ ] `cargo build -p proxer-app` succeeds.
- [ ] A unit test shows `CredStore` (trait) round-trips `save`/`load`/`clear` with the in-memory implementation.
- [ ] `auto_login` returns `false` when no credentials are stored and `true` after a successful `login` with `remember = true` followed by an app restart simulation (new `CredStore` load).
- [ ] No command returns or logs the password.

**Verify:** `cargo test -p proxer-app && cargo build -p proxer-app` → exit 0

**Steps:**

- [ ] **Step 1: Add dependencies to `src-tauri/Cargo.toml`.**

```toml
[dependencies]
proxer-parse = { path = "../crates/proxer-parse" }
proxer-client = { path = "../crates/proxer-client" }
keyring = { version = "3", features = ["apple-native", "windows-native", "sync-secret-service"] }
tokio = { version = "1", features = ["sync"] }
serde = { version = "1", features = ["derive"] }
```

(Keep the existing `tauri`, `tauri-build`, and `serde_json` entries from the scaffold.)

- [ ] **Step 2: Write the failing test and trait in `src-tauri/src/creds.rs`.**

```rust
pub trait CredStore: Send + Sync {
    fn save(&self, user: &str, pass: &str) -> Result<(), String>;
    fn load(&self) -> Option<(String, String)>;
    fn clear(&self);
}

pub struct KeyringStore;
const SERVICE: &str = "me.louven.proxer";

impl CredStore for KeyringStore {
    fn save(&self, user: &str, pass: &str) -> Result<(), String> {
        keyring::Entry::new(SERVICE, "account").and_then(|e| e.set_password(&format!("{user}\n{pass}"))).map_err(|e| e.to_string())
    }
    fn load(&self) -> Option<(String, String)> {
        let s = keyring::Entry::new(SERVICE, "account").ok()?.get_password().ok()?;
        let (u, p) = s.split_once('\n')?;
        Some((u.into(), p.into()))
    }
    fn clear(&self) {
        if let Ok(e) = keyring::Entry::new(SERVICE, "account") { let _ = e.delete_credential(); }
    }
}

#[cfg(test)]
pub struct MemStore(pub std::sync::Mutex<Option<(String, String)>>);
#[cfg(test)]
impl CredStore for MemStore {
    fn save(&self, u: &str, p: &str) -> Result<(), String> { *self.0.lock().unwrap() = Some((u.into(), p.into())); Ok(()) }
    fn load(&self) -> Option<(String, String)> { self.0.lock().unwrap().clone() }
    fn clear(&self) { *self.0.lock().unwrap() = None; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip() {
        let s = MemStore(Default::default());
        assert!(s.load().is_none());
        s.save("u", "p").unwrap();
        assert_eq!(s.load(), Some(("u".into(), "p".into())));
        s.clear();
        assert!(s.load().is_none());
    }
}
```

Run: `cargo test -p proxer-app creds`
Expected: PASS for `round_trip` (it should fail first only if the file is missing from `mod` declarations: add `mod creds;` in `lib.rs` before running).

- [ ] **Step 3: `src-tauri/src/commands.rs`.**

```rust
use crate::creds::CredStore;
use proxer_client::{Client, Error};
use proxer_parse::{parse_episodes, parse_info, parse_search, parse_streams, Episode, Info, SearchHit, Stream};
use std::sync::Arc;
use tauri::State;

pub struct App {
    pub client: Client,
    pub store: Arc<dyn CredStore>,
}

fn err(e: Error) -> String {
    match e {
        Error::NotLoggedIn => "NotLoggedIn".into(),
        Error::BadCredentials => "BadCredentials".into(),
        Error::Parse(p) => format!("Parse:{p}"),
        Error::Network(_) => "Network".into(),
    }
}

#[tauri::command]
pub async fn login(app: State<'_, App>, username: String, password: String, remember: bool) -> Result<(), String> {
    app.client.login(&username, &password).await.map_err(err)?;
    if remember { app.store.save(&username, &password)?; }
    Ok(())
}

#[tauri::command]
pub async fn auto_login(app: State<'_, App>) -> Result<bool, String> {
    let Some((u, p)) = app.store.load() else { return Ok(false) };
    match app.client.login(&u, &p).await {
        Ok(()) => Ok(true),
        Err(Error::BadCredentials) => { app.store.clear(); Ok(false) }
        Err(e) => Err(err(e)),
    }
}

#[tauri::command]
pub async fn logout(app: State<'_, App>) -> Result<(), String> {
    app.store.clear();
    app.client.clear_credentials().await;
    Ok(())
}

#[tauri::command]
pub async fn search(app: State<'_, App>, query: String) -> Result<Vec<SearchHit>, String> {
    let q: String = url_encode(&query);
    let html = app.client.get(&format!("/search?s=search&name={q}&type=all-anime")).await.map_err(err)?;
    Ok(parse_search(&html))
}

#[tauri::command]
pub async fn get_info(app: State<'_, App>, id: u32) -> Result<Info, String> {
    Ok(parse_info(&app.client.get(&format!("/info/{id}")).await.map_err(err)?))
}

#[tauri::command]
pub async fn get_episodes(app: State<'_, App>, id: u32) -> Result<Vec<Episode>, String> {
    Ok(parse_episodes(&app.client.get(&format!("/info/{id}/list")).await.map_err(err)?, id))
}

#[tauri::command]
pub async fn get_streams(app: State<'_, App>, id: u32, episode: u32, lang: String) -> Result<Vec<Stream>, String> {
    let lang: String = lang.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let html = app.client.get(&format!("/watch/{id}/{episode}/{lang}")).await.map_err(err)?;
    parse_streams(&html).map_err(|_| "Parse:streams".into())
}

fn url_encode(s: &str) -> String {
    s.bytes().map(|b| match b {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
        _ => format!("%{b:02X}"),
    }).collect()
}
```

- [ ] **Step 4: Wire in `src-tauri/src/lib.rs`.**

```rust
mod commands;
mod creds;

use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(commands::App {
            client: proxer_client::Client::new("https://proxer.me"),
            store: Arc::new(creds::KeyringStore),
        })
        .invoke_handler(tauri::generate_handler![
            commands::login, commands::auto_login, commands::logout,
            commands::search, commands::get_info, commands::get_episodes, commands::get_streams
        ])
        .run(tauri::generate_context!())
        .expect("tauri run");
}
```

- [ ] **Step 5: Build, test, commit.**

Run: `cargo test -p proxer-app && cargo build -p proxer-app`
Expected: exit 0

```bash
git add -A && git commit -m "feat(app): tauri commands and keychain credential store"
```

---

### Task 6: Svelte UI: login, search, info (desktop)

**Goal:** Working desktop screens: Login (saved credentials, auto-login), Search, Info with stream list; Settings logout.

**Files:**
- Modify: `src/App.svelte`
- Create: `src/lib/api.ts`, `src/lib/Login.svelte`, `src/lib/Search.svelte`, `src/lib/Info.svelte`

**Acceptance Criteria:**
- [ ] On first start the Login form shows; after logging in with "remember me" checked, restarting the app skips the form.
- [ ] Wrong credentials show "Wrong username or password" and stay on the form.
- [ ] Searching `naruto` lists rows including `NARUTO外伝 七代目火影と緋色の花つ月`.
- [ ] Opening a hit shows its title, Status field, and an episode list; picking an episode+language lists its streams, each labelled Normal (`proxer-stream`) or Airing (`ps-test`). Opening an airing anime (`/info/59782`, Tensei shitara Ken deshita 2) shows its airing streams, and an episode without streams shows "No streams (yet)". A `NotLoggedIn` error returns to the Login screen.
- [ ] "Log out" clears saved credentials (next start shows the form).

**Verify:** `npm run build && npm run check` → exit 0; then manual run `npm run tauri dev` and walk the acceptance list

**Steps:**

- [ ] **Step 1: `src/lib/api.ts`.**

```ts
import { invoke } from "@tauri-apps/api/core";

export type SearchHit = { id: number; title: string; genres: string[]; kind: string; rating: number | null };
export type Info = { title: string; fields: Record<string, string> };
export type Stream = { code: string; type: string; name: string; replace: string };
export type Episode = { number: number; langs: string[] };
export const isAiring = (s: Stream) => s.type === "ps-test";

export const api = {
  login: (username: string, password: string, remember: boolean) => invoke<void>("login", { username, password, remember }),
  autoLogin: () => invoke<boolean>("auto_login"),
  logout: () => invoke<void>("logout"),
  search: (query: string) => invoke<SearchHit[]>("search", { query }),
  info: (id: number) => invoke<Info>("get_info", { id }),
  episodes: (id: number) => invoke<Episode[]>("get_episodes", { id }),
  streams: (id: number, episode: number, lang: string) => invoke<Stream[]>("get_streams", { id, episode, lang }),
};
```

- [ ] **Step 2: `src/lib/Login.svelte`.**

```svelte
<script lang="ts">
  import { api } from "./api";
  let { onok }: { onok: () => void } = $props();
  let username = $state(""), password = $state(""), remember = $state(true), error = $state(""), busy = $state(false);
  async function submit(e: Event) {
    e.preventDefault(); busy = true; error = "";
    try { await api.login(username, password, remember); onok(); }
    catch (x) { error = x === "BadCredentials" ? "Wrong username or password" : "Could not reach Proxer"; }
    busy = false;
  }
</script>

<form onsubmit={submit}>
  <h1>Proxer</h1>
  <input bind:value={username} placeholder="Username" autocomplete="username" required />
  <input bind:value={password} type="password" placeholder="Password" autocomplete="current-password" required />
  <label><input type="checkbox" bind:checked={remember} /> Remember me</label>
  {#if error}<p role="alert">{error}</p>{/if}
  <button disabled={busy}>Log in</button>
</form>
```

- [ ] **Step 3: `src/lib/Search.svelte` and `src/lib/Info.svelte`.**

`Search.svelte`:
```svelte
<script lang="ts">
  import { api, type SearchHit } from "./api";
  let { onopen, onauth }: { onopen: (id: number) => void; onauth: () => void } = $props();
  let q = $state(""), hits = $state<SearchHit[]>([]), error = $state("");
  async function go(e: Event) {
    e.preventDefault(); error = "";
    try { hits = await api.search(q); }
    catch (x) { x === "NotLoggedIn" ? onauth() : (error = "Search failed"); }
  }
</script>

<form onsubmit={go}><input bind:value={q} placeholder="Search anime" /><button>Search</button></form>
{#if error}<p role="alert">{error}</p>{/if}
<ul>
  {#each hits as h}
    <li><button onclick={() => onopen(h.id)}>{h.title}</button> <small>{h.kind} · {h.genres.join(", ")} · {h.rating ?? "–"}</small></li>
  {/each}
</ul>
```

`Info.svelte` (episode picker; each stream is labelled Normal or Airing):
```svelte
<script lang="ts">
  import { api, isAiring, type Info, type Episode, type Stream } from "./api";
  let { id, onback, onauth }: { id: number; onback: () => void; onauth: () => void } = $props();
  let info = $state<Info | null>(null), episodes = $state<Episode[]>([]), streams = $state<Stream[]>([]);
  let selected = $state<{ ep: number; lang: string } | null>(null), error = $state("");
  function fail(x: unknown) { x === "NotLoggedIn" ? onauth() : (error = "Could not load"); }
  $effect(() => {
    (async () => {
      try { info = await api.info(id); episodes = await api.episodes(id); } catch (x) { fail(x); }
    })();
  });
  async function pick(ep: number, lang: string) {
    selected = { ep, lang }; streams = [];
    try { streams = await api.streams(id, ep, lang); } catch (x) { x === "Parse:streams" ? (streams = []) : fail(x); }
  }
</script>

<button onclick={onback}>Back</button>
{#if error}<p role="alert">{error}</p>{/if}
{#if info}
  <h2>{info.title}</h2>
  <p>Status: {info.fields["Status"] ?? "–"}</p>
  <h3>Episodes</h3>
  <ul>
    {#each episodes as e}
      <li>{e.number}: {#each e.langs as l}<button onclick={() => pick(e.number, l)}>{l}</button> {/each}</li>
    {:else}<li>No episodes</li>{/each}
  </ul>
  {#if selected}
    <h3>Streams for episode {selected.ep} ({selected.lang})</h3>
    <ul>
      {#each streams as s}<li>{s.name} <small>[{isAiring(s) ? "Airing" : "Normal"}]</small></li>{:else}<li>No streams (yet)</li>{/each}
    </ul>
  {/if}
{/if}
```

- [ ] **Step 4: `src/App.svelte`.**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./lib/api";
  import Login from "./lib/Login.svelte";
  import Search from "./lib/Search.svelte";
  import Info from "./lib/Info.svelte";
  let screen = $state<"boot" | "login" | "search" | "info">("boot"), id = $state(0);
  onMount(async () => { screen = (await api.autoLogin().catch(() => false)) ? "search" : "login"; });
  async function logout() { await api.logout(); screen = "login"; }
</script>

{#if screen === "login"}
  <Login onok={() => (screen = "search")} />
{:else if screen !== "boot"}
  <nav><button onclick={() => (screen = "search")}>Search</button> <button onclick={logout}>Log out</button></nav>
  {#if screen === "search"}
    <Search onopen={(i) => { id = i; screen = "info"; }} onauth={() => (screen = "login")} />
  {:else}
    <Info {id} onback={() => (screen = "search")} onauth={() => (screen = "login")} />
  {/if}
{/if}
```

- [ ] **Step 5: Verify and commit.**

Run: `npm run build && npm run check`
Expected: exit 0

Run: `npm run tauri dev`, then walk the acceptance list (login, restart, wrong password, search `naruto`, open a hit, log out).

```bash
git add -A && git commit -m "feat(ui): login, search, info screens"
```

---

## Self-review

- **Spec coverage:** probe (Task 0), crates + workspace (1), parsers (2, 3), client with auto re-login and 2-concurrency cap (4), commands and keychain (5), login/search/info UI and settings logout (6). Covered in Plans 2 and 3, not here: resolver, media proxy, player, Android/TV, D-pad navigation, Android Keystore. Cookie persistence is a recorded deviation.
- **Placeholders:** none; Task 3 Step 4 carries one conditional selector adjustment, decided by the acceptance test.
- **Type consistency:** `SearchHit`, `Info`, `Stream`, `Error`, `Client::{new,login,get,set_credentials,clear_credentials}` and the command names match across Tasks 2 to 6 and `api.ts`.
