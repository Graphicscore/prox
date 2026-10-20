<script lang="ts">
  import type { Snippet } from "svelte";
  let { tab, onnav, onlogout, onupdate, updating = false, version = "", children }: {
    tab: "home" | "search" | "calendar" | null; onnav: (t: "home" | "search" | "calendar") => void; onlogout: () => void; onupdate: () => void; updating?: boolean; version?: string; children?: Snippet;
  } = $props();
</script>

<header class="top">
  <strong class="logo">Proxer</strong>
  <button class="tab" aria-current={tab === "home" ? "page" : undefined} onclick={() => onnav("home")}>Home</button>
  <button class="tab" aria-current={tab === "search" ? "page" : undefined} onclick={() => onnav("search")}>Search</button>
  <button class="tab" aria-current={tab === "calendar" ? "page" : undefined} onclick={() => onnav("calendar")}>Calendar</button>
  <span class="grow"></span>
  {#if version}<span class="ver">{version}</span>{/if}
  <button class="ghost icon" class:spin={updating} onclick={onupdate} title="Check for updates" aria-label="Check for updates">
    <svg viewBox="0 0 24 24" width="1.2em" height="1.2em" aria-hidden="true"><path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" d="M20 12a8 8 0 1 1-2.34-5.66M20 4v4h-4"/></svg>
  </button>
  <button class="ghost" onclick={onlogout}>Log out</button>
</header>
<main>{@render children?.()}</main>
{#if version}<footer class="foot">{version}</footer>{/if}

<style>
  .top { display: flex; align-items: center; gap: 10px; padding: 12px var(--pad); background: var(--bg-2); position: sticky; top: 0; z-index: 5; }
  .logo { color: var(--text-hi); font-size: 1.1rem; margin-right: 12px; }
  .tab { background: transparent; border-radius: 0; padding: 0.3em 0.2em; margin-right: 8px; border-bottom: 2px solid transparent; }
  .tab[aria-current="page"] { border-bottom-color: var(--accent); }
  .grow { flex: 1; }
  main { padding-bottom: 40px; }
  .ver { color: var(--muted); font-size: 0.75rem; white-space: nowrap; }
  @media (max-width: 600px) { .ver { display: none; } } /* phones: only the footer line */
  .foot { padding: 0 var(--pad) 24px; color: var(--muted); font-size: 0.75rem; text-align: center; }
  .icon { display: inline-flex; align-items: center; padding: 0.45em 0.6em; }
  .spin svg { animation: spin 1s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
