# Title actions, my list, next episode Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers-extended-cc:subagent-driven-development (recommended) or superpowers-extended-cc:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire Note/Favorite/Finished and per-episode Watchlist to Proxer, add a "My list" Home row, a Next-episode button with autoplay, and mark episodes watched when they end.

**Architecture:** Pure parsing/decision logic in `proxer-parse` (`mylist.rs`), a token-reading POST path in `proxer-client` + four Tauri commands, then Svelte wiring (helpers with node tests, Home row, Title actions, Player next/near-end).

**Tech Stack:** Rust (scraper, serde_json, reqwest, wiremock tests), Tauri v2 commands, Svelte 5 runes, TypeScript, `node --test`.

**Spec:** `docs/superpowers/specs/2026-10-20-title-actions-design.md`

## Global Constraints

- The `#proxerToken` value is a secret: never log it, never put it into an error string; reqwest errors on POST are stripped with `without_url()`. Accept only ASCII-alphanumeric token values.
- Write endpoints (exact): note/favor/finish `POST /info/<id>?format=json&json=<action>&<token>=1` body `checkPost=1`; reminder `POST /watch/<id>/<ep>/<lang>?format=json&type=reminder&<token>=1&title=reminder_<this|next>` body `check=1`; progress `POST /ucp?format=json&type=incState&id=<cid>&value=<n>&<token>=1` body `check=1`; state `POST /comment?id=<cid>&format=json&json=updateState&status=<0..3>&<token>=1` body `check=1`.
- List states: 0 Geschaut, 1 Am Schauen, 2 Wird noch geschaut, 3 Abgebrochen.
- Command inputs are allowlisted: action in {note, favor, finish}; which in {this, next}; lang non-empty `[a-z]+`. Anything else -> `"Unsupported"`.
- Error strings: existing codes plus `Proxer:<msg>` (Proxer refused, msg shown as-is) and `Parse:token`, `Parse:list`, `Parse:action`, `Parse:list entry`.
- Disabled buttons lose focus on TV: action buttons are never disabled while busy; guard with a `busy` flag instead.
- Existing behavior stays: Back stack, D-pad scopes, TV focus restore, stale-result guards, demo mode (`?demo`, DEV only, not in production bundle).
- `npm test`, `npm run check` (0 errors, 0 warnings), `npm run build`, `cargo test --workspace` green after every task.

**User decisions (already made):**
- "Watchlist this/next need to be connected to the episode UI and not the show overview."
- Home row: anime "marked to be watched but hasn't yet, starting with the ones the user is currently watching".
- Near the end of an episode a Next-episode button shows; when it ends the next one autoplays.
- An episode played to the end is marked watched; not-in-list titles are added as "Am Schauen"; Next pressed near the end counts as watched.
- "always test UI on your own, merge back, make choices on my behalf".

---

### Task 1: List parsing and watched-planning (`proxer-parse`)

**Goal:** `token`, `parse_my_list`, `action_result`, `plan_watched` with a synthetic fixture and tests.

**Files:**
- Create: `crates/proxer-parse/src/mylist.rs`
- Modify: `crates/proxer-parse/src/lib.rs`
- Create: `fixtures/ucp-anime.html`
- Test: `crates/proxer-parse/tests/mylist.rs`

**Acceptance Criteria:**
- [ ] `token` returns the `#proxerToken` value; `None` when missing or not ASCII-alphanumeric.
- [ ] `parse_my_list(fixture)` returns 4 entries (the one with `cid:"bad"` skipped), field values as in the test; strings containing `]`, `[` and `\"` do not break the array cut.
- [ ] Missing variable or truncated JSON -> `Err`; `[]` -> `Ok(vec![])`.
- [ ] `action_result` classifies `{"error":0}`, `{"error":1}`, `{"error":"1"}`, no `error` key, and non-JSON as in the test.
- [ ] `plan_watched` cases from the test pass.

**Verify:** `cargo test -p proxer-parse --test mylist` → all tests pass

**Steps:**

- [ ] **Step 1: Fixture** `fixtures/ucp-anime.html` (synthetic, same shape as the real page, no personal data):

```html
<!DOCTYPE html><html><head><title>UCP: Animeliste - Proxer.Me</title></head><body>
<div id="main"><input type="hidden" id="proxerToken" value="abc123DEF">
<table id="box-table-a"></table>
<script type="text/javascript">
var ucp_list_filters = {};
var ucp_list_items = [{"medium":"animeseries","episodes":"12","media_format":"tv","entry_state":"1","cid":"1001","name":"Tensei shitara Ken deshita 2","id":"59782","comment":"","comment_state":"2","episode":"0","rating":"0","sugoi":"0","timestamp":"1792500000"},{"medium":"animeseries","episodes":"12","media_format":"tv","entry_state":"1","cid":"1002","name":"Oshi ga [Budoukan] \"Itte\"","id":"60001","comment":"a ] tricky [ comment","comment_state":"1","episode":"5","rating":"0","sugoi":"0","timestamp":"1792400000"},{"medium":"movie","episodes":"1","media_format":"movie","entry_state":"1","cid":"1003","name":"Kill la Kill","id":"5000","comment":"","comment_state":"0","episode":"1","rating":"9","sugoi":"0","timestamp":"1500000000"},{"medium":"animeseries","episodes":"0","media_format":"tv","entry_state":"1","cid":"1004","name":"Airing Show","id":"61000","comment":"","comment_state":"1","episode":"3","rating":"0","sugoi":"0","timestamp":"1792450000"},{"medium":"ova","episodes":"2","media_format":"ova","entry_state":"1","cid":"bad","name":"Broken","id":"7","comment":"","comment_state":"3","episode":"0","rating":"0","sugoi":"0","timestamp":"1"}];
</script></div></body></html>
```

