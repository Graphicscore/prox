# src/ - Svelte 5 UI

## OVERVIEW
Svelte 5 (runes) UI in the system WebView; one codebase for desktop, phone and Android TV (D-pad first). Earned its own file: 30+ files, own test runner, TV/focus logic unique to this dir.

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Screen routing, Back, updater button | `App.svelte` | No router: `screen` state; Home/Search/Calendar stay mounted (hidden) while Title is open |
| Call the Rust side | `lib/api.ts` | `realApi` = one `invoke` per command; types mirror Rust serde structs (snake_case fields) |
| Error message for a code | `lib/api.ts` `describeError` | Maps Rust string codes (`Parse:*`, `Http:N`, `Captcha`, `Proxer:<msg>`...) |
| Demo data (no login) | `lib/demo.ts` | `makeDemoApi()` must satisfy `Api`; add every new api method here too |
| Screens | `lib/{Home,Search,Title,Player,Calendar,Login}.svelte` | Player is the largest (fullscreen, seek, next-episode pop-up, mark watched; HLS via lazy `hls.js/light`) |
| Reusable widgets | `lib/ui/*.svelte` | Shell (top bar/tabs), PosterRow/Grid, EpisodePicker, Hero, Toasts |
| Pure logic (tested) | `lib/ui/helpers.ts` | episode/lang selection, paging windows, calendar days, resume episode |
| D-pad / spatial focus | `lib/spatialNav.ts`, `lib/dpad.ts` | `pickNext` geometry; dpad has seek and fullscreen modes |
| Back stack | `lib/back.ts` | `pushBack(fn)` returns an unregister; newest handler wins |
| TV detection | `lib/tv.ts` | `?tv=1`, localStorage `tv`, `AndroidTv.isTv()` bridge, else Android+no touch |
| TV soft keyboard | `lib/tvInput.ts` | Text fields go readonly on focus; Enter unlocks |
| Shared reactive state | `lib/*.svelte.ts` | `toast.svelte.ts`, `liststate.svelte.ts` (runes outside components) |
| Recently played | `lib/recent.ts` | localStorage; cleared on logout |
| Theme / TV sizing | `theme.css` | `html.tv` class switches the TV layout |

## CONVENTIONS
- Tests: `node --test` runs `.test.ts` directly (Node type stripping): import with the `.ts` extension, only pure modules (no `.svelte`, no Tauri imports). New test files must sit in `lib/` or `lib/ui/` to match the `npm test` glob.
- Components take callbacks as lowercase props (`onopen`, `onback`, `onauth`), Svelte 5 style, no event dispatchers.
- Any layer that can be closed (dialog, player, picker) registers with `pushBack` while open and unregisters on destroy.
- Focus is restored explicitly after navigation (`returnFocus`, `tick()` then `.focus()`); D-pad users lose their place otherwise.
- Untrusted URLs from page data go through `safeEmbedUrl` (mirrors Rust `is_allowed_embed_url`); keep both in sync.
- `AndroidTv.*` is a Kotlin JS bridge (`isTv`, `immersive`, `installUpdate`); always optional-chain it, it is absent on desktop.
- Dev-only behavior stays behind `import.meta.env.DEV` so Rollup drops it (demo, `?screen=`).

## ANTI-PATTERNS
- Do not import demo data at top level or outside the DEV check: it would ship in release builds.
- Do not let D-pad focus open the soft keyboard on TV (it swallows the D-pad).
- Do not unmount Home/Search/Calendar when opening a title; Back must keep their scroll and focused poster.
- Do not reuse a `<video>` element without releasing it (`removeAttribute("src")` + `load()`, and `hls.destroy()` for HLS), and call `play()` on `canplay`: Android WebView requires it.
- Keyed lists must stay unique by id (appended pages skip ids already shown).

## UI WORK WITHOUT LOGIN
`npm run dev`, then `http://localhost:1420/?demo&screen=home` (`search&q=naruto`, `title&id=53&pick=1`, `calendar`, `login`; add `&tv=1` for the TV layout).
