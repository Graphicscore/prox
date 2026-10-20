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
    Mock::given(method("GET")).and(path("/info/1")).respond_with(ResponseTemplate::new(200).set_body_string("<html>404</html>")).up_to_n_times(1).mount(&s).await;
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
}

#[tokio::test]
async fn get_without_credentials_is_not_logged_in() {
    let s = MockServer::start().await;
    Mock::given(method("GET")).and(path("/info/1")).respond_with(ResponseTemplate::new(200).set_body_string("<html></html>")).mount(&s).await;
    let c = Client::new(&s.uri());
    assert!(matches!(c.get("/info/1").await, Err(Error::NotLoggedIn)));
}
