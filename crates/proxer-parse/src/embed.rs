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
