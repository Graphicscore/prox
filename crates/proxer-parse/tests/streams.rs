use proxer_parse::{parse_streams, ParseError, StreamKind};

#[test]
#[ignore = "needs fixtures/watch-45-1.html (user-saved)"]
fn parses_streams_fixture() {
    let html = std::fs::read_to_string("../../fixtures/watch-45-1.html").unwrap();
    let s = parse_streams(&html).unwrap();
    assert!(s.iter().any(|x| x.kind == "proxer-stream"
        && x.replace == "https://stream.proxer.me/embed-#-728x504.html"));
}

#[test]
fn missing_streams_is_error() {
    assert!(matches!(parse_streams("<html></html>"), Err(ParseError::NoStreams)));
}

fn page(kind: &str, name: &str) -> String {
    format!(r#"<script>var streams = [{{"id":"1","code":"abc","type":"{kind}","htype":"iframe","name":"{name}","replace":"https://example.test/embed-#-728x504.html"}}];</script>"#)
}

#[test]
fn normal_and_airing_kinds() {
    let n = parse_streams(&page("proxer-stream", "Proxer-Stream")).unwrap();
    let a = parse_streams(&page("ps-test", "Proxer-Stream Airing")).unwrap();
    let o = parse_streams(&page("something-else", "X")).unwrap();
    assert_eq!(n[0].stream_kind(), StreamKind::Normal);
    assert_eq!(a[0].stream_kind(), StreamKind::Airing);
    assert_eq!(o[0].stream_kind(), StreamKind::Other);
}

#[test]
fn empty_streams_array_is_ok() {
    let s = parse_streams("<script>var streams = [];</script>").unwrap();
    assert!(s.is_empty());
}
