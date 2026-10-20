use crate::creds::CredStore;
use proxer_client::{Client, Error};
use proxer_parse::{parse_episode_list, parse_info, parse_search_page, parse_embed, parse_streams, EpisodeList, Info, MediaSource, SearchPage, Stream, StreamKind};
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
}
