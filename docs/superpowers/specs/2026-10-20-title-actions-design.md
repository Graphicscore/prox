# Piece 2: title actions, my list, next episode: Design

Date: 2026-10-20
Status: written autonomously (user: "make choices on my behalf"); choices are marked **Choice**.

## Scope

1. **Title actions** on the title hero: Vermerken (Note), Favorisieren (Favorite), Abgeschlossen (Finished).
2. **Watchlist: this / next episode** on the episode area, next to the selected episode's streams (user: they belong to the episode UI, not the show overview).
3. **My list row** on Home: titles from the user's Proxer anime list, "Am Schauen" (currently watching) first, then "Wird noch geschaut" (planned).
4. **Next episode**: in the player a "Next episode" button; near the end of a video it pops up over the video; when the video ends the next episode autoplays.
5. **Mark watched**: an episode played to the end (or left via "Next episode" near the end) sets the list progress to that episode.

Anime-Kalender stays piece 3.

## Findings (logged-in pages, read in the user's browser, nothing clicked)

- **Token**: every logged-in page has a hidden input `#proxerToken`. Its VALUE is used as a query parameter NAME with value 1 on every write. Never log it; reqwest errors are stripped of URLs.
- **Title actions**: `POST /info/<id>?format=json&json=<note|favor|finish>&<token>=1`, body `checkPost=1`. Response JSON `{uid, code, title, msg}` (verified live): `title` is the readable German sentence (HTML entities), `msg` an HTML breadcrumb; no `error` key. Favorites are capped at 10 (code 3). The info page shows no current state for these (plain links).
- **Reminder**: `POST /watch/<id>/<ep>/<lang>?format=json&type=reminder&<token>=1&title=<reminder_this|reminder_next>`, body `check=1`. Response JSON with `msg`.
- **My list**: `GET /ucp?s=anime` embeds the whole list as `var ucp_list_items = [{...}];` (567 entries for this user, 148 KB, one page). Each item has string fields `id` (title id), `cid` (list entry id), `name`, `episode` (progress), `episodes` (total, "0" if unknown), `comment_state` ("0" Geschaut, "1" Am Schauen, "2" Wird noch geschaut, "3" Abgebrochen), `timestamp` (unix, last change), `medium`, `media_format`, plus fields we ignore (comment, rating, ...). Not sorted.
- **Progress**: `POST /ucp?format=json&type=incState&id=<cid>&value=<n>&<token>=1`, body `check=1`. Sets the progress to `n` (absolute, despite the name).
- **Status**: `POST /comment?id=<cid>&format=json&json=updateState&status=<0..3>&<token>=1`, body `check=1`. Response `{error, msg}`.

## Choices

- **Choice (user)**: an episode that ends for a title not in the list adds it as "Am Schauen" with progress = that episode.
- **Choice (user)**: watched = video ended, or "Next episode" pressed in the near-end window. Progress is never lowered.
- **Choice**: a title in "Wird noch geschaut" moves to "Am Schauen" when an episode is marked watched; when progress reaches the known total (`episodes > 0`), status becomes "Geschaut".
- **Choice**: near-end window = last 90 s of the video, or the last 10 % if the video is shorter than 15 min. The pop-up button takes focus only if focus is on the video (TV: Center then plays next).
- **Choice**: autoplay starts the next episode immediately on `ended` (no countdown), same language, first playable stream (same rule as Watch). If the next episode is on the next episode page, that page is loaded first. No next episode: nothing happens.
- **Choice**: embedded (iframe) streams cannot report time or end: they get the toolbar "Next episode" button only, no autoplay and no mark-watched.
- **Choice**: the hero shows the list state as a meta chip, e.g. "Am Schauen · 3/12". Note is shown pressed when the state is "Wird noch geschaut", Finished when "Geschaut". Favorite has no readable state: plain button. Every action shows Proxer's message (German, as-is) in a status line under the hero; failures show it as an error.
- **Choice**: Home row title "My list"; it is the first row (above Airing now); progress "3/12" is shown in the poster's badge spot; row hides when empty like other rows. The hero stays the first Airing title.
- **Choice**: list data is fetched fresh when needed (Home mount, Title open, after a write). No cache.

