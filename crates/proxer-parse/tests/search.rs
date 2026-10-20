use proxer_parse::{parse_search, parse_search_page, SearchHit};

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
fn ids_are_unique() {
    // The real page lists every result several times (100 rows, 25 titles); the UI keys its list by id.
    let html = std::fs::read_to_string("../../fixtures/search-naruto.html").unwrap();
    let hits = parse_search(&html);
    let ids: std::collections::HashSet<u32> = hits.iter().map(|h| h.id).collect();
    assert_eq!(ids.len(), hits.len());
    assert_eq!(hits.len(), 25);
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

#[test]
fn fixture_page_has_next() {
    let html = std::fs::read_to_string("../../fixtures/search-naruto.html").unwrap();
    let page = parse_search_page(&html);
    assert_eq!(page.hits.len(), 25);
    assert!(page.has_next);
}

#[test]
fn last_page_has_no_next() {
    let html = r#"<a class="menu" href="/search?p=1">&lt;</a><a class="menu active" href="/search?p=2">2</a>
                  <table><tr class="entry7"><td></td><td><a href="/info/7#top">Seven</a></td></tr></table>"#;
    let page = parse_search_page(html);
    assert_eq!(page.hits.len(), 1);
    assert!(!page.has_next);
}

#[test]
fn inline_next_link_is_detected() {
    assert!(parse_search_page(r#"<a class="menu" href="/search?p=2">&gt;</a>"#).has_next);
}
