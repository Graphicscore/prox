use proxer_parse::{is_captcha, is_logged_in, login_action};

// Synthetic (no scrubbed capture exists): the shape of the 2026-11 "Captcha Eingabe" page on /info/<id>.
const CAPTCHA: &str = r#"<html><head><title>Info</title></head><body><a id="mod_logout_submit">out</a>
<div id="main"><h3>Captcha Eingabe</h3><form id="captcha" onsubmit="return checkCaptcha()"><div class="g-recaptcha" data-sitekey="x"></div></form></div></body></html>"#;

#[test]
fn captcha_page_detected_and_still_logged_in() {
    assert!(is_captcha(CAPTCHA));
    assert!(is_logged_in(CAPTCHA)); // why the login check alone let it through
    assert!(is_captcha(r#"<div id="main"><div class="x g-recaptcha"></div></div>"#));
    assert!(is_captcha(r#"<html><div class="g-recaptcha"></div></html>"#)); // no #main: whole page
}

#[test]
fn normal_pages_are_not_captcha() {
    for f in ["info-45", "home-anon", "watch-45-1", "ucp-anime", "list-airing", "search-naruto", "calendar"] {
        let html = std::fs::read_to_string(format!("../../fixtures/{f}.html")).unwrap();
        assert!(!is_captcha(&html), "{f}");
    }
    // the word alone, or a marker outside #main, is not the check page
    assert!(!is_captcha(r#"<div id="main">Captcha im Titel</div>"#));
    assert!(!is_captcha(r#"<div id="login"><div class="g-recaptcha"></div></div><div id="main">ok</div>"#));
    assert!(!is_captcha(""));
}

#[test]
fn anon_home_has_login_form() {
    let html = std::fs::read_to_string("../../fixtures/home-anon.html").unwrap();
    assert!(login_action(&html).unwrap().starts_with("/login?"));
    assert!(!is_logged_in(&html));
}

#[test]
fn info_fixture_is_logged_in() {
    let html = std::fs::read_to_string("../../fixtures/info-45.html").unwrap();
    assert!(is_logged_in(&html));
}

#[test]
fn inline_logged_in_detection() {
    assert!(is_logged_in(r#"<a id="mod_logout_submit">out</a>"#));
    assert!(!is_logged_in("<html></html>"));
}
