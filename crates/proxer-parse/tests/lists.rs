use proxer_parse::parse_search_page;

fn read(name: &str) -> String {
    std::fs::read_to_string(format!("../../fixtures/{name}")).unwrap()
}

#[test]
fn airing_list_parses_like_search() {
    let page = parse_search_page(&read("list-airing.html"));
    assert_eq!(page.hits.len(), 50);
    let ids: std::collections::HashSet<u32> = page.hits.iter().map(|h| h.id).collect();
    assert_eq!(ids.len(), 50);
    assert!(page.hits.iter().any(|h| h.id == 53 && h.title == "One Piece"));
    assert!(page.has_next);
}

#[test]
fn rating_list_parses_like_search() {
    let page = parse_search_page(&read("list-rating.html"));
    assert_eq!(page.hits.len(), 50);
    assert_eq!(page.hits[0].id, 53);
    assert_eq!(page.hits[0].rating, Some(9.64));
    assert!(page.has_next);
}