- [ ] **Step 2: Failing tests** `crates/proxer-parse/tests/mylist.rs`:

```rust
use proxer_parse::{action_result, parse_my_list, plan_watched, token, ListEntry, Step};

const FIX: &str = include_str!("../../../fixtures/ucp-anime.html");

fn entry(state: u8, episode: u32, episodes: u32) -> ListEntry {
    ListEntry { id: 1, cid: 9, name: "x".into(), episode, episodes, state, timestamp: 0, medium: "animeseries".into() }
}

#[test]
fn token_found_and_validated() {
    assert_eq!(token(FIX).as_deref(), Some("abc123DEF"));
    assert_eq!(token("<html></html>"), None);
    assert_eq!(token(r#"<input id="proxerToken" value="a&b=1">"#), None);
    assert_eq!(token(r#"<input id="proxerToken" value="">"#), None);
}

#[test]
fn my_list_from_fixture() {
    let l = parse_my_list(FIX).unwrap();
    assert_eq!(l.len(), 4);
    assert_eq!(l[0], ListEntry { id: 59782, cid: 1001, name: "Tensei shitara Ken deshita 2".into(), episode: 0, episodes: 12, state: 2, timestamp: 1792500000, medium: "animeseries".into() });
    assert_eq!(l[1].name, "Oshi ga [Budoukan] \"Itte\"");
    assert_eq!((l[1].state, l[1].episode), (1, 5));
    assert_eq!(l[3].episodes, 0);
    assert!(l.iter().all(|e| e.id != 7));
}

#[test]
fn my_list_errors_and_empty() {
    assert!(parse_my_list("<html></html>").is_err());
    assert!(parse_my_list(r#"<script>var ucp_list_items = [{"id":"1""#).is_err());
    assert_eq!(parse_my_list("<script>var ucp_list_items = [];</script>").unwrap(), vec![]);
}

#[test]
fn action_results() {
    assert_eq!(action_result(r#"{"error":0,"msg":"ok"}"#), Some(Ok("ok".into())));
    assert_eq!(action_result(r#"{"error":1,"msg":"nein"}"#), Some(Err("nein".into())));
    assert_eq!(action_result(r#"{"error":"1","msg":"x"}"#), Some(Err("x".into())));
    assert_eq!(action_result(r#"{"msg":"Lesezeichen gesetzt"}"#), Some(Ok("Lesezeichen gesetzt".into())));
    assert_eq!(action_result("<html>login</html>"), None);
}

#[test]
fn watched_plans() {
    // planned -> progress + Am Schauen
    assert_eq!(plan_watched(&entry(2, 0, 12), 1), vec![Step::Progress(1), Step::State(1)]);
    // dropped and watching again -> Am Schauen
    assert_eq!(plan_watched(&entry(3, 4, 12), 5), vec![Step::Progress(5), Step::State(1)]);
    // already watching, progress only
    assert_eq!(plan_watched(&entry(1, 3, 12), 4), vec![Step::Progress(4)]);
    // never lowers progress
    assert_eq!(plan_watched(&entry(1, 6, 12), 4), vec![]);
    // reaches the total -> Geschaut
    assert_eq!(plan_watched(&entry(1, 11, 12), 12), vec![Step::Progress(12), Step::State(0)]);
    // unknown total (airing) never completes
    assert_eq!(plan_watched(&entry(1, 3, 0), 4), vec![Step::Progress(4)]);
    // finished title rewatched: nothing
    assert_eq!(plan_watched(&entry(0, 12, 12), 3), vec![]);
}
```

- [ ] **Step 3: Run** `cargo test -p proxer-parse --test mylist` → FAIL (unresolved imports).

- [ ] **Step 4: Implement** `crates/proxer-parse/src/mylist.rs`:

```rust
use scraper::{Html, Selector};
use serde::Serialize;
use serde_json::Value;

/// Value of the hidden `#proxerToken` input. Proxer uses it as a query parameter NAME on every write.
/// Secret: never log it. Only ASCII-alphanumeric values are accepted (it goes into a URL).
pub fn token(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("#proxerToken").expect("selector");
    let v = doc.select(&sel).next()?.value().attr("value")?;
    (!v.is_empty() && v.chars().all(|c| c.is_ascii_alphanumeric())).then(|| v.to_string())
}

/// One row of the user's anime list (`/ucp?s=anime`). `state`: 0 Geschaut, 1 Am Schauen, 2 Wird noch geschaut, 3 Abgebrochen.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ListEntry {
    pub id: u32,
    pub cid: u64,
    pub name: String,
    pub episode: u32,
    pub episodes: u32, // 0 = unknown (airing)
    pub state: u8,
    pub timestamp: u64,
    pub medium: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ListError {
    #[error("no list variable on page")]
    Missing,
    #[error("list JSON invalid")]
    BadJson,
}

