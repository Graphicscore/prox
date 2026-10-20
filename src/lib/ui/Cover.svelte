<script lang="ts">
  import { coverUrl } from "./helpers";
  let { id, title = "", class: cls = "" }: { id: number; title?: string; class?: string } = $props();
  let failed = $state(false);
</script>

{#if failed}
  <div class="ph {cls}" aria-hidden="true">{title.slice(0, 1)}</div>
{:else}
  <img class={cls} src={coverUrl(id)} alt="" loading="lazy" onerror={() => (failed = true)} />
{/if}

<style>
  img, .ph { width: 100%; height: 100%; object-fit: cover; display: block; border-radius: var(--radius); background: var(--bg-3); }
  .ph { display: flex; align-items: center; justify-content: center; font-size: 2rem; color: var(--muted); }
</style>
