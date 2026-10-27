use base64::Engine;
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

pub const HLS_MIME: &str = "application/x-mpegURL";

pub fn parse_embed(html: &str) -> Result<MediaSource, EmbedError> {
    let doc = Html::parse_document(html);
    // Airing embeds: the static <source> is a throttled fallback MP4 (~1.5 Mbit/s for a ~4 Mbit/s file); the site plays HLS.
    if let Some(url) = hls_source(&doc) {
        return Ok(MediaSource { expires_at: expiry(&url), url, mime: HLS_MIME.into() });
    }
    let sel = Selector::parse("video source[src]").unwrap();
    let el = doc.select(&sel).next().ok_or(EmbedError::NoSource)?;
    let url = el.value().attr("src").unwrap_or_default().to_string();
    if !url.starts_with("https://") {
        return Err(EmbedError::NoSource);
    }
    let mime = el.value().attr("type").unwrap_or("video/mp4").to_string();
    Ok(MediaSource { expires_at: expiry(&url), url, mime })
}

/// The HLS entry of `window.plyr_config.playlist[0].sources`, set by an inline `data:` base64 script.
fn hls_source(doc: &Html) -> Option<String> {
    const DATA: &str = "data:text/javascript;base64,";
    let sel = Selector::parse("script[src]").unwrap();
    let js = doc.select(&sel).filter_map(|e| e.value().attr("src")?.strip_prefix(DATA)).find_map(|b| {
        let js = String::from_utf8(base64::engine::general_purpose::STANDARD.decode(b).ok()?).ok()?;
        js.contains("plyr_config").then_some(js)
    })?;
    let cfg: serde_json::Value = serde_json::from_str(js[js.find('{')?..=js.rfind('}')?].trim()).ok()?;
    cfg["playlist"][0]["sources"].as_array()?.iter().find_map(|s| {
        let url = s["src"].as_str()?;
        (s["type"].as_str()?.eq_ignore_ascii_case(HLS_MIME) && url.starts_with("https://")).then(|| url.to_string())
    })
}

fn expiry(url: &str) -> Option<u64> {
    let query = url.split_once('?')?.1;
    query.split('&').find_map(|kv| kv.strip_prefix("t=")).and_then(|v| v.parse().ok())
}