/// The `[...]` at the start of `s`, matched by brackets outside JSON strings.
fn json_array(s: &str) -> Option<&str> {
    let (mut depth, mut in_str, mut esc) = (0i32, false, false);
    for (i, c) in s.bytes().enumerate() {
        if in_str {
            if esc { esc = false } else if c == b'\\' { esc = true } else if c == b'"' { in_str = false }
            continue;
        }
        match c {
            b'"' => in_str = true,
            b'[' => depth += 1,
            b']' => { depth -= 1; if depth == 0 { return Some(&s[..=i]); } }
            _ => {}
        }
    }
    None
}

/// Proxer sends numbers as strings; accept both.
fn field<T: std::str::FromStr>(v: &Value, k: &str) -> Option<T> {
    match v.get(k)? {
        Value::String(s) => s.trim().parse().ok(),
        n @ Value::Number(_) => n.to_string().parse().ok(),
        _ => None,
    }
}

/// Parse `var ucp_list_items = [...]` from the list page. Entries with unusable id/cid/state are skipped.
pub fn parse_my_list(html: &str) -> Result<Vec<ListEntry>, ListError> {
    let at = html.find("var ucp_list_items").ok_or(ListError::Missing)?;
    let rest = &html[at..];
    let open = rest.find('[').ok_or(ListError::Missing)?;
    let arr = json_array(&rest[open..]).ok_or(ListError::BadJson)?;
    let items: Vec<Value> = serde_json::from_str(arr).map_err(|_| ListError::BadJson)?;
    Ok(items
        .iter()
        .filter_map(|v| {
            Some(ListEntry {
                id: field(v, "id")?,
                cid: field(v, "cid")?,
                name: v.get("name")?.as_str()?.to_string(),
                episode: field(v, "episode").unwrap_or(0),
                episodes: field(v, "episodes").unwrap_or(0),
                state: field(v, "comment_state")?,
                timestamp: field(v, "timestamp").unwrap_or(0),
                medium: v.get("medium").and_then(Value::as_str).unwrap_or("").to_string(),
            })
        })
        .collect())
}

/// A write response: `Some(Ok(msg))`, `Some(Err(msg))` when Proxer refused (`error` != 0), `None` if not JSON
/// (e.g. a login page after a lost session).
pub fn action_result(body: &str) -> Option<Result<String, String>> {
    let v: Value = serde_json::from_str(body.trim()).ok()?;
    let msg = v.get("msg").and_then(Value::as_str).unwrap_or("").to_string();
    let failed = match v.get("error") {
        Some(Value::Number(n)) => n.as_i64() != Some(0),
        Some(Value::String(s)) => s.trim() != "0",
        Some(Value::Bool(b)) => *b,
        _ => false,
    };
    Some(if failed { Err(msg) } else { Ok(msg) })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// set list progress to n (Proxer's `incState` sets an absolute value)
    Progress(u32),
    /// set list state (0..3)
    State(u8),
}

/// What to write when `episode` was watched: never lowers progress; Geschaut when the known total is reached,
/// else Wird noch geschaut / Abgebrochen become Am Schauen.
pub fn plan_watched(e: &ListEntry, episode: u32) -> Vec<Step> {
    let mut steps = Vec::new();
    if episode > e.episode {
        steps.push(Step::Progress(episode));
    }
    let progress = e.episode.max(episode);
    if e.episodes > 0 && progress >= e.episodes {
        if e.state != 0 {
            steps.push(Step::State(0));
        }
    } else if (e.state == 2 || e.state == 3) && !steps.is_empty() {
        steps.push(Step::State(1));
    }
    steps
}
```

Note the `!steps.is_empty()` guard: a planned title whose progress does not change stays planned (rewatching episode 0 is impossible, so this only matters for robustness).

In `crates/proxer-parse/src/lib.rs` add `mod mylist;` and
`pub use mylist::{action_result, parse_my_list, plan_watched, token, ListEntry, ListError, Step};`

- [ ] **Step 5: Run** `cargo test -p proxer-parse` → all pass (old and new).

- [ ] **Step 6: Commit**

```bash
git add crates/proxer-parse fixtures/ucp-anime.html
git commit -m "feat(parse): my anime list, proxerToken, action results, watched planning"
```

```json:metadata
{"files": ["crates/proxer-parse/src/mylist.rs", "crates/proxer-parse/src/lib.rs", "fixtures/ucp-anime.html", "crates/proxer-parse/tests/mylist.rs"], "verifyCommand": "cargo test -p proxer-parse", "acceptanceCriteria": ["token validated", "parse_my_list fixture 4 entries", "errors and empty list", "action_result cases", "plan_watched cases"], "modelTier": "mechanical"}
```

---

### Task 2: Client POST and Tauri commands

**Goal:** `Client::post` plus commands `my_list`, `title_action`, `reminder`, `mark_watched`, registered in the invoke handler.

**Files:**
- Modify: `crates/proxer-client/src/lib.rs` (add `post`)
- Test: `crates/proxer-client/tests/client.rs` (add one test)
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs` (register commands)

**Acceptance Criteria:**
- [ ] `post` sends a form through the semaphore, maps 5xx/429 to `Http`, strips the URL from network errors; wiremock test checks method, path, query and body.
- [ ] Commands use the exact endpoints from Global Constraints; inputs outside the allowlists return `"Unsupported"` without any request.
- [ ] `mark_watched` adds a missing title via `note`, re-reads the list, then executes `plan_watched` steps in order; a missing entry after `note` returns `Parse:list entry`.
- [ ] Unit tests for `action_kind`, `reminder_title`, `valid_lang`.

**Verify:** `cargo test --workspace` → all pass

**Steps:**

