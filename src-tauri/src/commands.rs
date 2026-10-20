use crate::creds::CredStore;
use proxer_client::{Client, Error};
use proxer_parse::{parse_episode_list, parse_info, parse_search_page, parse_streams, EpisodeList, Info, SearchPage, Stream};
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
    fn err_codes() {
        assert_eq!(err(Error::NotLoggedIn), "NotLoggedIn");
        assert_eq!(err(Error::BadCredentials), "BadCredentials");
        assert_eq!(err(Error::Parse("x")), "Parse:x");
        assert_eq!(err(Error::Http(503)), "Http:503");
        let n = reqwest::Client::new().get("http://").build().unwrap_err();
        assert_eq!(err(Error::Network(n)), "Network");
    }
}
