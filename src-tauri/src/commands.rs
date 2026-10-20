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