- [ ] **Step 1: Failing client test** (append to `crates/proxer-client/tests/client.rs`; add `query_param` and `body_string` to the `wiremock::matchers` import):

```rust
#[tokio::test]
async fn post_sends_form_and_maps_status() {
    let s = MockServer::start().await;
    Mock::given(method("POST")).and(path("/ucp")).and(query_param("type", "incState")).and(body_string("check=1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"error":0,"msg":"ok"}"#)).mount(&s).await;
    Mock::given(method("POST")).and(path("/busy")).respond_with(ResponseTemplate::new(503)).mount(&s).await;
    let c = Client::new(&s.uri());
    assert_eq!(c.post("/ucp?format=json&type=incState&id=1&value=2", &[("check", "1")]).await.unwrap(), r#"{"error":0,"msg":"ok"}"#);
    assert!(matches!(c.post("/busy", &[("check", "1")]).await, Err(Error::Http(503))));
}
```

- [ ] **Step 2: Implement `post`** in `impl Client` (after `get`):

```rust
    /// POST a form to one of Proxer's JSON write endpoints. The URL carries the session token, so errors are
    /// stripped of it. No re-login here: callers GET the token page first (which re-logs in if needed).
    pub async fn post(&self, path: &str, form: &[(&str, &str)]) -> Result<String, Error> {
        let _p = self.gate.acquire().await.expect("semaphore");
        let res = self.http().post(format!("{}{}", self.base, path)).form(form).send().await.map_err(|e| e.without_url())?;
        check_status(res.status())?;
        Ok(res.text().await.map_err(|e| e.without_url())?)
    }
```

Run `cargo test -p proxer-client` → pass.

- [ ] **Step 3: Commands** in `src-tauri/src/commands.rs`. Extend the `proxer_parse` import with `action_result, parse_my_list, plan_watched, token, ListEntry, Step`. Add:

```rust
fn action_kind(a: &str) -> Option<&'static str> {
    match a { "note" => Some("note"), "favor" => Some("favor"), "finish" => Some("finish"), _ => None }
}

fn reminder_title(w: &str) -> Option<&'static str> {
    match w { "this" => Some("reminder_this"), "next" => Some("reminder_next"), _ => None }
}

fn valid_lang(l: &str) -> bool {
    !l.is_empty() && l.bytes().all(|b| b.is_ascii_lowercase())
}

/// POST and read Proxer's JSON answer; `{error:1,msg}` becomes `Proxer:<msg>`.
async fn post_action(app: &App, path: &str, form: &[(&str, &str)]) -> Result<String, String> {
    let body = app.client.post(path, form).await.map_err(err)?;
    match action_result(&body) {
        Some(Ok(m)) => Ok(m),
        Some(Err(m)) => Err(format!("Proxer:{m}")),
        None => Err("Parse:action".into()),
    }
}

/// GET `page` for a fresh token, then POST to `target(token)`.
async fn write(app: &App, page: &str, target: impl Fn(&str) -> String, form: &[(&str, &str)]) -> Result<String, String> {
    let html = app.client.get(page).await.map_err(err)?;
    let tok = token(&html).ok_or_else(|| "Parse:token".to_string())?;
    post_action(app, &target(&tok), form).await
}

async fn load_entry(app: &App, id: u32) -> Result<(String, Option<ListEntry>), String> {
    let html = app.client.get("/ucp?s=anime").await.map_err(err)?;
    let e = parse_my_list(&html).map_err(|_| "Parse:list".to_string())?.into_iter().find(|e| e.id == id);
    Ok((html, e))
}

#[tauri::command]
pub async fn my_list(app: State<'_, App>) -> Result<Vec<ListEntry>, String> {
    let html = app.client.get("/ucp?s=anime").await.map_err(err)?;
    parse_my_list(&html).map_err(|_| "Parse:list".into())
}

#[tauri::command]
pub async fn title_action(app: State<'_, App>, id: u32, action: String) -> Result<String, String> {
    let a = action_kind(&action).ok_or_else(|| "Unsupported".to_string())?;
    write(&app, &format!("/info/{id}"), |t| format!("/info/{id}?format=json&json={a}&{t}=1"), &[("checkPost", "1")]).await
}

#[tauri::command]
pub async fn reminder(app: State<'_, App>, id: u32, episode: u32, lang: String, which: String) -> Result<String, String> {
    let title = reminder_title(&which).ok_or_else(|| "Unsupported".to_string())?;
    if !valid_lang(&lang) {
        return Err("Unsupported".into());
    }
    let page = format!("/watch/{id}/{episode}/{lang}");
    write(&app, &page, |t| format!("{page}?format=json&type=reminder&{t}=1&title={title}"), &[("check", "1")]).await
}

/// Episode watched to the end: add the title if missing (Vermerken), then progress/state per `plan_watched`.
#[tauri::command]
pub async fn mark_watched(app: State<'_, App>, id: u32, episode: u32) -> Result<(), String> {
    let (mut html, mut entry) = load_entry(&app, id).await?;
    if entry.is_none() {
        write(&app, &format!("/info/{id}"), |t| format!("/info/{id}?format=json&json=note&{t}=1"), &[("checkPost", "1")]).await?;
        (html, entry) = load_entry(&app, id).await?;
    }
    let entry = entry.ok_or_else(|| "Parse:list entry".to_string())?;
    let tok = token(&html).ok_or_else(|| "Parse:token".to_string())?;
    for step in plan_watched(&entry, episode) {
        let path = match step {
            Step::Progress(n) => format!("/ucp?format=json&type=incState&id={}&value={n}&{tok}=1", entry.cid),
            Step::State(s) => format!("/comment?id={}&format=json&json=updateState&status={s}&{tok}=1", entry.cid),
        };
        post_action(&app, &path, &[("check", "1")]).await?;
    }
    Ok(())
}
```

