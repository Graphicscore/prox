use scraper::{Html, Selector};

pub fn login_action(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("form[action^='/login']").unwrap();
    doc.select(&sel).next()?.value().attr("action").map(String::from)
}

pub fn is_logged_in(html: &str) -> bool {
    html.contains("id=\"mod_logout_submit\"")
}

/// Proxer's reCAPTCHA identity check ("Captcha Eingabe"): an element with `id="captcha"` or class `g-recaptcha`
/// inside `#main` (anywhere if the page has no `#main`). It keeps the logout marker, so it must be checked separately.
pub fn is_captcha(html: &str) -> bool {
    if !html.to_ascii_lowercase().contains("captcha") {
        return false; // cheap pre-check: normal pages never mention it
    }
    let doc = Html::parse_document(html);
    let main = Selector::parse("#main").unwrap();
    let mark = Selector::parse("#captcha, .g-recaptcha").unwrap();
    match doc.select(&main).next() {
        Some(m) => m.select(&mark).next().is_some(),
        None => doc.select(&mark).next().is_some(),
    }
}
