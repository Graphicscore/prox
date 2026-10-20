<script lang="ts">
  import { onMount } from "svelte";
  import { api, describeError, streamLabel, type Episode, type Info, type PageLink, type Stream } from "./api";
  import { pushBack } from "./back";
  import Player from "./Player.svelte";
  import Hero from "./ui/Hero.svelte";
  import EpisodePicker from "./ui/EpisodePicker.svelte";
  import { activeLang, episodesWith } from "./ui/helpers";
  let { id, onback, onauth }: { id: number; onback: () => void; onauth: () => void } = $props();
  let info = $state<Info | null>(null), episodes = $state<Episode[]>([]), pages = $state<PageLink[]>([]);
  let epPage = $state(1), streams = $state<Stream[]>([]), lang = $state(""), playing = $state<Stream | null>(null);
  let selected = $state<{ ep: number; lang: string } | null>(null), error = $state("");
  onMount(() => pushBack(() => { onback(); return true; }));
  // Remote users land on Watch instead of nothing (Home/Search are hidden, so focus fell back to <body>).
  let watchBtn = $state<HTMLButtonElement>();
  $effect(() => { if (watchBtn && episodes.length && document.activeElement === document.body) watchBtn.focus({ preventScroll: true }); });
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
    (async () => {
      try { const i = await api.info(cur); if (cur === id) info = i; } catch (x) { fail(x); return; }
      await loadPage(1);
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
    if (s) playing = s; else if (!error) error = "No playable stream for episode " + first;
  }
  const meta = $derived([info?.fields["Status"], info?.fields["Genre"]].filter(Boolean).join(" · "));
</script>

{#if error}<p role="alert" class="err">{error}</p>{/if}
{#if info}
  <Hero {id} title={info.title} {meta}>
    <button class="primary" bind:this={watchBtn} onclick={watch} disabled={!episodes.length}>▶ Watch</button>
    {#each ["Note", "Favorite", "Finished"] as label (label)}
      <button disabled title="Available in the next update">{label}</button>
    {/each}
  </Hero>
  <EpisodePicker {episodes} {pages} {epPage} {selected} {streams} bind:lang onpage={loadPage} onpick={pick} onplay={(s) => (playing = s)} />
  {#if playing}
    {#key playing}
      <Player stream={playing} onback={() => (playing = null)} {onauth} />
    {/key}
  {/if}
{/if}

<style>
  .err { padding: 12px var(--pad); color: var(--bad); }
</style>