## Architecture

### Rust
- `proxer-parse`:
  - `token(html) -> Option<String>`: value of `#proxerToken`.
  - `parse_my_list(html) -> Result<Vec<ListEntry>, ParseError>`: finds `var ucp_list_items = `, cuts the JSON array (bracket matching that respects strings), deserializes with serde (string fields to numbers). `ListEntry { id: u32, cid: u64, name: String, episode: u32, episodes: u32, state: u8, timestamp: u64, medium: String }`. Unknown/extra fields ignored; an entry with an unparsable number is skipped.
  - `action_result(json) -> Result<String, String>`: `{error:0,msg}` -> Ok(msg), `{error:1,msg}` -> Err(msg); missing `error` counts as Ok (reminder responses).
- `proxer-client`: `post(path, form) -> Result<String, Error>` through the same gate; URL stripped from errors; no redirect following needed (default ok).
- Commands (`src-tauri/src/commands.rs`), all allowlisted inputs:
  - `my_list() -> Vec<ListEntry>`
  - `title_action(id, action)` with `action` in {note, favor, finish} -> `String` (msg). Reads the token from `/info/<id>`.
  - `reminder(id, episode, lang, which)` with `which` in {this, next} and `lang` matching `^[a-z]+$` -> `String`. Token from the watch page.
  - `mark_watched(id, episode)` -> `()`; logic in a pure function `plan_watched(entry: Option<&ListEntry>, episode) -> Vec<Step>` (Step = Note | SetState(u8) | SetProgress(u32)), unit-tested; the command executes the steps (Note then re-read the list to get `cid`).
- Errors map to the existing strings (`NotLoggedIn`, `Http:<n>`, `Network`, `Parse:<x>`, `Unsupported`); a Proxer `{error:1}` becomes `Proxer:<msg>`; `describeError` shows the message.

### Frontend
- `api.ts`: `myList`, `titleAction`, `reminder`, `markWatched`; demo API gets fakes (in-memory list).
- `ui/helpers.ts`: `myListHits(entries)` (state 1 by timestamp desc, then state 2 by timestamp desc, mapped to poster items with badge "ep/total"), `nextEpisode(episodes, lang, ep, pages, epPage)` -> `{ep} | {page} | null`, `nearEnd(currentTime, duration)`.
- `Home.svelte`: "My list" row first.
- `Title.svelte`: wires the three hero buttons + list chip + status line; passes `onnext` / `onwatched` / `hasNext` to `Player`; generalizes `watch()` into `playEpisode(n)`.
- `EpisodePicker.svelte`: Watchlist buttons get `onreminder(which)`.
- `Player.svelte`: toolbar "Next episode" button when `hasNext`; near-end pop-up over the video (also in fullscreen); `ended` -> `onwatched()` then `onnext()`; pressing Next in the window -> `onwatched()` too.

## Testing
- Rust unit tests: `token`, `parse_my_list` (synthetic fixture `fixtures/ucp-anime.html` in the real shape, a handful of entries, no personal data; plus a broken-JSON case), `action_result`, `plan_watched` (not in list, state 2, already ahead, reaches total, unknown total), command allowlists.
- Web tests: `myListHits`, `nextEpisode` (same page, page boundary, last episode), `nearEnd`.
- Live checks on the TV emulator by the agent: Home "My list" row matches the account; Note/Favorite/Finished/Watchlist show Proxer's message; play the last seconds of an episode -> progress/state change visible in the list; Next episode button and autoplay. Test writes on the user's account are reverted afterwards (progress and state restored; favorite toggled back if it toggles).

## Risks
- `ucp_list_items` is a script variable; if Proxer renames it, the row and mark-watched fail (Parse error shown, rest of the app unaffected).
- Favorite may not toggle; the message tells the user what happened.
- 148 KB list fetch per Home load and per mark-watched: acceptable (80 ms in the browser).
