<script lang="ts">
  import { api, describeError, type SearchHit } from "./api";
  import PosterGrid from "./ui/PosterGrid.svelte";
  let { onopen, onauth }: { onopen: (id: number) => void; onauth: () => void } = $props();
  let q = $state(""), hits = $state<SearchHit[]>([]), error = $state("");
  let term = $state(""), page = $state(1), hasNext = $state(false), busy = $state(false), searched = $state(false);
  async function load(query: string, p: number) {
    error = ""; busy = true;
    try {
      const r = await api.search(query, p);
      hits = r.hits; hasNext = r.has_next; term = query; page = p; searched = true;
    } catch (x) { if (x === "NotLoggedIn") onauth(); else error = describeError(x); }
    busy = false;
  }
  function go(e: Event) { e.preventDefault(); load(q, 1); }
</script>

<form class="bar" onsubmit={go}>
  <input bind:value={q} placeholder="Search anime" aria-label="Search anime" />
  <button class="primary">Search</button>
</form>
{#if error}<p role="alert" class="msg">{error}</p>{/if}
{#if searched && !hits.length && !error}<p class="msg">No results.</p>{/if}
<PosterGrid {hits} {onopen} />
{#if hits.length && (page > 1 || hasNext)}
  <nav class="pager" aria-label="Search pages">
    <button disabled={busy || page <= 1} onclick={() => load(term, page - 1)}>Previous</button>
    <span>Page {page}</span>
    <button disabled={busy || !hasNext} onclick={() => load(term, page + 1)}>Next</button>
  </nav>
{/if}

<style>
  .bar { display: flex; gap: 10px; padding: 18px var(--pad); }
  .bar input { flex: 1; min-width: 0; }
  .msg { padding: 0 var(--pad) 12px; color: var(--muted); }
  .pager { display: flex; gap: 12px; align-items: center; justify-content: center; padding: 18px; }
</style>
