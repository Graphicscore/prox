# New UI: manual verification checklist

Setup. The human logs in on each device; agents then drive the app.
- Desktop: `env WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri dev`.
- TV and phone emulators: build with `--split-per-abi` and install as in `docs/android-setup.md`.
- TV remote keys: `adb shell input keyevent KEYCODE_DPAD_LEFT|RIGHT|UP|DOWN|CENTER|BACK`. Read state through WebView DevTools as in `docs/superpowers/plans/2026-10-20-plan3-manual-verification.md`.

## Desktop (tauri dev)
- [ ] Home shows hero + rows (Airing now, Top rated, Most viewed); covers load.
- [ ] A "Most viewed" row with no hits is hidden entirely.
- [ ] Forcing a list error shows the message and a Retry button that reloads the row.
- [ ] Search: query shows a cover grid; Previous/Next pages work.
- [ ] Open a title: hero, the Watch button and the five disabled buttons (Note, Favorite, Finished, Watchlist: this ep., Watchlist: next ep.) are visible.
- [ ] Language chips and episode page chips switch; the current one is highlighted.
- [ ] Picking an episode shows its streams right below the tiles.
- [ ] Watch plays the first supported stream; Close player returns to the title; Back returns to Home/Search.

## TV emulator
- [ ] Posters ~190px, 24px text, no scrollbars, focus ring clearly visible.
- [ ] Home: Left/Right moves inside a row, Up/Down moves between rows (and top bar).
- [ ] Center opens a title; language chips and page chips are reachable with the D-pad.
- [ ] Center on an episode shows streams below; Center on a stream or Watch plays.
- [ ] Back closes the player, then the title, then leaves the app.

## Phone emulator
- [ ] No sideways page scroll at 390px; rows scroll inside themselves.
- [ ] Hero shows long titles and wrapped buttons unclipped.
- [ ] Episode tiles are tappable (4 per row), chips wrap cleanly.
- [ ] Watch plays; Android Back closes player, then title.

## Added in final review
- [ ] Login screen (log out to see it): padded from the window edges, fields fill the row; on TV it keeps the overscan margin.
- [ ] Moving Up/Down never leaves the focused element hidden under the top bar (chips, poster tops, focus rings fully visible).
- [ ] TV Search field: arriving by D-pad does not open the keyboard, Select opens it, Back closes it.
- [ ] TV player: the D-pad stays inside the player; fullscreen still works.
- [ ] Right at the end of a poster row may jump to the next row (known behavior).

Screenshots in dev: demo mode is opt-in, add `?demo` to every URL (e.g. `http://localhost:1420/?demo&screen=home&tv=1`; also `&screen=login`).

## Results
(fill in: device, date, pass/fail, notes)

