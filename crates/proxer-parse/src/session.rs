use scraper::{Html, Selector};

pub fn login_action(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("form[action^='/login']").unwrap();
    doc.select(&sel).next()?.value().attr("action").map(String::from)
}

pub fn is_logged_in(html: &str) -> bool {
    html.contains("id=\"mod_logout_submit\"")
}

/// Proxer's reCAPTCHA identity check ("Captcha Eingabe"): BOTH an element with `id="captcha"` (the check form) and one
/// with class `g-recaptcha` inside `#main` (anywhere if the page has no `#main`). An invisible widget alone (e.g. on a
/// comment form of a normal title page) is not the check, nor is a bare `id="captcha"`, which another element could carry.
/// The page keeps the logout marker, so it must be checked separately.
pub fn is_captcha(html: &str) -> bool {
    if !html.to_ascii_lowercase().contains("captcha") {
        return false; // cheap pre-check: normal pages never mention it
    }
    let doc = Html::parse_document(html);
    let main = Selector::parse("#main").unwrap();
    let scope = doc.select(&main).next().unwrap_or_else(|| doc.root_element());
    ["#captcha", ".g-recaptcha"].iter().all(|s| scope.select(&Selector::parse(s).unwrap()).next().is_some())
}
