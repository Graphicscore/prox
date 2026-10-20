use proxer_parse::{parse_search, SearchHit};

#[test]
fn parses_known_row() {
    let html = std::fs::read_to_string("../../fixtures/search-naruto.html").unwrap();
    let hits = parse_search(&html);
    let h: &SearchHit = hits.iter().find(|h| h.id == 12092).expect("row 12092");
    assert_eq!(h.title, "NARUTO外伝 七代目火影と緋色の花つ月");
    assert_eq!(h.kind, "Mangaserie");
    assert!(h.genres.contains(&"Action".to_string()));
    assert!(h.genres.contains(&"Shounen".to_string()));
    assert_eq!(h.rating, Some(7.55));
}

#[test]
fn garbage_gives_empty() {
    assert!(parse_search("<html><body>nothing</body></html>").is_empty());
    assert!(parse_search("").is_empty());
}

#[test]
fn row_without_info_link_is_skipped() {
    let html = r#"<table><tr class="entry1"><td></td><td>no link</td></tr></table>"#;
    assert!(parse_search(html).is_empty());
}
