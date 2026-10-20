<script lang="ts">
  import { api, describeError, type SearchHit } from "./api";
  let { onopen, onauth }: { onopen: (id: number) => void; onauth: () => void } = $props();
  let q = $state(""), hits = $state<SearchHit[]>([]), error = $state("");
  let term = $state(""), page = $state(1), hasNext = $state(false), busy = $state(false);
  async function load(query: string, p: number) {
    error = ""; busy = true;
    try {
      const r = await api.search(query, p);
      hits = r.hits; hasNext = r.has_next; term = query; page = p;
    } catch (x) { if (x === "NotLoggedIn") onauth(); else error = describeError(x); }
    busy = false;
  }
  function go(e: Event) { e.preventDefault(); load(q, 1); }
</script>

<form onsubmit={go}><input bind:value={q} placeholder="Search anime" aria-label="Search anime" /><button>Search</button></form>
{#if error}<p role="alert">{error}</p>{/if}
<ul>
  {#each hits as h (h.id)}
    <li><button onclick={() => onopen(h.id)}>{h.title}</button> <small>{h.kind} · {h.genres.join(", ")} · {h.rating ?? "–"}</small></li>
  {/each}
</ul>
{#if hits.length && (page > 1 || hasNext)}
  <nav aria-label="Search pages">
    <button disabled={busy || page <= 1} onclick={() => load(term, page - 1)}>Previous</button>
    <span>Page {page}</span>
    <button disabled={busy || !hasNext} onclick={() => load(term, page + 1)}>Next</button>
  </nav>
{/if}
