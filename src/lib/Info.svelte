<script lang="ts">
  import { api, streamLabel, describeError, type Info, type Episode, type PageLink, type Stream } from "./api";
  import Player from "./Player.svelte";
  let { id, onback, onauth }: { id: number; onback: () => void; onauth: () => void } = $props();
  let info = $state<Info | null>(null), episodes = $state<Episode[]>([]), pages = $state<PageLink[]>([]);
  let playing = $state<Stream | null>(null);
  let epPage = $state(1), streams = $state<Stream[]>([]);
  let selected = $state<{ ep: number; lang: string } | null>(null), error = $state("");
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
  async function pick(ep: number, lang: string) {
    playing = null; selected = { ep, lang }; streams = []; error = "";
    const mine = selected; // drop the result if another pick happened meanwhile
    try {
      const r = await api.streams(id, ep, lang);
      if (selected === mine) streams = r;
    } catch (x) { if (selected === mine && x !== "Parse:streams") fail(x); }
  }
</script>

<button onclick={onback}>Back</button>
{#if error}<p role="alert">{error}</p>{/if}
{#if info}
  <h2>{info.title}</h2>
  <p>Status: {info.fields["Status"] ?? "–"}</p>
  <h3>Episodes</h3>
  {#if pages.length > 1}
    <nav aria-label="Episode pages">
      {#each pages as p (p.page)}
        <button aria-current={p.page === epPage ? "page" : undefined} disabled={p.page === epPage} onclick={() => loadPage(p.page)}>{p.label}</button>
      {/each}
    </nav>
  {/if}
  <ul>
    {#each episodes as e (e.number)}
      <li>{e.number}: {#each e.langs as l (l)}<button onclick={() => pick(e.number, l)}>{l}</button> {/each}</li>
    {:else}<li>No episodes</li>{/each}
  </ul>
  {#if selected}
    <h3>Streams for episode {selected.ep} ({selected.lang})</h3>
    <ul>
      {#each streams as s}
        <li>
          <button disabled={streamLabel(s) === "Unsupported"} onclick={() => (playing = s)}>{s.name}</button>
          <small>[{streamLabel(s)}]</small>
        </li>
      {:else}<li>No streams (yet)</li>{/each}
    </ul>
  {/if}
{/if}
{#if playing}
  {#key playing}
    <Player stream={playing} onback={() => (playing = null)} {onauth} />
  {/key}
{/if}
