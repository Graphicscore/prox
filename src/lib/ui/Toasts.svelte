<script lang="ts">
  import { toasts } from "../toast.svelte";
</script>

<div class="toasts" role="status" aria-live="polite">
  {#each toasts as t (t.id)}<p class="toast fade-in">{t.text}</p>{/each}
</div>

<style>
  .toasts { position: fixed; left: 50%; bottom: 3rem; transform: translateX(-50%); z-index: 2000; display: flex; flex-direction: column;
    align-items: center; gap: 8px; pointer-events: none; width: max-content; max-width: calc(100vw - 2 * var(--pad)); }
  .toast { background: #000d; color: #fff; padding: 0.6em 1.1em; border-radius: 999px; box-shadow: 0 6px 20px #0008; text-align: center; }
</style>
