# Piece 3: Anime-Kalender: Design

Date: 2026-10-20
Status: written autonomously (user: "make choices on my behalf"); choices are marked **Choice**.

## Findings
- `GET /calendar` ("Anime-Countdown") is readable anonymously and logged in; 7 days starting today, about 70 entries. Saved as `fixtures/calendar.html` (calendar part only; scripts and forms removed).
- Default (grid) view: per day a `div.headDate` ("Dienstag,\n20. Oktober"), then entries `div.calendarEntry` (class also `entry<titleId>`). Each entry has: title link `a[data-ajax][href^="/info/<id>"]` with the title text; `a[href^="/watch/<id>/<ep>/<lang>"]` with "Episode N"; hidden `span[id^="airTime"]` = unix seconds; `span.average` rating ("0.00" when unrated); a `div` with "Genres: A B C".
- The list view is chosen by a browser cookie (`calendarView`) the app never sets, so the app always gets the grid view.

## Choices
- **Choice**: a "Calendar" tab in the top bar after Search (Home | Search | Calendar), kept mounted like Home/Search so Back restores focus.
- **Choice**: days are grouped by the local date of the air time (not by Proxer's German day headers), sorted by time; headings "Today", "Tomorrow", then e.g. "Thursday 22 October" (en-GB, like the rest of the English UI). **Choice**: two episodes of one title on the same day become one poster, badge "09:00 · Ep 1, 2" (poster rows are keyed by title id).
- **Choice**: each day is a poster row (the existing PosterRow); the badge shows local time and episode, e.g. "09:00 · Ep 1". Opening a poster opens the title page (as from Home).
- **Choice**: entries whose air time is in the past stay listed (Proxer shows them too); no countdown timer.
- **Choice**: no "only my list" filter (YAGNI); can be added later from `myList`.
- Empty calendar: "Nothing scheduled."; errors as on other screens (`describeError`, Retry).

## Architecture
- `proxer-parse`: `parse_calendar(html) -> Vec<CalendarEntry>`, `CalendarEntry { id: u32, title: String, episode: u32, lang: String, air: u64 }`; entries missing id/title/episode/air are skipped. **Choice**: rating and genres are not parsed (the screen shows neither; genres are space-separated with multi-word names like "Slice of Life", so they cannot be split reliably).
- Command `get_calendar() -> Vec<CalendarEntry>` (`GET /calendar`).
- Frontend: `api.getCalendar()`, demo fake; helper `calendarDays(entries, now)` -> `{ key, label, hits }[]` (hits are poster items with `badge`); `Calendar.svelte`; Shell tab; App screen "calendar".

## Testing
- Rust: fixture -> 70 entries, first two entries exact; broken/empty HTML -> empty Vec.
- Web: `calendarDays` grouping across midnight, labels Today/Tomorrow/date, sort by time, badge format (time zone fixed via `TZ=Europe/Berlin` in the test command or by computing expectations with the same local-time functions).
- Agent device check on the TV emulator: tab reachable with the D-pad, rows navigable, opening a title, Back restores focus.
