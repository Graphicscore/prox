use proxer_parse::{action_result, parse_my_list, plan_watched, token, ListEntry, Step};

const FIX: &str = include_str!("../../../fixtures/ucp-anime.html");

fn entry(state: u8, episode: u32, episodes: u32) -> ListEntry {
    ListEntry { id: 1, cid: 9, name: "x".into(), episode, episodes, state, timestamp: 0, medium: "animeseries".into() }
}

#[test]
fn token_found_and_validated() {
    assert_eq!(token(FIX).as_deref(), Some("abc123DEF"));
    assert_eq!(token("<html></html>"), None);
    assert_eq!(token(r#"<input id="proxerToken" value="a&b=1">"#), None);
    assert_eq!(token(r#"<input id="proxerToken" value="">"#), None);
}

#[test]
fn my_list_from_fixture() {
    let l = parse_my_list(FIX).unwrap();
    assert_eq!(l.len(), 4);
    assert_eq!(l[0], ListEntry { id: 59782, cid: 1001, name: "Tensei shitara Ken deshita 2".into(), episode: 0, episodes: 12, state: 2, timestamp: 1792500000, medium: "animeseries".into() });
    assert_eq!(l[1].name, "Oshi ga [Budoukan] \"Itte\"");
    assert_eq!((l[1].state, l[1].episode), (1, 5));
    assert_eq!(l[3].episodes, 0);
    assert!(l.iter().all(|e| e.id != 7));
}

#[test]
fn my_list_errors_and_empty() {
    assert!(parse_my_list("<html></html>").is_err());
    assert!(parse_my_list(r#"<script>var ucp_list_items = [{"id":"1""#).is_err());
    assert_eq!(parse_my_list("<script>var ucp_list_items = [];</script>").unwrap(), vec![]);
}

#[test]
fn action_results() {
    assert_eq!(action_result(r#"{"error":0,"msg":"ok"}"#), Some(Ok("ok".into())));
    assert_eq!(action_result(r#"{"error":1,"msg":"nein"}"#), Some(Err("nein".into())));
    assert_eq!(action_result(r#"{"error":"1","msg":"x"}"#), Some(Err("x".into())));
    assert_eq!(action_result(r#"{"msg":"Lesezeichen gesetzt"}"#), Some(Ok("Lesezeichen gesetzt".into())));
    assert_eq!(action_result("<html>login</html>"), None);
}

#[test]
fn action_result_prefers_title_as_plain_text() {
    let note = r#"{"uid":"1","code":"2","title":"Dieser Eintrag ist bereits in deiner Liste.","msg":"<i>User-CP</i> -> <i><a href=\"/ucp?s=anime#top\">Anime</a></i>"}"#;
    assert_eq!(action_result(note), Some(Ok("Dieser Eintrag ist bereits in deiner Liste.".into())));
    let favor = r#"{"uid":"1","code":"2","title":"Du hast bereits 10 Eintr&auml;ge in deinen Favoriten.","msg":"<i>User-CP</i> -> <i><a href=\"/ucp?s=anime#top\">Anime</a></i>"}"#;
    assert_eq!(action_result(favor), Some(Ok("Du hast bereits 10 Einträge in deinen Favoriten.".into())));
    let msg_only = r#"{"title":"","msg":"<i>User-CP</i> -> <i><a href=\"/ucp?s=anime#top\">Anime</a></i>"}"#;
    assert_eq!(action_result(msg_only), Some(Ok("User-CP -> Anime".into())));
    assert_eq!(action_result(r#"{"error":1,"title":"Nicht &amp; erlaubt"}"#), Some(Err("Nicht & erlaubt".into())));
}

#[test]
fn watched_plans() {
    // planned -> progress + Am Schauen
    assert_eq!(plan_watched(&entry(2, 0, 12), 1), vec![Step::Progress(1), Step::State(1)]);
    // dropped and watching again -> Am Schauen
    assert_eq!(plan_watched(&entry(3, 4, 12), 5), vec![Step::Progress(5), Step::State(1)]);
    // already watching, progress only
    assert_eq!(plan_watched(&entry(1, 3, 12), 4), vec![Step::Progress(4)]);
    // never lowers progress
    assert_eq!(plan_watched(&entry(1, 6, 12), 4), vec![]);
    // reaches the total -> Geschaut
    assert_eq!(plan_watched(&entry(1, 11, 12), 12), vec![Step::Progress(12), Step::State(0)]);
    // unknown total (airing) never completes
    assert_eq!(plan_watched(&entry(1, 3, 0), 4), vec![Step::Progress(4)]);
    // finished title rewatched: nothing
    assert_eq!(plan_watched(&entry(0, 12, 12), 3), vec![]);
}
