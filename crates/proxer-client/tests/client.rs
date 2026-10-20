use proxer_client::{Client, Error};
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const LOGIN_PAGE: &str = r#"<form method="post" action="/login?abc123=1"><input name="username"><input name="password"></form>"#;
const IN: &str = r#"<a id="mod_logout_submit">out</a>"#;

#[tokio::test]
async fn login_ok_and_bad() {
    let s = MockServer::start().await;
    Mock::given(method("GET")).and(path("/")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).mount(&s).await;
    Mock::given(method("POST")).and(path_regex("^/login")).respond_with(ResponseTemplate::new(200).set_body_string(IN)).up_to_n_times(1).mount(&s).await;
    let c = Client::new(&s.uri());
    assert!(c.login("u", "p").await.is_ok());
    Mock::given(method("POST")).and(path_regex("^/login")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).mount(&s).await;
    assert!(matches!(c.login("u", "bad").await, Err(Error::BadCredentials)));
}

#[tokio::test]
async fn get_relogins_once() {
    let s = MockServer::start().await;
    Mock::given(method("GET")).and(path("/")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).mount(&s).await;
    Mock::given(method("POST")).and(path_regex("^/login")).respond_with(ResponseTemplate::new(200).set_body_string(IN)).expect(1).mount(&s).await;
    Mock::given(method("GET")).and(path("/info/1")).respond_with(ResponseTemplate::new(200).set_body_string("<html>404</html>")).up_to_n_times(2).mount(&s).await;
    Mock::given(method("GET")).and(path("/info/1")).respond_with(ResponseTemplate::new(200).set_body_string(IN)).mount(&s).await;
    let c = Client::new(&s.uri());
    c.set_credentials("u", "p").await;
    assert_eq!(c.get("/info/1").await.unwrap(), IN);
}

#[tokio::test]
async fn get_after_failed_relogin_is_not_logged_in() {
    let s = MockServer::start().await;
    Mock::given(method("GET")).and(path("/")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).mount(&s).await;
    Mock::given(method("POST")).and(path_regex("^/login")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).expect(1).mount(&s).await;
    Mock::given(method("GET")).and(path("/info/1")).respond_with(ResponseTemplate::new(200).set_body_string("<html></html>")).mount(&s).await;
    let c = Client::new(&s.uri());
    c.set_credentials("u", "bad").await;
    assert!(matches!(c.get("/info/1").await, Err(Error::NotLoggedIn)));
    // bad credentials were cleared: no further login POST (expect(1) above)
    assert!(matches!(c.get("/info/1").await, Err(Error::NotLoggedIn)));
}

#[tokio::test]
async fn concurrent_gets_login_once() {
    let s = MockServer::start().await;
    Mock::given(method("GET")).and(path("/")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).mount(&s).await;
    Mock::given(method("POST")).and(path_regex("^/login")).respond_with(ResponseTemplate::new(200).set_body_string(IN)).expect(1).mount(&s).await;
    Mock::given(method("GET")).and(path("/info/1")).respond_with(ResponseTemplate::new(200).set_body_string("<html>404</html>")).up_to_n_times(3).mount(&s).await;
    Mock::given(method("GET")).and(path("/info/1")).respond_with(ResponseTemplate::new(200).set_body_string(IN)).mount(&s).await;
    let c = Client::new(&s.uri());
    c.set_credentials("u", "p").await;
    let (a, b) = tokio::join!(c.get("/info/1"), c.get("/info/1"));
    assert_eq!(a.unwrap(), IN);
    assert_eq!(b.unwrap(), IN);
}

#[tokio::test]
async fn get_without_credentials_is_not_logged_in() {
    let s = MockServer::start().await;
    Mock::given(method("GET")).and(path("/info/1")).respond_with(ResponseTemplate::new(200).set_body_string("<html></html>")).mount(&s).await;
    let c = Client::new(&s.uri());
    assert!(matches!(c.get("/info/1").await, Err(Error::NotLoggedIn)));
}