Add to the `tests` module:

```rust
    #[test]
    fn write_allowlists() {
        assert_eq!(action_kind("note"), Some("note"));
        assert_eq!(action_kind("favor"), Some("favor"));
        assert_eq!(action_kind("finish"), Some("finish"));
        assert_eq!(action_kind("delete"), None);
        assert_eq!(reminder_title("this"), Some("reminder_this"));
        assert_eq!(reminder_title("next"), Some("reminder_next"));
        assert_eq!(reminder_title("x"), None);
        assert!(valid_lang("engsub"));
        assert!(!valid_lang(""));
        assert!(!valid_lang("eng&x=1"));
        assert!(!valid_lang("EngSub"));
    }
```

- [ ] **Step 4: Register** in `src-tauri/src/lib.rs` `generate_handler!`: append `commands::my_list, commands::title_action, commands::reminder, commands::mark_watched`.

- [ ] **Step 5: Run** `cargo test --workspace` → pass; `cargo build -p proxer-app 2>&1 | tail -3` (or the src-tauri crate name from its Cargo.toml) → builds without warnings from the new code.

- [ ] **Step 6: Commit**

```bash
git add crates/proxer-client src-tauri/src
git commit -m "feat: Proxer write commands (title actions, reminder, my list, mark watched)"
```

```json:metadata
{"files": ["crates/proxer-client/src/lib.rs", "crates/proxer-client/tests/client.rs", "src-tauri/src/commands.rs", "src-tauri/src/lib.rs"], "verifyCommand": "cargo test --workspace", "acceptanceCriteria": ["post via semaphore, status mapping, url stripped", "exact endpoints", "allowlists", "mark_watched note+reload+steps"], "modelTier": "standard"}
```

---

### Task 3: Frontend API, demo fakes and helpers

**Goal:** TS API for the four commands, demo fakes, `describeError` for `Proxer:`, helpers `myListHits`, `nextEpisode`, `nearEnd` with node tests.

**Files:**
- Modify: `src/lib/api.ts`
- Modify: `src/lib/demo.ts`
- Modify: `src/lib/ui/helpers.ts`
- Test: `src/lib/ui/helpers.test.ts`

**Acceptance Criteria:**
- [ ] `api.myList/titleAction/reminder/markWatched` invoke `my_list/title_action/reminder/mark_watched` with the argument names of Task 2.
- [ ] `describeError("Proxer:Schon vorhanden")` returns `"Schon vorhanden"`; `"Proxer:"` returns `"Proxer refused the action"`.
- [ ] Helper tests below pass; `npm run check` 0/0.

**Verify:** `npm test` → all pass

**Steps:**

- [ ] **Step 1: Failing tests** appended to `src/lib/ui/helpers.test.ts` (extend the import with `myListHits, nearEnd, nextEpisode`):

```ts
const L = (id: number, state: number, timestamp: number, episode = 0, episodes = 12) =>
  ({ id, cid: id, name: "t" + id, episode, episodes, state, timestamp, medium: "animeseries" });

test("myListHits: watching first, then planned, each newest first; others dropped", () => {
  const hits = myListHits([L(1, 2, 10), L(2, 1, 5, 3), L(3, 0, 99), L(4, 1, 7, 1, 0), L(5, 2, 20), L(6, 3, 50)]);
  assert.deepEqual(hits.map((h) => h.id), [4, 2, 5, 1]);
  assert.equal(hits[0].badge, "1/?");
  assert.equal(hits[1].badge, "3/12");
  assert.equal(hits[1].title, "t2");
});

const pages = [{ page: 1, label: "1 - 50" }, { page: 2, label: "51 - 100" }];

test("nextEpisode: same page, page boundary, last episode", () => {
  assert.deepEqual(nextEpisode(eps, "engsub", 1, pages, 1), { ep: 2 });
  assert.deepEqual(nextEpisode(eps, "gersub", 2, pages, 1), { page: 2 });
  assert.equal(nextEpisode(eps, "gersub", 2, pages, 2), null);
  assert.equal(nextEpisode(eps, "engsub", 3, [], 1), null);
});

test("nearEnd: last 90 s, or last 10 % for videos under 15 min", () => {
  assert.equal(nearEnd(1400, 1440), true);
  assert.equal(nearEnd(1300, 1440), false);
  assert.equal(nearEnd(540, 600), true);
  assert.equal(nearEnd(530, 600), false);
  assert.equal(nearEnd(10, NaN), false);
  assert.equal(nearEnd(0, 0), false);
});
```

(`eps` is the fixture already defined at the top of the file: episode 1 engsub/engdub/xxsub, 2 gersub/engsub, 3 engsub.)

- [ ] **Step 2: Run** `npm test` → FAIL (missing exports).

- [ ] **Step 3: `api.ts`**: add `badge?: string` to `SearchHit`; add types and calls:

```ts
export type ListEntry = { id: number; cid: number; name: string; episode: number; episodes: number; state: number; timestamp: number; medium: string };
export type TitleAction = "note" | "favor" | "finish";
```

