<script lang="ts">
  import type { Snippet } from "svelte";
  let { tab, onnav, onlogout, children }: {
    tab: "home" | "search" | null; onnav: (t: "home" | "search") => void; onlogout: () => void; children?: Snippet;
  } = $props();
</script>

<header class="top">
  <strong class="logo">Proxer</strong>
  <button class="tab" aria-current={tab === "home" ? "page" : undefined} onclick={() => onnav("home")}>Home</button>
  <button class="tab" aria-current={tab === "search" ? "page" : undefined} onclick={() => onnav("search")}>Search</button>
  <span class="grow"></span>
  <button class="ghost" onclick={onlogout}>Log out</button>
</header>
<main>{@render children?.()}</main>

<style>
  .top { display: flex; align-items: center; gap: 10px; padding: 12px var(--pad); background: var(--bg-2); position: sticky; top: 0; z-index: 5; }
  .logo { color: var(--text-hi); font-size: 1.1rem; margin-right: 12px; }
  .tab { background: transparent; border-radius: 0; padding: 0.3em 0.2em; margin-right: 8px; border-bottom: 2px solid transparent; }
  .tab[aria-current="page"] { border-bottom-color: var(--accent); }
  .grow { flex: 1; }
  main { padding-bottom: 40px; }
</style>
