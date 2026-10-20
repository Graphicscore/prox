use scraper::{Html, Selector};

pub fn login_action(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("form[action^='/login']").unwrap();
    doc.select(&sel).next()?.value().attr("action").map(String::from)
}

pub fn is_logged_in(html: &str) -> bool {
    html.contains("id=\"mod_logout_submit\"")
}
