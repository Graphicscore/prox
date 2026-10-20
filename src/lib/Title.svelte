<script lang="ts">
  import { onMount } from "svelte";
  import { api, describeError, streamLabel, type Episode, type Info, type PageLink, type Stream } from "./api";
  import { pushBack } from "./back";
  import Player from "./Player.svelte";
  import Hero from "./ui/Hero.svelte";
  import EpisodePicker from "./ui/EpisodePicker.svelte";
  import { languagesOf } from "./ui/helpers";
  let { id, onback, onauth }: { id: number; onback: () => void; onauth: () => void } = $props();
  let info = $state<Info | null>(null), episodes = $state<Episode[]>([]), pages = $state<PageLink[]>([]);
  let epPage = $state(1), streams = $state<Stream[]>([]), lang = $state(""), playing = $state<Stream | null>(null);
  let selected = $state<{ ep: number; lang: string } | null>(null), error = $state("");
  onMount(() => pushBack(() => { onback(); return true; }));
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
    const l = lang || languagesOf(episodes)[0];
    const first = episodes.find((e) => e.langs.includes(l));
    if (!first) return;
    await pick(first.number, l);
    const s = streams.find((x) => streamLabel(x) !== "Unsupported");
    if (s) playing = s;
  }
  const meta = $derived([info?.fields["Status"], info?.fields["Genre"]].filter(Boolean).join(" · "));
</script>

{#if error}<p role="alert" class="err">{error}</p>{/if}
{#if info}
  <Hero {id} title={info.title} {meta}>
    <button class="primary" onclick={watch} disabled={!episodes.length}>▶ Watch</button>
    {#each ["Note", "Favorite", "Finished", "Watchlist: this ep.", "Watchlist: next ep."] as label (label)}
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
