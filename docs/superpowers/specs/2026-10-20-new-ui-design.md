# New UI (piece 1 of 3): Design

Date: 2026-10-20
Status: approved in chat, pending written-spec review

## Request and decomposition

The user asked for a modern, responsive UI for desktop and TV, with the Proxer title actions (Vermerken, Favorisieren, Abgeschlossen, Watchlist: Diese Episode, Watchlist: Nächste Episode) and the Anime-Kalender available in the app. That is three separable pieces, built in this order:

1. **New UI** (this spec): design system, app shell, Home, Search, Title page, Player, covers. The action buttons are drawn but not wired.
2. **Title actions**: wire the five buttons (findings below).
3. **Anime-Kalender**: a Calendar screen and tab (findings below).

Each piece gets its own spec, plan and review cycle. Piece 1 must leave a defined place for the buttons (Title page hero) and for a Calendar tab (added by piece 3, not shown before it works).

## Decisions made with the user

- Layout: **style C, big header** (streaming-app look): blurred cover backdrop with the poster, title, details and action buttons on top; rows and grids below.
- **Covers are used.** Cover URL: `https://cdn.proxer.me/cover/<id>.jpg` (200x280-ish JPEG, `Cache-Control: max-age=31536000`, the CDN returns 403 without a User-Agent, which WebViews always send). The id is the title id, so no parsing is needed. The CSP already allows `img-src https:`.
- **Home = rows of covers:** Airing now (`/anime/airing`), Top rated (`/anime/animeseries/rating/all`), and Most viewed (`/anime/animeseries/clicks/all`). Most viewed currently returns no entries because Proxer reset that list and has not repopulated it (user, 2026-10-20): **any row that is empty hides itself**, so Most viewed appears automatically once it is populated.
- The episode **page switch** ("1-50") sits on the left, right after the language chips (a control far on the right is hard to reach with a remote).
- One layout for desktop, tablet, phone and TV. TV is the same layout with larger tokens and the thick focus ring.

## Findings that shape the build

- `/anime/airing` and `/anime/animeseries/rating/all` return 50 rows each in the same `tr.entry` table as search (6 cells: status, title, genres, kind, flag, rating), so the existing `parse_search_page` reads them. Both have a pager (`>` link) like search.
- The info page fields already give status, genres, rating for the hero.
- Existing remote support (spatial navigation, Back stack, TV mode, IME lock, player focus scope) stays and must keep working for every new component.

## Architecture

Frontend-heavy. One new Tauri command; everything else is Svelte.

### Design tokens (`src/lib/theme.css`)
CSS custom properties for colors (dark theme only), spacing scale, radius, type scale, poster width/height, focus ring, hero height. `html.tv` redefines the sizing tokens (larger posters, 24px base, thicker ring). Breakpoints (phone, tablet, desktop) only change tokens. No CSS framework and no new dependencies. Replaces the scattered CSS in `App.svelte`, `Player.svelte` and others.

### Components (`src/lib/ui/`), one job each
- `Shell.svelte`: top bar (logo, tabs Home and Search, Log out) and the screen area; owns screen state that is in `App.svelte` today.
- `Cover.svelte`: `<img>` from a title id (`coverUrl(id)`), lazy loading, a neutral placeholder on error.
- `PosterRow.svelte`, `PosterGrid.svelte`: a horizontally scrolling row and a wrapped grid of posters (title below, optional rating badge); items are buttons; focused item scrolls into view (center).
- `Hero.svelte`: blurred and darkened cover backdrop (CSS blur hides the low resolution), sharp poster, title, meta line, a slot for buttons.
- `Home.svelte`: hero (first Airing title) plus rows; per-row states: loading (skeleton posters), empty (row hidden), error (small message with retry).
- `Search.svelte`: restyled; results in `PosterGrid`; keeps paging (Previous/Next) and the existing `search` command.
- `Title.svelte`: replaces `Info.svelte`: `Hero` (Watch, Note, Favorite, Finished, Watchlist this/next as drawn buttons; Watch plays the first stream of the first episode of the chosen language), then `EpisodePicker`.
- `EpisodePicker.svelte`: language chips (languages present for the shown page), page dropdown left of the tiles (shown when more than one page), episode tiles, and the streams for the selected episode directly below (Normal/Airing/Unsupported badges). Fixes the old "streams at the bottom of 50 rows" problem and keeps the pick() stale-result guard.
- `Player.svelte`: behavior unchanged (retry, fullscreen, guarded iframe, D-pad scope), restyled with tokens.

