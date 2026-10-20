use proxer_parse::{is_logged_in, login_action};

#[test]
fn anon_home_has_login_form() {
    let html = std::fs::read_to_string("../../fixtures/home-anon.html").unwrap();
    assert!(login_action(&html).unwrap().starts_with("/login?"));
    assert!(!is_logged_in(&html));
}

#[test]
#[ignore = "needs fixtures/info-45.html (user-saved)"]
fn info_fixture_is_logged_in() {
    let html = std::fs::read_to_string("../../fixtures/info-45.html").unwrap();
    assert!(is_logged_in(&html));
}

#[test]
fn inline_logged_in_detection() {
    assert!(is_logged_in(r#"<a id="mod_logout_submit">out</a>"#));
    assert!(!is_logged_in("<html></html>"));
}
