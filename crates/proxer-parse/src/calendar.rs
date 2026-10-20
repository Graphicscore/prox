use scraper::{ElementRef, Html, Selector};
use serde::Serialize;

/// One Anime-Countdown entry (`/calendar`, grid view). `air` is unix seconds.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CalendarEntry {
    pub id: u32,
    pub title: String,
    pub episode: u32,
    pub lang: String,
    pub air: u64,
}

fn text(e: ElementRef) -> String {
    e.text().collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Entries without a title link, a watch link or an air time are skipped.
pub fn parse_calendar(html: &str) -> Vec<CalendarEntry> {
    let doc = Html::parse_document(html);
    let sel = |s: &str| Selector::parse(s).expect("selector");
    let (entry, title, watch, air) = (
        sel("div.calendarEntry"),
        sel(r#"a[data-ajax][href^="/info/"]"#),
        sel(r#"a[href^="/watch/"]"#),
        sel(r#"span[id^="airTime"]"#),
    );
    doc.select(&entry)
        .filter_map(|e| {
            let t = e.select(&title).next()?;
            let id = t.value().attr("href")?.strip_prefix("/info/")?.split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()?;
            // /watch/<id>/<episode>/<lang>#top
            let w = e.select(&watch).next()?.value().attr("href")?;
            let mut parts = w.trim_start_matches("/watch/").split(['/', '#']);
            parts.next()?;
            let episode = parts.next()?.parse().ok()?;
            let lang = parts.next()?.to_string();
            if lang.is_empty() || !lang.bytes().all(|b| b.is_ascii_lowercase()) {
                return None;
            }
            let air = text(e.select(&air).next()?).parse().ok()?;
            Some(CalendarEntry { id, title: text(t), episode, lang, air })
        })
        .collect()
}