### Data
- New Rust command `get_list(kind: String, page: Option<u32>) -> SearchPage`, `kind` in `{airing, rating, clicks}` mapped by a small pure function to the paths above; `p` page parameter handled like search (1-based). Rejects unknown kinds. Reuses `parse_search_page`; no parser change.
- `api.ts`: `getList(kind, page)`, `coverUrl(id)`.
- Hero for Home: the first hit of the Airing list (no extra request).

### Navigation and focus
Screen state with the existing Back stack: Home <-> Title <-> Player; Search <-> Title. Tabs: Home, Search. D-pad: Left/Right along rows and chips and tiles, Up/Down between rows and between sections; the existing `spatialNav`, `dpad`, `back`, `tv`, `tvInput` modules are reused unchanged unless a component needs a larger focus scroll margin.

### Errors and states
- Cover load failure: placeholder tile with the title initial.
- List fetch failure (`Http:*`, `Network`): row shows the existing `describeError` message and a Retry button; `NotLoggedIn` returns to Login as everywhere.
- Empty list: row hidden (Most viewed today).
- No streams yet: "No streams (yet)" as today.

## Out of scope for piece 1
Working action buttons (piece 2), Calendar (piece 3), a Library tab or user lists, light theme, new settings screen, descriptions/synopsis, cast, comments.

## Testing
- Rust: unit tests for the `kind` -> path mapping (incl. unknown kind, page clamp) and `get_list` parsing against saved fixtures `fixtures/list-airing.html` and `fixtures/list-rating.html` (anonymous-readable copies, scrubbed like earlier fixtures).
- Web (`node --test`): `coverUrl`, and pure helpers extracted from components (e.g. language chips from an episode list, "empty row hides" decision).
- `npm run check` (0 errors, 0 warnings) and `npm run build`; the existing 50 Rust and 12 web tests stay green.
- Emulator checks (screenshots and WebView DevTools state) on the TV and phone emulators after the user logs in once: Home rows, remote navigation through rows and into a title, episode picker, playback of a Normal and an Airing stream, Back stack. Desktop checked by the user in `tauri dev`.

## Findings for piece 2 (title actions), recorded here so they are not lost
- Vermerken, Favorisieren, Abgeschlossen on the info page are links `a.ajaxLink` with `data-action` = `note`, `favor`, `finish`. Proxer's own script (`/components/com_proxer/js/info/details.js`) does `POST <info page URL>?format=json&json=<action>&<proxerToken>=1` with body `checkPost=1`, where `<proxerToken>` is the VALUE of the hidden input `#proxerToken` used as a parameter NAME with value 1. Response is JSON with `title` and `msg` (German text). The actions toggle; the current state shown on the page (an `active` class, to be verified) must be read to draw the button state.
- Watchlist: Diese Episode / Nächste Episode on the watch page are `a.menu.reminder` with `title` = `reminder_this` / `reminder_next`; Proxer's script (`/components/com_proxer/js/media/default.js`) does `POST <watch page URL>?format=json&type=reminder&<proxerToken>=1&title=<title>` with body `check=1`; response JSON with `msg`.
- Both need the logged-in session and a fresh `#proxerToken` read from the page just before the POST. Never log the token. The responses are German strings to show as-is.

## Findings for piece 3 (Anime-Kalender)
- `/calendar` ("Anime-Countdown") is server-rendered, grouped by day ("Dienstag, 20. Oktober"). Each entry is `div.calendarEntry` containing the title link `/info/<id>`, "Episode N", "Austrahlung: HH:MM Uhr", two unix timestamps (air time and a countdown target), genres and a rating. About 140 title links on the page. Parseable like the other pages; covers by id.

## Risks
- Cover resolution is low (about 200x280): posters are shown small and the hero uses a blurred backdrop; on TV posters are scaled up modestly.
- The CDN needs a User-Agent (fine in WebViews; would break a plain non-browser fetch).
- Proxer list pages can change or be empty (Most viewed now): rows must degrade silently.
