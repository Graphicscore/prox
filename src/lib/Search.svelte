<script lang="ts">
  import { api, type SearchHit } from "./api";
  let { onopen, onauth }: { onopen: (id: number) => void; onauth: () => void } = $props();
  let q = $state(""), hits = $state<SearchHit[]>([]), error = $state("");
  async function go(e: Event) {
    e.preventDefault(); error = "";
    try { hits = await api.search(q); }
    catch (x) { if (x === "NotLoggedIn") onauth(); else error = "Search failed"; }
  }
</script>

<form onsubmit={go}><input bind:value={q} placeholder="Search anime" aria-label="Search anime" /><button>Search</button></form>
{#if error}<p role="alert">{error}</p>{/if}
<ul>
  {#each hits as h (h.id)}
    <li><button onclick={() => onopen(h.id)}>{h.title}</button> <small>{h.kind} · {h.genres.join(", ")} · {h.rating ?? "–"}</small></li>
  {/each}
</ul>
