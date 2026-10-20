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
