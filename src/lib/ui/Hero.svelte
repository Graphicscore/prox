<script lang="ts">
  import type { Snippet } from "svelte";
  import Cover from "./Cover.svelte";
  import { coverUrl } from "./helpers";
  let { id, title, meta = "", children }: { id: number; title: string; meta?: string; children?: Snippet } = $props();
</script>

<header class="hero">
  <div class="bg" style:background-image={`url(${coverUrl(id)})`}></div>
  <div class="fade"></div>
  <div class="in">
    <div class="p"><Cover {id} {title} /></div>
    <div class="txt">
      <h1>{title}</h1>
      {#if meta}<p class="meta">{meta}</p>{/if}
      <div class="actions">{@render children?.()}</div>
    </div>
  </div>
</header>

<style>
  .hero { position: relative; min-height: var(--hero-h); overflow: hidden; display: flex; align-items: flex-end; }
  .bg { position: absolute; inset: -30px; background-size: cover; background-position: center; filter: blur(18px) brightness(0.45) saturate(1.2); transform: scale(1.1); }
  .fade { position: absolute; inset: 0; background: linear-gradient(to top, var(--bg) 4%, transparent 65%); }
  .in { position: relative; flex: 1; min-width: 0; padding: 70px var(--pad) 16px; display: flex; gap: 18px; align-items: flex-end; }
  .p { flex: 0 0 var(--hero-poster-w); height: calc(var(--hero-poster-w) * 1.4); box-shadow: 0 6px 18px #0008; border-radius: var(--radius); }
  .txt { min-width: 0; }
  h1 { overflow-wrap: anywhere; font-size: 1.6rem; color: var(--text-hi); }
  .meta { color: var(--text); opacity: 0.8; margin: 4px 0 10px; }
  .actions { display: flex; flex-wrap: wrap; gap: 8px; }
</style>
