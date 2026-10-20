use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Info {
    pub title: String,
    pub fields: BTreeMap<String, String>,
}

fn clean(s: String) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn parse_info(html: &str) -> Info {
    let doc = Html::parse_document(html);
    let title_sel = Selector::parse("title").unwrap();
    let raw = doc.select(&title_sel).next().map(|t| t.text().collect::<String>()).unwrap_or_default();
    let title = raw.split(" (").next().unwrap_or("").trim().to_string();
    let tr = Selector::parse("tr").unwrap();
    let td = Selector::parse("td").unwrap();
    let mut fields = BTreeMap::new();
    for row in doc.select(&tr) {
        let cells: Vec<_> = row.select(&td).collect();
        if let [k, v] = cells.as_slice() {
            let key = clean(k.text().collect());
            let val = clean(v.text().collect());
            if !key.is_empty() && !val.is_empty() {
                fields.entry(key).or_insert(val);
            }
        }
    }
    Info { title, fields }
}
