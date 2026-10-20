<script lang="ts">
  import type { SearchHit } from "../api";
  import Poster from "./Poster.svelte";
  let { hits, onopen }: { hits: SearchHit[]; onopen: (id: number) => void } = $props();
</script>

<div class="grid">
  {#each hits as h (h.id)}<Poster id={h.id} title={h.title} rating={h.rating} onclick={() => onopen(h.id)} />{/each}
</div>

<style>
  .grid { display: grid; grid-template-columns: repeat(auto-fill, var(--poster-w)); justify-content: start; gap: var(--gap); padding: 0 var(--pad); }
</style>
