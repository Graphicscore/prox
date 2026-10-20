# Plan 1 manual verification

Agents cannot launch the GUI or log in. Walk through this once with `npm run tauri dev`.

1. First start shows the login form.
2. Log in with "Remember me" on, then restart the app: the form is skipped.
3. Wrong password shows "Wrong username or password" and stays on the form.
4. Search `naruto`: lists Naruto (Animeserie/TV) and only anime rows (no manga, light novels, one-shots).
5. Open a hit: title, Status and an episode list appear. Pick an episode and language: streams are labelled [Normal], [Airing] or [Unsupported].
6. Open `/info/59782` (Tensei shitara Ken deshita 2, airing): airing streams appear; an episode without streams shows "No streams (yet)".
7. Search `one piece`, use Next/Previous, then open it and use the episode page buttons ("1 - 50", "51 - 100", ...) to reach the last episodes.
8. Log out, then restart: the form shows again (keychain entry cleared).
9. Optional: make the keychain unavailable with "Remember me" on: a warning shows and Search still loads.

Status: all steps walked through on 2026-10-20 and working, after the fixes for duplicated search rows and pagination.
