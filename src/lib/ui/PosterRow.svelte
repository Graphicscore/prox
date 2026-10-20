<script lang="ts">
  import type { SearchHit } from "../api";
  import Poster from "./Poster.svelte";
  let { title, hits, status, error = "", onopen, onretry }: {
    title: string; hits: SearchHit[]; status: "loading" | "ready" | "error"; error?: string;
    onopen: (id: number) => void; onretry: () => void;
  } = $props();
</script>

{#if status !== "ready" || hits.length > 0}
  <section class="row">
    <h2>{title}</h2>
    {#if status === "error"}
      <p role="alert">{error} <button onclick={onretry}>Retry</button></p>
    {:else if status === "loading"}
      <div class="strip" aria-busy="true">{#each Array(8) as _, i (i)}<div class="skel"></div>{/each}</div>
    {:else}
      <div class="strip">
        {#each hits as h (h.id)}<Poster id={h.id} title={h.title} rating={h.badge ?? h.rating} onclick={() => onopen(h.id)} />{/each}
      </div>
    {/if}
  </section>
{/if}

<style>
  .row { padding: 0 var(--pad); margin: 18px 0; }
  h2 { font-size: 1.05rem; color: var(--text-hi); margin-bottom: 10px; }
  .strip { display: flex; gap: var(--gap); overflow-x: auto; padding: 10px; scroll-padding: 0 var(--pad); }
  .skel { flex: 0 0 var(--poster-w); height: var(--poster-h); border-radius: var(--radius); background: var(--bg-3); }
  p { color: var(--muted); }
</style>
