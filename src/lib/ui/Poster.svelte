<script lang="ts">
  import Cover from "./Cover.svelte";
  let { id, title, rating = null, onclick }: { id: number; title: string; rating?: number | string | null; onclick: () => void } = $props();
</script>

<button class="poster" data-id={id} {onclick} aria-label={title}>
  <span class="img"><Cover {id} {title} />{#if rating}<span class="badge">{rating}</span>{/if}</span>
  <span class="t">{title}</span>
</button>

<style>
  .poster { display: block; width: var(--poster-w); padding: 0; background: none; border-radius: var(--radius); text-align: left; color: var(--text); }
  .img { transition: transform 0.15s ease; position: relative; display: block; width: var(--poster-w); height: var(--poster-h); }
  .badge { position: absolute; right: 6px; bottom: 6px; background: #000a; color: #fff; border-radius: 10px; padding: 1px 7px; font-size: 0.75rem; }
  .t { display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; margin-top: 6px; font-size: 0.8rem; line-height: 1.25; min-height: 2.5em; }
  .poster:focus-visible .img { transform: scale(1.04); }
  @media (prefers-reduced-motion: reduce) { .poster:focus-visible .img { transform: none; } }
</style>
