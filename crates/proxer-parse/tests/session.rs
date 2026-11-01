use proxer_parse::{is_captcha, is_logged_in, login_action};

// Synthetic (no scrubbed capture exists): the shape of the 2026-11 "Captcha Eingabe" page on /info/<id>.
const CAPTCHA: &str = r#"<html><head><title>Info</title></head><body><a id="mod_logout_submit">out</a>
<div id="main"><h3>Captcha Eingabe</h3><form id="captcha" onsubmit="return checkCaptcha()"><div class="g-recaptcha" data-sitekey="x"></div></form></div></body></html>"#;

#[test]
fn captcha_page_detected_and_still_logged_in() {
    assert!(is_captcha(CAPTCHA));
    assert!(is_logged_in(CAPTCHA)); // why the login check alone let it through
    assert!(is_captcha(r#"<div id="main"><div id="captcha"></div><div class="x g-recaptcha"></div></div>"#));
    assert!(is_captcha(r#"<html><form id="captcha"><div class="g-recaptcha"></div></form></html>"#)); // no #main: whole page
}

#[test]
fn one_marker_alone_is_not_captcha() {
    // both markers present, but the widget sits outside #main
    assert!(!is_captcha(r#"<html><body>captcha<div id="main"><div id="captcha"></div></div><div class="g-recaptcha"></div></body></html>"#));
    // /info/77017 (2026-11): a normal title page the owner sees without any check, but with an invisible widget in #main
    let invisible = r#"<html><head><title>Info</title></head><body><a id="mod_logout_submit">out</a><div id="main"><h1>Titel</h1>
<form id="comment"><div class="g-recaptcha" data-sitekey="x" data-size="invisible"></div></form></div></body></html>"#;
    assert!(!is_captcha(invisible));
    // a bare id="captcha" could collide with any other element (a label, a help anchor), so the widget is required too
    assert!(!is_captcha(r#"<div id="main"><form id="captcha"><input name="x"></form></div>"#));
}

#[test]
fn normal_pages_are_not_captcha() {
    let mut n = 0;
    for e in std::fs::read_dir("../../fixtures").unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|x| x == "html") {
            assert!(!is_captcha(&std::fs::read_to_string(&p).unwrap()), "{}", p.display());
            n += 1;
        }
    }
    assert!(n >= 10, "{n}"); // every vendored fixture (10 at 8c18a5e)
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
