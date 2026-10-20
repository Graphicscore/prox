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
