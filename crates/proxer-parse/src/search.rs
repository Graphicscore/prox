use scraper::{Html, Selector};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchHit {
    pub id: u32,
    pub title: String,
    pub genres: Vec<String>,
    pub kind: String,
    pub rating: Option<f32>,
}

fn text(e: scraper::ElementRef) -> String {
    e.text().collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn parse_search(html: &str) -> Vec<SearchHit> {
    let doc = Html::parse_document(html);
    let row = Selector::parse("tr[class^=entry]").unwrap();
    let td = Selector::parse("td").unwrap();
    let link = Selector::parse("a[href^='/info/']").unwrap();
    doc.select(&row)
        .filter_map(|r| {
            let a = r.select(&link).next()?;
            let id = a.value().attr("href")?.trim_start_matches("/info/").split('#').next()?.parse().ok()?;
            let cells: Vec<_> = r.select(&td).collect();
            let genres = cells.get(2).map(|c| text(*c).split(' ').filter(|s| !s.is_empty()).map(String::from).collect()).unwrap_or_default();
            let kind = cells.get(3).map(|c| text(*c)).unwrap_or_default();
            let rating = cells.get(5).and_then(|c| text(*c).split(' ').next().and_then(|n| n.parse().ok()));
            Some(SearchHit { id, title: text(a), genres, kind, rating })
        })
        .collect()
}
