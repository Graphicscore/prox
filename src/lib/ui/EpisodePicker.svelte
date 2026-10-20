<script lang="ts">
  import { streamLabel, type Episode, type PageLink, type Stream } from "../api";
  import { episodesWith, languagesOf } from "./helpers";
  let { episodes, pages, epPage, selected, streams, onpage, onpick, onplay, lang = $bindable("") }: {
    episodes: Episode[]; pages: PageLink[]; epPage: number; selected: { ep: number; lang: string } | null; streams: Stream[];
    onpage: (p: number) => void; onpick: (ep: number, lang: string) => void; onplay: (s: Stream) => void; lang?: string;
  } = $props();
  const langs = $derived(languagesOf(episodes));
  const active = $derived(langs.includes(lang) ? lang : (langs[0] ?? ""));
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
      {#each pages as p (p.page)}
        <button class="chip" aria-current={p.page === epPage ? "page" : undefined} disabled={p.page === epPage} onclick={() => onpage(p.page)}>{p.label}</button>
      {/each}
    {/if}
  </div>
  <div class="tiles">
    {#each nums as n (n)}
      <button class="tile" aria-pressed={selected?.ep === n && selected?.lang === active} onclick={() => onpick(n, active)}>{n}</button>
    {:else}<p class="muted">No episodes</p>{/each}
  </div>
  {#if selected}
    <h3>Streams for episode {selected.ep} ({selected.lang})</h3>
    <div class="streams">
      {#each streams as s}
        {@const label = streamLabel(s)}
        <button class="stream" disabled={label === "Unsupported"} onclick={() => onplay(s)}>▶ {s.name} <span class="badge {label.toLowerCase()}">{label}</span></button>
      {:else}<p class="muted">No streams (yet)</p>{/each}
    </div>
  {/if}
</section>

<style>
  .picker { padding: 12px var(--pad) 0; }
  .chips { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; margin-bottom: 12px; }
  h2 { font-size: 1.05rem; color: var(--text-hi); margin-right: 6px; }
  h3 { font-size: 0.95rem; color: var(--text-hi); margin: 18px 0 8px; }
  .sep { width: 12px; }
  .chip[aria-pressed="true"], .chip[aria-current="page"] { background: var(--bg-4); color: var(--text-hi); }
  .tiles { display: grid; grid-template-columns: repeat(auto-fill, minmax(4.2rem, 1fr)); gap: 8px; }
  .tile { border-radius: 6px; padding: 0.7em 0; }
  .tile[aria-pressed="true"] { background: var(--accent); }
  .streams { display: flex; flex-wrap: wrap; gap: 10px; }
  .badge { font-size: 0.7rem; padding: 1px 6px; border-radius: 4px; margin-left: 6px; background: var(--bg-4); }
  .badge.normal { background: var(--accent); }
  .badge.airing { background: var(--good); }
  .muted { color: var(--muted); }
</style>