In `realApi`:

```ts
  myList: () => invoke<ListEntry[]>("my_list"),
  titleAction: (id: number, action: TitleAction) => invoke<string>("title_action", { id, action }),
  reminder: (id: number, episode: number, lang: string, which: "this" | "next") => invoke<string>("reminder", { id, episode, lang, which }),
  markWatched: (id: number, episode: number) => invoke<void>("mark_watched", { id, episode }),
```

In `describeError`, before the final return:

```ts
  if (c.startsWith("Proxer:")) return c.slice(7) || "Proxer refused the action";
```

- [ ] **Step 4: `helpers.ts`** (import types `ListEntry, PageLink, SearchHit` from `../api.ts` alongside `Episode`):

```ts
/** Home "My list": Am Schauen (1) first, then Wird noch geschaut (2), each by last change, newest first. */
export function myListHits(entries: ListEntry[]): SearchHit[] {
  const by = (s: number) => entries.filter((e) => e.state === s).sort((a, b) => b.timestamp - a.timestamp);
  return [...by(1), ...by(2)].map((e) => ({ id: e.id, title: e.name, genres: [], kind: e.medium, rating: null, badge: `${e.episode}/${e.episodes || "?"}` }));
}

/** The episode after `ep` in `lang`: on this page, else the next page (to load first), else null. */
export function nextEpisode(episodes: Episode[], lang: string, ep: number, pages: PageLink[], epPage: number): { ep: number } | { page: number } | null {
  const n = episodesWith(episodes, lang).find((x) => x > ep);
  if (n !== undefined) return { ep: n };
  return pages.some((p) => p.page === epPage + 1) ? { page: epPage + 1 } : null;
}

/** Near-end window for the Next-episode pop-up: last 90 s, or the last 10 % of videos shorter than 15 min. */
export function nearEnd(t: number, duration: number): boolean {
  if (!Number.isFinite(duration) || duration <= 0) return false;
  return duration - t <= (duration < 900 ? duration * 0.1 : 90);
}
```

- [ ] **Step 5: `demo.ts`** fakes (import `ListEntry` type). Inside `makeDemoApi`, before `return`:

```ts
  const mine: ListEntry[] = [
    { id: 59782, cid: 1, name: "Tensei shitara Ken deshita 2", episode: 3, episodes: 12, state: 1, timestamp: 300, medium: "animeseries" },
    { id: 1874, cid: 2, name: "Steins;Gate", episode: 0, episodes: 24, state: 2, timestamp: 200, medium: "animeseries" },
    { id: 45, cid: 3, name: "Naruto", episode: 220, episodes: 220, state: 0, timestamp: 100, medium: "animeseries" },
  ];
```

and in the returned object:

```ts
    myList: async () => mine.map((e) => ({ ...e })),
    titleAction: async (_id: number, action: "note" | "favor" | "finish") =>
      ({ note: "Der Eintrag wurde vermerkt.", favor: "Der Eintrag wurde favorisiert.", finish: "Der Eintrag wurde als abgeschlossen markiert." })[action],
    reminder: async (_id: number, ep: number, _lang: string, which: "this" | "next") => `Lesezeichen für Episode ${which === "next" ? ep + 1 : ep} gesetzt.`,
    markWatched: async (id: number, ep: number) => {
      const e = mine.find((x) => x.id === id);
      if (e) { e.episode = Math.max(e.episode, ep); if (e.state === 2) e.state = 1; }
    },
```

- [ ] **Step 6: Run** `npm test` and `npm run check` → pass, 0 errors 0 warnings.

- [ ] **Step 7: Commit**

```bash
git add src/lib/api.ts src/lib/demo.ts src/lib/ui/helpers.ts src/lib/ui/helpers.test.ts
git commit -m "feat(web): API for list/actions, demo fakes, my-list/next-episode helpers"
```

```json:metadata
{"files": ["src/lib/api.ts", "src/lib/demo.ts", "src/lib/ui/helpers.ts", "src/lib/ui/helpers.test.ts"], "verifyCommand": "npm test && npm run check", "acceptanceCriteria": ["api calls", "describeError Proxer:", "helper tests"], "modelTier": "mechanical"}
```

---

### Task 4: My list row, title actions, watchlist buttons

**Goal:** Home "My list" row first; Title hero Note/Favorite/Finished wired with list-state chip and message line; EpisodePicker Watchlist buttons wired.

**Files:**
- Modify: `src/lib/Home.svelte`
- Modify: `src/lib/ui/PosterRow.svelte`, `src/lib/ui/Poster.svelte`
- Modify: `src/lib/Title.svelte`
- Modify: `src/lib/ui/EpisodePicker.svelte`

**Acceptance Criteria:**
- [ ] Home rows in order: My list, Airing now, Top rated, Most viewed; My list badge shows `episode/total`; the hero is still the first Airing hit; empty My list hides.
- [ ] Title meta ends with e.g. `Am Schauen · 3/12` when the title is in the list.
- [ ] Note shows `aria-pressed="true"` for state 2, Finished for state 0; after an action Proxer's message shows in a `role="status"` line, errors in the existing alert; the list state reloads after success.
- [ ] Watchlist buttons call `api.reminder(id, selected.ep, selected.lang, "this"|"next")` and show the message the same way.
- [ ] No action button is ever `disabled` while busy (TV focus); a second click while busy is ignored.
- [ ] Demo screenshot `?demo&screen=title&id=59782&pick=1` shows the chip and enabled buttons.

