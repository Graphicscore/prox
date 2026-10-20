use proxer_parse::{parse_search, SearchHit};

#[test]
fn parses_known_row() {
    let html = std::fs::read_to_string("../../fixtures/search-naruto.html").unwrap();
    let hits = parse_search(&html);
    let h: &SearchHit = hits.iter().find(|h| h.id == 45).expect("row 45");
    assert_eq!(h.title, "Naruto");
    assert_eq!(h.kind, "Animeserie/TV");
    assert!(h.genres.contains(&"Action".to_string()));
    assert!(h.genres.contains(&"Shounen".to_string()));
    assert_eq!(h.rating, Some(8.37));
    assert!(hits.iter().all(|h| !h.kind.contains("Manga") && !h.kind.contains("Novel")));
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
