use proxer_parse::parse_info;

#[test]
fn parses_info_fixture() {
    let html = std::fs::read_to_string("../../fixtures/info-45.html").unwrap();
    let info = parse_info(&html);
    assert_eq!(info.title, "Naruto");
    assert_eq!(info.fields.get("Status").map(String::as_str), Some("Abgeschlossen"));
}

#[test]
fn parses_inline_table() {
    let html = "<html><head><title>Naruto (Anime) - Details - Proxer.Me</title></head><body><table><tr><td>Status</td><td>Abgeschlossen</td></tr></table></body></html>";
    let info = parse_info(html);
    assert_eq!(info.title, "Naruto");
    assert_eq!(info.fields.get("Status").map(String::as_str), Some("Abgeschlossen"));
}
