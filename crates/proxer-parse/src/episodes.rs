use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Episode {
    pub number: u32,
    pub langs: Vec<String>,
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