#[tokio::test]
async fn logout_drops_session_and_credentials() {
    use wiremock::matchers::header_regex;
    let s = MockServer::start().await;
    Mock::given(method("GET")).and(path("/")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).mount(&s).await;
    Mock::given(method("POST")).and(path_regex("^/login")).respond_with(ResponseTemplate::new(200).append_header("Set-Cookie", "sid=1; Path=/").set_body_string(IN)).expect(1).mount(&s).await;
    Mock::given(method("GET")).and(path("/info/1")).and(header_regex("cookie", "sid=1")).respond_with(ResponseTemplate::new(200).set_body_string(IN)).mount(&s).await;
    Mock::given(method("GET")).and(path("/info/1")).respond_with(ResponseTemplate::new(200).set_body_string("<html></html>")).mount(&s).await;
    let c = Client::new(&s.uri());
    c.login("u", "p").await.unwrap();
    assert_eq!(c.get("/info/1").await.unwrap(), IN);
    c.logout().await;
    assert!(matches!(c.get("/info/1").await, Err(Error::NotLoggedIn)));
    // expect(1) on the login POST: logout left no credentials to re-login with
}

#[tokio::test]
async fn login_post_500_is_http_error_and_keeps_credentials() {
    let s = MockServer::start().await;
    Mock::given(method("GET")).and(path("/")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).mount(&s).await;
    Mock::given(method("POST")).and(path_regex("^/login")).respond_with(ResponseTemplate::new(500).set_body_string("oops")).mount(&s).await;
    let c = Client::new(&s.uri());
    c.set_credentials("u", "p").await;
    assert!(matches!(c.login("u", "p").await, Err(Error::Http(500))));
    // credentials untouched: a later get still tries to re-login (page is logged out -> POST again -> Http)
    Mock::given(method("GET")).and(path("/info/1")).respond_with(ResponseTemplate::new(200).set_body_string("<html></html>")).mount(&s).await;
    assert!(matches!(c.get("/info/1").await, Err(Error::Http(500))));
}

#[tokio::test]
async fn login_200_with_form_is_bad_credentials() {
    let s = MockServer::start().await;
    Mock::given(method("GET")).and(path("/")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).mount(&s).await;
    Mock::given(method("POST")).and(path_regex("^/login")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).mount(&s).await;
    assert!(matches!(Client::new(&s.uri()).login("u", "p").await, Err(Error::BadCredentials)));
}

#[tokio::test]
async fn login_200_without_markers_is_parse_error() {
    let s = MockServer::start().await;
    Mock::given(method("GET")).and(path("/")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).mount(&s).await;
    Mock::given(method("POST")).and(path_regex("^/login")).respond_with(ResponseTemplate::new(200).set_body_string("<html>maintenance</html>")).mount(&s).await;
    assert!(matches!(Client::new(&s.uri()).login("u", "p").await, Err(Error::Parse("login result"))));
}

#[tokio::test]
async fn get_5xx_and_429_are_http_errors_without_relogin() {
    for code in [503u16, 429] {
        let s = MockServer::start().await;
        Mock::given(method("GET")).and(path("/info/1")).respond_with(ResponseTemplate::new(code)).expect(1).mount(&s).await;
        Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_string(IN)).expect(0).mount(&s).await;
        let c = Client::new(&s.uri());
        c.set_credentials("u", "p").await;
        assert!(matches!(c.get("/info/1").await, Err(Error::Http(n)) if n == code));
        // MockServer verifies expect(1)/expect(0) on drop
    }
}

#[tokio::test]
async fn login_twice_on_same_client_succeeds() {
    use wiremock::matchers::header_regex;
    let s = MockServer::start().await;
    // home shows the login form only when no sid cookie is sent
    Mock::given(method("GET")).and(path("/")).and(header_regex("cookie", "sid=1")).respond_with(ResponseTemplate::new(200).set_body_string(IN)).mount(&s).await;
    Mock::given(method("GET")).and(path("/")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).mount(&s).await;
    Mock::given(method("POST")).and(path_regex("^/login")).respond_with(ResponseTemplate::new(200).append_header("Set-Cookie", "sid=1; Path=/").set_body_string(IN)).mount(&s).await;
    let c = Client::new(&s.uri());
    c.login("u", "p").await.unwrap();
    c.login("u", "p").await.unwrap();
}

#[tokio::test]
async fn login_posts_real_form_fields() {
    use wiremock::matchers::body_string_contains;
    let s = MockServer::start().await;
    Mock::given(method("GET")).and(path("/")).respond_with(ResponseTemplate::new(200).set_body_string(LOGIN_PAGE)).mount(&s).await;
    Mock::given(method("POST")).and(path_regex("^/login")).and(body_string_contains("remember=1")).and(body_string_contains("submit=Login")).and(body_string_contains("secretkey=")).respond_with(ResponseTemplate::new(200).set_body_string(IN)).expect(1).mount(&s).await;
    Client::new(&s.uri()).login("u", "p").await.unwrap();
}
