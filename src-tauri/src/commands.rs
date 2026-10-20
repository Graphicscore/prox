use crate::creds::CredStore;
use proxer_client::{Client, Error};
use proxer_parse::{action_result, parse_my_list, plan_watched, token, ListEntry, Step, parse_episode_list, parse_info, parse_search_page, parse_embed, parse_streams, EpisodeList, Info, MediaSource, SearchPage, Stream, StreamKind};
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
        Error::Http(n) => format!("Http:{n}"),
        Error::Network(_) => "Network".into(),
    }
}

#[tauri::command]
pub async fn login(app: State<'_, App>, username: String, password: String, remember: bool) -> Result<bool, String> {
    app.client.login(&username, &password).await.map_err(err)?;
    if !remember {
        app.store.clear(); // an older saved account must not auto-login next start
    }
    // true = credentials persisted (or not requested); false = remember requested but keychain write failed
    Ok(!remember || app.store.save(&username, &password).is_ok())
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
    app.client.logout().await;
    Ok(())
}

#[tauri::command]
pub async fn search(app: State<'_, App>, query: String, page: Option<u32>) -> Result<SearchPage, String> {
    let q: String = url_encode(&query);
    let p = page.unwrap_or(1).max(1); // the site counts pages from 1
    let html = app.client.get(&format!("/search?s=search&name={q}&typ=all-anime&p={p}")).await.map_err(err)?;
    Ok(parse_search_page(&html))
}

/// Home rows: kind -> the Proxer list page (same `tr.entry` table as search).
fn list_path(kind: &str) -> Option<&'static str> {
    match kind {
        "airing" => Some("/anime/airing"),
        "rating" => Some("/anime/animeseries/rating/all"),
        "clicks" => Some("/anime/animeseries/clicks/all"),
        _ => None,
    }
}

#[tauri::command]
pub async fn get_list(app: State<'_, App>, kind: String) -> Result<SearchPage, String> {
    let path = list_path(&kind).ok_or_else(|| "Unsupported".to_string())?;
    let html = app.client.get(path).await.map_err(err)?;
    Ok(parse_search_page(&html))
}

#[tauri::command]
pub async fn get_info(app: State<'_, App>, id: u32) -> Result<Info, String> {
    Ok(parse_info(&app.client.get(&format!("/info/{id}")).await.map_err(err)?))
}

#[tauri::command]
pub async fn get_episodes(app: State<'_, App>, id: u32, page: Option<u32>) -> Result<EpisodeList, String> {
    let p = page.unwrap_or(1).max(1); // 50 episodes per page
    Ok(parse_episode_list(&app.client.get(&format!("/info/{id}/list/{p}")).await.map_err(err)?, id))
}

#[tauri::command]
pub async fn get_streams(app: State<'_, App>, id: u32, episode: u32, lang: String) -> Result<Vec<Stream>, String> {
    let lang: String = lang.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let html = app.client.get(&format!("/watch/{id}/{episode}/{lang}")).await.map_err(err)?;
    parse_streams(&html).map_err(|_| "Parse:streams".into())
}

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

fn url_encode(s: &str) -> String {
    s.bytes().map(|b| match b {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
        _ => format!("%{b:02X}"),
    }).collect()
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_encode_cases() {
        assert_eq!(url_encode("a b&c#d"), "a%20b%26c%23d");
        assert_eq!(url_encode("é"), "%C3%A9");
        assert_eq!(url_encode("外"), "%E5%A4%96");
        assert_eq!(url_encode("Az09-_."), "Az09-_.");
    }

    #[test]
    fn list_paths() {
        assert_eq!(list_path("airing"), Some("/anime/airing"));
        assert_eq!(list_path("rating"), Some("/anime/animeseries/rating/all"));
        assert_eq!(list_path("clicks"), Some("/anime/animeseries/clicks/all"));
        assert_eq!(list_path("../etc"), None);
        assert_eq!(list_path(""), None);
    }

    #[test]
    fn err_codes() {
        assert_eq!(err(Error::NotLoggedIn), "NotLoggedIn");
        assert_eq!(err(Error::BadCredentials), "BadCredentials");
        assert_eq!(err(Error::Parse("x")), "Parse:x");
        assert_eq!(err(Error::Http(503)), "Http:503");
        let n = reqwest::Client::new().get("http://").build().unwrap_err();
        assert_eq!(err(Error::Network(n)), "Network");
    }

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
}
