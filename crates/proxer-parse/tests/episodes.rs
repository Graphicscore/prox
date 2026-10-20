use proxer_parse::{parse_episodes, Episode};

#[test]
fn groups_languages_per_episode() {
    let html = r#"<a href="/watch/59782/2/gersub">x</a><a href="/watch/59782/1/engsub">x</a>
                  <a href="/watch/59782/2/engsub">x</a><a href="/watch/99/1/engsub">other anime</a>"#;
    assert_eq!(
        parse_episodes(html, 59782),
        vec![
            Episode { number: 1, langs: vec!["engsub".into()] },
            Episode { number: 2, langs: vec!["engsub".into(), "gersub".into()] },
        ]
    );
}

#[test]
fn no_links_is_empty() {
    assert!(parse_episodes("<html></html>", 1).is_empty());
}
