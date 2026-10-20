use proxer_client::{is_allowed_embed_url, Client, Error};

#[test]
fn allowlist() {
    assert!(is_allowed_embed_url("https://stream.proxer.me/embed-a-1.html"));
    assert!(is_allowed_embed_url("https://abc.proxer.me/embed-a.html"));
    assert!(!is_allowed_embed_url("http://stream.proxer.me/x"));
    assert!(!is_allowed_embed_url("https://evilproxer.me/x"));
    assert!(!is_allowed_embed_url("https://proxer.me.evil.test/x"));
    assert!(!is_allowed_embed_url("https://stream.proxer.me@evil.test/x"));
    assert!(!is_allowed_embed_url("https://proxer.me/x"));
    assert!(!is_allowed_embed_url("not a url"));
}

#[tokio::test]
async fn foreign_host_is_refused_without_a_request() {
    let c = Client::new("https://proxer.me");
    assert!(matches!(c.get_embed("https://evil.test/x").await, Err(Error::Parse("embed host not allowed"))));
}
