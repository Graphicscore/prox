<script lang="ts">
  import Loading from "./Loading.svelte";
  import { streamLabel, type Episode, type PageLink, type Stream } from "../api";
  import { activeLang, episodesWith, languagesOf, pageWindow, streamThumb } from "./helpers";
  let { episodes, pages, epPage, selected, streams, loadingPage = 0, loadingStreams = false, onpage, onpick, onplay, onreminder, lang = $bindable("") }: {
    episodes: Episode[]; pages: PageLink[]; epPage: number; loadingPage?: number; loadingStreams?: boolean; selected: { ep: number; lang: string } | null; streams: Stream[];
    onpage: (p: number) => void; onpick: (ep: number, lang: string) => void; onplay: (s: Stream) => void; onreminder: (which: "this" | "next") => void; lang?: string;
  } = $props();
  const langs = $derived(languagesOf(episodes));
  const active = $derived(activeLang(episodes, lang));
  const nums = $derived(episodesWith(episodes, active));
</script>

<section class="picker">
  <div class="chips">
    <h2>Episodes</h2>
    {#each langs as l (l)}
      <button class="chip" aria-pressed={l === active} onclick={() => (lang = l)}>{l}</button>
    {/each}
    {#if pages.length > 1}
      <span class="sep"></span>
      {#each pageWindow(pages, epPage) as p, i (p ? p.page : "gap" + i)}
        {#if !p}<span class="gap">…</span>{:else}
        <button class="chip" aria-current={p.page === epPage ? "page" : undefined} onclick={() => p.page !== epPage && onpage(p.page)}>{p.label}</button>{/if}
      {/each}
    {/if}
  </div>
  {#if loadingPage}<Loading compact label="Loading episodes…" />{/if}
  <div class="tiles" class:busy={loadingPage}>
    {#each nums as n (n)}
      <button class="tile" aria-pressed={selected?.ep === n && selected?.lang === active} onclick={() => onpick(n, active)}>{n}</button>
    {:else}<p class="muted">No episodes</p>{/each}
  </div>
  {#if selected}
    <h3>Streams for episode {selected.ep} ({selected.lang})</h3>
    {@const thumb = streams.map(streamThumb).find(Boolean)}
    {#if thumb}<img class="thumb fade-in" src={thumb} alt="Episode {selected.ep} preview" />{/if}
    <div class="streams fade-in">
      {#each streams as s}
        {@const label = streamLabel(s)}
        <button class="stream" disabled={label === "Unsupported"} onclick={() => onplay(s)}>▶ {s.name} <span class="badge {label.toLowerCase()}">{label}</span></button>
      {:else}{#if loadingStreams}<Loading compact label="Loading streams…" />{:else}<p class="muted">No streams (yet)</p>{/if}{/each}
      <span class="sep"></span>
      <button onclick={() => onreminder("this")}>Watchlist: this episode</button>
      <button onclick={() => onreminder("next")}>Watchlist: next episode</button>
    </div>
  {/if}
</section>

<style>
  .picker { padding: 12px var(--pad) 0; }
  .chips { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; margin-bottom: 12px; }
  h2 { font-size: 1.05rem; color: var(--text-hi); margin-right: 6px; }
  h3 { font-size: 0.95rem; color: var(--text-hi); margin: 18px 0 8px; }
  .sep { width: 12px; }
  .gap { color: var(--muted); padding: 0 2px; }
  .chip[aria-pressed="true"], .chip[aria-current="page"] { background: var(--accent); color: #fff; }
  .tiles { display: grid; grid-template-columns: repeat(auto-fill, minmax(4.2rem, 1fr)); gap: 8px; }
  .tile { border-radius: 6px; padding: 0.7em 0; }
  .tile[aria-pressed="true"] { background: var(--accent); }
  .streams { display: flex; flex-wrap: wrap; gap: 10px; align-items: center; }
  .badge { font-size: 0.7rem; padding: 1px 6px; border-radius: 4px; margin-left: 6px; background: var(--bg-4); }
  .badge.normal { background: var(--accent); }
  .badge.airing { background: var(--good); }
  .thumb { display: block; width: min(16rem, 100%); aspect-ratio: 16 / 9; object-fit: cover; border-radius: var(--radius); background: var(--bg-3); margin-bottom: 10px; }
  .busy { opacity: 0.45; transition: opacity 0.15s ease; }
  .muted { color: var(--muted); }
</style>
