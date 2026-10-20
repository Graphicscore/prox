<script lang="ts">
  let { label = "Loading…", full = false, compact = false }: { label?: string; full?: boolean; compact?: boolean } = $props();
</script>

<p class="loading" class:full class:compact role="status"><span class="spin" aria-hidden="true"></span>{label}</p>

<style>
  .loading { display: flex; align-items: center; gap: 0.6em; padding: 18px var(--pad); color: var(--muted); }
  .compact { padding: 6px 0; }
  .full { min-height: 70vh; justify-content: center; font-size: 1.1rem; }
  .spin { width: 1.1em; height: 1.1em; flex: none; border-radius: 50%; border: 0.18em solid var(--bg-4); border-top-color: var(--accent); animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
