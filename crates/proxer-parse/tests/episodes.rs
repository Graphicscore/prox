use proxer_parse::{parse_episode_pages, parse_episodes, Episode, PageLink};

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

#[test]
fn episode_pages_from_pager() {
    let html = r##"<a href="/info/53/list/1#top">1 - 50</a> <a href="/info/53/list/2#top"> 51 - 100 </a>
                   <a href="/info/53/list/1#top">1 - 50</a> <a href="/info/99/list/2#top">other</a>"##;
    assert_eq!(
        parse_episode_pages(html, 53),
        vec![
            PageLink { page: 1, label: "1 - 50".into() },
            PageLink { page: 2, label: "51 - 100".into() },
        ]
    );
    assert!(parse_episode_pages("<html></html>", 53).is_empty());
}