**Verify:** `npm run check` → 0 errors 0 warnings; `npm test` → pass

**Steps:**

- [ ] **Step 1: Poster badge.** `Poster.svelte`: type `rating?: number | string | null` (it is only displayed). `PosterRow.svelte`: pass `rating={h.badge ?? h.rating}`.

- [ ] **Step 2: Home.** In `Home.svelte`: `type Row = { kind: ListKind | "mine"; ... }`; prepend `{ kind: "mine", title: "My list", status: "loading", hits: [], error: "" }`; hero becomes `rows.find((r) => r.kind === "airing")?.hits[0] ?? null`; in `load`:

```ts
    try { r.hits = r.kind === "mine" ? myListHits(await api.myList()) : (await api.getList(r.kind)).hits; r.status = "ready"; }
```

(import `myListHits` from `./ui/helpers`).

- [ ] **Step 3: Title.** Add state and helpers (import `type ListEntry`, `type TitleAction` from `./api`):

```ts
  const STATES = ["Geschaut", "Am Schauen", "Wird noch geschaut", "Abgebrochen"];
  let entry = $state<ListEntry | null>(null), message = $state(""), busy = false;
  async function loadEntry() {
    const cur = id;
    try { const l = await api.myList(); if (cur === id) entry = l.find((e) => e.id === cur) ?? null; }
    catch (x) { if (x === "NotLoggedIn") onauth(); } // the chip is optional: the page stays usable without it
  }
  /** Run a write; show Proxer's message, refresh the list state. Buttons stay enabled (TV focus), so guard re-entry. */
  async function act(run: () => Promise<string>) {
    if (busy) return;
    busy = true; message = ""; error = "";
    try { message = (await run()) || "Done"; await loadEntry(); } catch (x) { fail(x); }
    busy = false;
  }
  const titleAct = (a: TitleAction) => act(() => api.titleAction(id, a));
```

In the `$effect` after `await loadPage(1);` add `loadEntry();` (not awaited). Reset `entry = null; message = "";` at the start of the effect (when `id` changes).

`meta` becomes:

```ts
  const meta = $derived([info?.fields["Status"], info?.fields["Genre"], entry && `${STATES[entry.state] ?? ""} · ${entry.episode}/${entry.episodes || "?"}`].filter(Boolean).join(" · "));
```

Replace the disabled `{#each ["Note", "Favorite", "Finished"] ...}` with:

```svelte
    <button aria-pressed={entry?.state === 2} onclick={() => titleAct("note")}>Note</button>
    <button onclick={() => titleAct("favor")}>Favorite</button>
    <button aria-pressed={entry?.state === 0} onclick={() => titleAct("finish")}>Finished</button>
```

After `</Hero>`: `{#if message}<p role="status" class="msg">{message}</p>{/if}`. Pass `onreminder={(w) => { const s = selected; if (s) act(() => api.reminder(id, s.ep, s.lang, w)); }}` to `EpisodePicker`. Styles:

```css
  .msg { padding: 8px var(--pad) 0; color: var(--muted); }
  button[aria-pressed="true"] { background: var(--good); color: #fff; }
```

- [ ] **Step 4: EpisodePicker.** Add prop `onreminder: (which: "this" | "next") => void`; the two Watchlist buttons lose `disabled`/`title` and get `onclick={() => onreminder("this")}` / `onclick={() => onreminder("next")}`.

- [ ] **Step 5: Check** `npm run check`, `npm test`, `npm run build`; demo screenshots (dev server `npx vite --port 1420 --strictPort`; kill it afterwards with `pkill -f "[n]ode.*vite"`):
  `google-chrome-stable --headless=new --disable-gpu --hide-scrollbars --window-size=1280,900 --virtual-time-budget=6000 --screenshot=<scratch>/t4-title.png "http://localhost:1420/?demo&screen=title&id=59782&pick=1"` and `...?demo&screen=home` → the chip "Am Schauen · 3/12" and the My list row are visible.

- [ ] **Step 6: Commit**

```bash
git add src/lib
git commit -m "feat(ui): My list row, title actions and watchlist buttons wired"
```

```json:metadata
{"files": ["src/lib/Home.svelte", "src/lib/ui/PosterRow.svelte", "src/lib/ui/Poster.svelte", "src/lib/Title.svelte", "src/lib/ui/EpisodePicker.svelte"], "verifyCommand": "npm run check && npm test && npm run build", "acceptanceCriteria": ["row order and badge", "meta chip", "pressed states and message line", "reminder wiring", "no disabled while busy"], "modelTier": "standard"}
```

---

### Task 5: Next episode, autoplay, mark watched

**Goal:** Player shows a toolbar Next button and a near-end pop-up, autoplays the next episode on `ended` inside the same Player instance (fullscreen survives), and reports watched episodes.

**Files:**
- Modify: `src/lib/Player.svelte`
- Modify: `src/lib/Title.svelte`

**Acceptance Criteria:**
- [ ] Player props `hasNext = false`, `onnext = () => {}`, `onwatched = () => {}`; toolbar "Next episode" when `hasNext`; pop-up "▶ Next episode" over the video when `nearEnd(currentTime, duration)` and `hasNext` (also in fullscreen); it takes focus only if the video had focus.
- [ ] `ended` -> `onwatched()` then, if `hasNext`, `onnext()`. Next pressed while near the end -> `onwatched()` first. Embedded iframe: toolbar button only, never `onwatched`.
- [ ] Changing the `stream` prop reloads the source in place (no remount): state reset, stale resolve results dropped, fullscreen kept.
- [ ] Title: `{#key}` uses a counter bumped only by user plays (stream button, Watch); `playNext()` loads the next page if needed, plays the first playable stream of the next episode in the same language, updates `selected`/`streams`; `onwatched` calls `api.markWatched(id, ep)` then reloads the list chip; failures show via `fail`.

