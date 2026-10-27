use proxer_parse::{parse_embed, EmbedError, Stream};

#[test]
fn normal_embed_fixture() {
    let html = std::fs::read_to_string("../../fixtures/embed-proxer-stream.html").unwrap();
    let m = parse_embed(&html).unwrap();
    assert!(m.url.starts_with("https://") && m.url.ends_with("/video.mp4"));
    assert_eq!(m.mime, "video/mp4");
    assert_eq!(m.expires_at, None);
}

#[test]
fn airing_embed_prefers_hls_over_fallback_mp4() {
    let html = std::fs::read_to_string("../../fixtures/embed-ps-test.html").unwrap();
    let m = parse_embed(&html).unwrap();
    assert_eq!(m.url, "https://s5.stream-service.proxer.me/hls/REDACTED/m.m3u8?a=REDACTED&t=1790000000");
    assert_eq!(m.mime, "application/x-mpegURL");
    assert_eq!(m.expires_at, Some(1790000000));
}

#[test]
fn config_without_hls_falls_back_to_source() {
    let cfg = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, r#"window.plyr_config = {"playlist":[{"sources":[{"type":"video/mp4","src":"https://x.proxer.me/a.mp4"}]}]};"#);
    let html = format!(r#"<script src="data:text/javascript;base64,{cfg}"></script><video><source type="video/mp4" src="https://s5.stream-service.proxer.me/fb/abc"></video>"#);
    let m = parse_embed(&html).unwrap();
    assert_eq!((m.url.as_str(), m.mime.as_str()), ("https://s5.stream-service.proxer.me/fb/abc", "video/mp4"));
    assert!(parse_embed(r#"<script src="data:text/javascript;base64,!!"></script>"#).is_err()); // garbage base64: no panic
}

#[test]
fn airing_expiry_is_read_from_t() {
    let html = r#"<video><source type="video/mp4" src="https://s5.stream-service.proxer.me/fb/abc?a=XyZ&amp;t=1790000000"></video>"#;
    let m = parse_embed(html).unwrap();
    assert_eq!(m.url, "https://s5.stream-service.proxer.me/fb/abc?a=XyZ&t=1790000000");
    assert_eq!(m.expires_at, Some(1790000000));
}

#[test]
fn missing_or_insecure_source_is_an_error() {
    assert_eq!(parse_embed("<html></html>"), Err(EmbedError::NoSource));
    assert_eq!(parse_embed(r#"<video><source src="http://x.test/a.mp4"></video>"#), Err(EmbedError::NoSource));
}

#[test]
fn embed_url_substitutes_the_code() {
    let s: Stream = serde_json::from_str(
        r#"{"code":"abc","type":"proxer-stream","name":"n","replace":"https://stream.proxer.me/embed-#-728x504.html"}"#,
    )
    .unwrap();
    assert_eq!(s.embed_url(), "https://stream.proxer.me/embed-abc-728x504.html");
}
