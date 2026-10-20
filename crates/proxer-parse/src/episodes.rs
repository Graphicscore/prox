use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Episode {
    pub number: u32,
    pub langs: Vec<String>,
}

/// One link of the episode pager ("1 - 50", "51 - 100", ...), 50 episodes per page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageLink {
    pub page: u32,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EpisodeList {
    pub episodes: Vec<Episode>,
    pub pages: Vec<PageLink>,
}

pub fn parse_episode_pages(html: &str, id: u32) -> Vec<PageLink> {
    let re = Regex::new(&format!(r##"href="/info/{id}/list/(\d+)#top"[^>]*>\s*([^<]*?)\s*</a>"##)).unwrap();
    let mut by_page: BTreeMap<u32, String> = BTreeMap::new();
    for c in re.captures_iter(html) {
        if let Ok(page) = c[1].parse() {
            by_page.entry(page).or_insert_with(|| c[2].to_string());
        }
    }
    by_page.into_iter().map(|(page, label)| PageLink { page, label }).collect()
}

pub fn parse_episode_list(html: &str, id: u32) -> EpisodeList {
    EpisodeList { episodes: parse_episodes(html, id), pages: parse_episode_pages(html, id) }
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