**Verify:** `npm run check` → 0/0; `npm test`, `npm run build` → pass

**Steps:**

- [ ] **Step 1: Player.** Imports: add `tick, untrack` from `svelte`, `nearEnd` from `./ui/helpers`. Props:

```ts
  let { stream, onback, onauth, hasNext = false, onnext = () => {}, onwatched = () => {} }: {
    stream: Stream; onback: () => void; onauth: () => void; hasNext?: boolean; onnext?: () => void; onwatched?: () => void;
  } = $props();
  let near = $state(false), nextBtn = $state<HTMLButtonElement | null>(null);
```

`resolve()` drops stale results:

```ts
  async function resolve() {
    error = "";
    const s = stream;
    try {
      const m = await api.resolve(s);
      if (alive && s === stream) src = m;
    } catch (x) {
      if (!alive || s !== stream) return;
      if (x === "NotLoggedIn") onauth(); else error = describeError(x);
    }
  }
```

Remove `resolve()` from `onMount` and load per stream instead:

```ts
  let shown: Stream | null = null;
  $effect(() => { // first mount and every Next: same instance, so fullscreen and focus scope survive
    if (stream === shown) return;
    shown = stream;
    untrack(() => { src = null; error = ""; frame = false; retried = false; focusedVideo = false; at = 0; near = false; resolve(); });
  });
  function ontime() {
    if (!video) return;
    const n = nearEnd(video.currentTime, video.duration);
    if (n && !near && document.activeElement === video) tick().then(() => nextBtn?.focus());
    near = n;
  }
  function goNext() { if (near) onwatched(); onnext(); }
  function onended() { onwatched(); if (hasNext) onnext(); }
```

`<video ...>` gets `ontimeupdate={ontime} {onended}`. Toolbar (inside `.bar`, after Fullscreen): `{#if hasNext}<button onclick={goNext}>Next episode</button>{/if}`. After the video/iframe block:

```svelte
  {#if near && hasNext && src && !frame}<button class="next-pop primary" bind:this={nextBtn} onclick={goNext}>▶ Next episode</button>{/if}
```

CSS: `section { position: relative; }` and `.next-pop { position: absolute; right: 1.5rem; bottom: 4.5rem; z-index: 2; }`.

- [ ] **Step 2: Title.** Replace `{#key playing}` by `{#key playKey}` with `let playKey = $state(0);` bumped where a user starts playback: in `watch()` (`playing = s; playKey++`) and the picker's `onplay={(s) => { playing = s; playKey++; }}`. Add (import `nextEpisode` from `./ui/helpers`):

```ts
  const hasNext = $derived(!!selected && nextEpisode(episodes, selected.lang, selected.ep, pages, epPage) !== null);
  /** Next episode in the same language, first playable stream; keeps the Player instance (fullscreen stays). */
  async function playNext() {
    const cur = selected;
    if (!cur || !playing) return;
    let n = nextEpisode(episodes, cur.lang, cur.ep, pages, epPage);
    if (n && "page" in n) { await loadPage(n.page); n = nextEpisode(episodes, cur.lang, 0, pages, epPage); }
    if (!n || !("ep" in n) || selected !== cur) return;
    const ep = n.ep;
    try {
      const r = await api.streams(id, ep, cur.lang);
      if (selected !== cur) return; // user picked something else meanwhile
      const s = r.find((x) => streamLabel(x) !== "Unsupported");
      if (!s) { error = "No playable stream for episode " + ep; return; }
      selected = { ep, lang: cur.lang }; streams = r; playing = s;
    } catch (x) { fail(x); }
  }
  function watched() {
    const ep = selected?.ep;
    if (ep) api.markWatched(id, ep).then(loadEntry, fail);
  }
```

Player usage: `<Player stream={playing} onback={() => (playing = null)} {onauth} {hasNext} onnext={playNext} onwatched={watched} />`.

Note: after `loadPage(n.page)` the tiles show the new page while the old episode keeps playing until the new stream starts; acceptable.

- [ ] **Step 3: Check** `npm run check` (0/0), `npm test`, `npm run build`.

- [ ] **Step 4: Commit**

```bash
git add src/lib/Player.svelte src/lib/Title.svelte
git commit -m "feat(player): next episode button, near-end pop-up, autoplay and mark watched"
```

```json:metadata
{"files": ["src/lib/Player.svelte", "src/lib/Title.svelte"], "verifyCommand": "npm run check && npm test && npm run build", "acceptanceCriteria": ["props and buttons", "ended/next watched semantics", "in-place stream switch", "playNext and watched in Title"], "modelTier": "standard"}
```

---

## After the tasks (controller, not a task)
Live verification on the TV emulator by the controller (user decision: agent tests the UI): My list row vs. the account; each action shows Proxer's message; seek a video to its last seconds via WebView DevTools -> autoplay of the next episode and list progress/state changed; Next button near the end; fullscreen survives autoplay. Revert every test write on the account afterwards. Then merge to `master`.
