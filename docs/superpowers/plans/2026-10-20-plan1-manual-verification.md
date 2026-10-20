# Plan 1 manual verification

Agents cannot launch the GUI or log in. Walk through this once with `npm run tauri dev`.

1. First start shows the login form.
2. Log in with "Remember me" on, then restart the app: the form is skipped.
3. Wrong password shows "Wrong username or password" and stays on the form.
4. Search `naruto`: lists Naruto (Animeserie/TV) and only anime rows (no manga, light novels, one-shots).
5. Open a hit: title, Status and an episode list appear. Pick an episode and language: streams are labelled [Normal], [Airing] or [Unsupported].
6. Open `/info/59782` (Tensei shitara Ken deshita 2, airing): airing streams appear; an episode without streams shows "No streams (yet)".
7. Open an anime with 100+ episodes and check the last episode appears (episode list pagination is unverified).
8. Log out, then restart: the form shows again (keychain entry cleared).
9. Optional: make the keychain unavailable with "Remember me" on: a warning shows and Search still loads.

## Pending fixtures

Save by hand from a logged-in browser (Ctrl+S, "Webpage, HTML only"), replace the username with `testuser`, then un-ignore the three `#[ignore]` tests in `crates/proxer-parse/tests`:

- `fixtures/info-45.html` from `https://proxer.me/info/45`
- `fixtures/watch-45-1.html` from `https://proxer.me/watch/45/1/engsub`
