<script lang="ts">
  import { onMount } from "svelte";
  import { api, describeError, streamLabel, type Episode, type Info, type ListEntry, type PageLink, type Stream, type TitleAction } from "./api";
  import { pushBack } from "./back";
  import Player from "./Player.svelte";
  import Hero from "./ui/Hero.svelte";
  import EpisodePicker from "./ui/EpisodePicker.svelte";
  import { activeLang, episodesWith, nextEpisode } from "./ui/helpers";
  let { id, onback, onauth }: { id: number; onback: () => void; onauth: () => void } = $props();
  let info = $state<Info | null>(null), episodes = $state<Episode[]>([]), pages = $state<PageLink[]>([]);
  let epPage = $state(1), streams = $state<Stream[]>([]), lang = $state(""), playing = $state<Stream | null>(null), playKey = $state(0);
  let selected = $state<{ ep: number; lang: string } | null>(null), error = $state("");
  onMount(() => pushBack(() => { onback(); return true; }));
  // Remote users land on Watch instead of nothing (Home/Search are hidden, so focus fell back to <body>).
  let watchBtn = $state<HTMLButtonElement>();
  $effect(() => { if (watchBtn && episodes.length && document.activeElement === document.body) watchBtn.focus({ preventScroll: true }); });
  const STATES = ["Geschaut", "Am Schauen", "Wird noch geschaut", "Abgebrochen"];
  let entry = $state<ListEntry | null>(null), message = $state(""), busy = false;
  async function loadEntry() {
    const cur = id;
    try { const l = await api.myList(); if (cur === id) entry = l.find((e) => e.id === cur) ?? null; }
    catch (x) { if (x === "NotLoggedIn") onauth(); } // the chip is optional: the page stays usable without it
  }
  /** Run a write; show Proxer's message, refresh the list state. Buttons stay enabled (TV focus), so guard re-entry. */
  async function act(run: () => Promise<string>) {
    if (busy) return;
    const cur = id;
    busy = true; message = ""; error = "";
    try { const m = (await run()) || "Done"; if (cur === id) { message = m; await loadEntry(); } }
    catch (x) { if (cur === id) fail(x); } // another title is open now: drop the stale result
    finally { busy = false; }
  }
  const titleAct = (a: TitleAction) => act(() => api.titleAction(id, a));
  function fail(x: unknown) { if (x === "NotLoggedIn") onauth(); else error = describeError(x); }
  async function loadPage(p: number) {
    const cur = id;
    try {
      const l = await api.episodes(cur, p);
      if (cur !== id) return;
      episodes = l.episodes; epPage = p;
      if (l.pages.length) pages = l.pages;
      error = "";
    } catch (x) { fail(x); }
  }
  $effect(() => {
    const cur = id;
    entry = null; message = "";
    (async () => {
      try { const i = await api.info(cur); if (cur === id) info = i; } catch (x) { fail(x); return; }
      await loadPage(1);
      loadEntry();
      if (import.meta.env.DEV) { // dev demo hook: ?pick=1 shows streams, ?play=1 opens the player
        const d = new URLSearchParams(location.search);
        if (d.get("play")) await watch(); else if (d.get("pick")) await pick(1, activeLang(episodes, lang));
      }
    })();
  });
  async function pick(ep: number, l: string) {
    selected = { ep, lang: l }; streams = []; playing = null; error = "";
    const mine = selected; // drop the result if another pick happened meanwhile
    try {
      const r = await api.streams(id, ep, l);
      if (selected === mine) streams = r;
    } catch (x) { if (selected === mine && x !== "Parse:streams") fail(x); }
  }
  /** Play the first supported stream of the first episode in the chosen (or first available) language. */
  async function watch() {
    const l = activeLang(episodes, lang);
    const first = episodesWith(episodes, l)[0];
    if (first === undefined) return;
    await pick(first, l);
    if (selected?.ep !== first || selected.lang !== l) return; // user picked another tile meanwhile
    const s = streams.find((x) => streamLabel(x) !== "Unsupported");
    if (s) { playing = s; playKey++; } else if (!error) error = "No playable stream for episode " + first;
  }
  const hasNext = $derived(!!selected && nextEpisode(episodes, selected.lang, selected.ep, pages, epPage) !== null);
  /** Next episode in the same language, first playable stream; keeps the Player instance (fullscreen stays). */
  async function playNext() {
    const cur = selected, cid = id;
    if (!cur || !playing) return;
    let n = nextEpisode(episodes, cur.lang, cur.ep, pages, epPage);
    if (n && "page" in n) { await loadPage(n.page); if (cid !== id || epPage !== n.page) return; n = nextEpisode(episodes, cur.lang, 0, pages, epPage); }
    if (!n || !("ep" in n) || selected !== cur || cid !== id) return;
    const ep = n.ep;
    try {
      const r = await api.streams(id, ep, cur.lang);
      if (selected !== cur || cid !== id) return; // user picked something else meanwhile
      const s = r.find((x) => streamLabel(x) !== "Unsupported");
      if (!s) { error = "No playable stream for episode " + ep; return; }
      selected = { ep, lang: cur.lang }; streams = r; playing = s;
    } catch (x) { fail(x); }
  }
  function watched() {
    const ep = selected?.ep, cur = id;
    if (ep) api.markWatched(id, ep).then(() => { if (cur === id) loadEntry(); }, (x) => { if (cur === id) fail(x); });
  }
  const meta = $derived([info?.fields["Status"], info?.fields["Genre"], entry && `${STATES[entry.state] ?? ""} · ${entry.episode}/${entry.episodes || "?"}`].filter(Boolean).join(" · "));
</script>

{#if error}<p role="alert" class="err">{error}</p>{/if}
{#if info}
  <Hero {id} title={info.title} {meta}>
    <button class="primary" bind:this={watchBtn} onclick={watch} disabled={!episodes.length}>▶ Watch</button>
    <button aria-pressed={entry?.state === 2} onclick={() => titleAct("note")}>Note</button>
    <button onclick={() => titleAct("favor")}>Favorite</button>
    <button aria-pressed={entry?.state === 0} onclick={() => titleAct("finish")}>Finished</button>
  </Hero>
  {#if message}<p role="status" class="msg">{message}</p>{/if}
  <EpisodePicker {episodes} {pages} {epPage} {selected} {streams} bind:lang onpage={loadPage} onpick={pick} onplay={(s) => { playing = s; playKey++; }} onreminder={(w) => { const s = selected; if (s) act(() => api.reminder(id, s.ep, s.lang, w)); }} />
  {#if playing}
    {#key playKey}
      <Player stream={playing} onback={() => (playing = null)} {onauth} {hasNext} onnext={playNext} onwatched={watched} />
    {/key}
  {/if}
{/if}

<style>
  .err { padding: 12px var(--pad); color: var(--bad); }
  .msg { padding: 8px var(--pad) 0; color: var(--muted); }
  button[aria-pressed="true"] { background: var(--good); color: #fff; }
</style>
