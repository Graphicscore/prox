use proxer_parse::{parse_calendar, CalendarEntry};

const FIX: &str = include_str!("../../../fixtures/calendar.html");

#[test]
fn calendar_from_fixture() {
    let c = parse_calendar(FIX);
    assert_eq!(c.len(), 70);
    assert_eq!(c[0], CalendarEntry { id: 76542, title: "Cyberpunk: Edgerunners 2".into(), episode: 1, lang: "engsub".into(), air: 1792479600 });
    assert_eq!(c[1], CalendarEntry { id: 78190, title: "Shiotaiou no Satou-san ga Ore ni dake Amai".into(), episode: 3, lang: "engsub".into(), air: 1792495800 });
    assert!(c.iter().all(|e| e.episode > 0 && e.air > 1_700_000_000 && !e.title.is_empty()));
}

#[test]
fn calendar_empty_and_partial() {
    assert!(parse_calendar("<html></html>").is_empty());
    let no_air = r#"<div class="calendarEntry"><a data-ajax="true" href="/info/5#top">X</a><a href="/watch/5/1/engsub#top">Episode 1</a></div>"#;
    assert!(parse_calendar(no_air).is_empty());
}
