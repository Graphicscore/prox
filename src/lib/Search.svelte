<script lang="ts">
  import { onMount, tick } from "svelte";
  import { api, describeError, type SearchHit } from "./api";
  import PosterGrid from "./ui/PosterGrid.svelte";
  import Loading from "./ui/Loading.svelte";
  let { onopen, onauth }: { onopen: (id: number) => void; onauth: () => void } = $props();
  let q = $state(""), hits = $state<SearchHit[]>([]), error = $state("");
  let term = $state(""), page = $state(1), hasNext = $state(false), busy = $state(false), searched = $state(false);
  async function load(query: string, p: number) {
    if (busy) return; // buttons stay enabled (TV focus); ignore repeats while a search runs
    error = ""; busy = true;
    try {
      const r = await api.search(query, p);
      hits = r.hits; hasNext = r.has_next; term = query; page = p; searched = true;
    } catch (x) { if (x === "NotLoggedIn") onauth(); else error = describeError(x); }
    busy = false;
    await tick(); if (searched && nearEnd()) loadMore(); // first page shorter than the screen
  }
  // More results load by themselves near the end of the grid (remote focus scrolls there too): a sentinel below
  // the grid, watched with some margin, asks for the next Proxer page.
  let sentinel = $state<HTMLElement>(), more = $state(false);
  const nearEnd = () => !!sentinel && sentinel.offsetParent !== null && sentinel.getBoundingClientRect().top < innerHeight + 800;
  async function loadMore() {
    if (busy || more || !hasNext) return;
    more = true;
    try {
      const r = await api.search(term, page + 1);
      const seen = new Set(hits.map((h) => h.id));
      hits = [...hits, ...r.hits.filter((h) => !seen.has(h.id))];
      page += 1; hasNext = r.has_next && r.hits.length > 0;
    } catch (x) { if (x === "NotLoggedIn") onauth(); else error = describeError(x); hasNext = false; }
    more = false;
    await tick();
    if (nearEnd()) loadMore(); // still at the end (short page or tall screen): keep filling
  }
  $effect(() => {
    if (!sentinel) return;
    const io = new IntersectionObserver((es) => { if (es.some((e) => e.isIntersecting)) loadMore(); }, { rootMargin: "0px 0px 800px 0px" });
    io.observe(sentinel);
    return () => io.disconnect();
  });
  if (import.meta.env.DEV) onMount(() => { const d = new URLSearchParams(location.search).get("q"); if (d) { q = d; load(d, 1); } }); // dev demo hook
  function go(e: Event) { e.preventDefault(); load(q, 1); }
</script>

<form class="bar" onsubmit={go}>
  <input bind:value={q} placeholder="Search anime" aria-label="Search anime" />
  <button class="primary">Search</button>
</form>
{#if busy}<Loading label="Searching…" />{/if}
{#if error}<p role="alert" class="msg">{error}</p>{/if}
{#if searched && !hits.length && !error}<p class="msg">No results.</p>{/if}
<PosterGrid {hits} {onopen} />
<div bind:this={sentinel} aria-hidden="true"></div>
{#if more}<Loading label="Loading more…" />{/if}

<style>
  .bar { display: flex; gap: 10px; padding: 18px var(--pad); }
  .bar input { flex: 1; min-width: 0; }
  .msg { padding: 0 var(--pad) 12px; color: var(--muted); }
</style>
