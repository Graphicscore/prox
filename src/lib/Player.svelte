<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { api, describeError, safeEmbedUrl, streamLabel, type MediaSource, type Stream } from "./api";
  let { stream, onback, onauth }: { stream: Stream; onback: () => void; onauth: () => void } = $props();
  let src = $state<MediaSource | null>(null), error = $state(""), frame = $state(false);
  let video = $state<HTMLVideoElement | null>(null);
  let section = $state<HTMLElement | null>(null), closeBtn = $state<HTMLButtonElement | null>(null);
  let retried = false, alive = true, at = 0;
  const embed = $derived(safeEmbedUrl(stream));

  async function resolve() {
    error = "";
    try {
      const m = await api.resolve(stream);
      if (alive) src = m; // ponytail: expires_at could drive a proactive re-resolve before it passes
    } catch (x) {
      if (!alive) return;
      if (x === "NotLoggedIn") onauth(); else error = describeError(x);
    }
  }
  function onVideoError() {
    if (!retried) { retried = true; at = video?.currentTime ?? 0; src = null; resolve(); } // airing URLs expire: scrape again once
    else error = "Playback failed";
  }
  function key(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement | null)?.tagName;
    if (e.key === "Escape") { onback(); return; }
    if (!video || tag === "INPUT") return;
    if (e.key === "ArrowLeft") { video.currentTime = Math.max(0, video.currentTime - 10); e.preventDefault(); }
    else if (e.key === "ArrowRight") { video.currentTime += 10; e.preventDefault(); }
    else if (e.key === " " && tag !== "BUTTON") { if (video.paused) video.play().catch(() => {}); else video.pause(); e.preventDefault(); }
  }
  onMount(() => { closeBtn?.focus(); section?.scrollIntoView({ block: "center" }); resolve(); });
  onDestroy(() => { alive = false; });
</script>

<svelte:window onkeydown={key} />

<section aria-label="Player" bind:this={section}>
  <button bind:this={closeBtn} onclick={onback}>Close player</button>
  <span>{stream.name} [{streamLabel(stream)}]</span>
  {#if error}<p role="alert">{error}</p>{/if}
  {#if !embed}<p role="alert">The embedded player address is not allowed.</p>{/if}
  {#if frame && embed}
    <iframe title="Embedded player" src={embed} allowfullscreen sandbox="allow-scripts allow-same-origin allow-presentation" style="width:100%;aspect-ratio:16/9;border:0"></iframe>
  {:else if src}
    <!-- ponytail: src.mime unused, no HLS path; add hls.js only if a probe shows m3u8 sources -->
    <!-- svelte-ignore a11y_media_has_caption -->
    <video bind:this={video} src={src.url} controls autoplay playsinline onerror={onVideoError} onplaying={() => (retried = false)} onloadedmetadata={() => { if (at && video) video.currentTime = at; at = 0; }} style="width:100%;max-height:80vh"></video>
  {:else if !frame && !error}
    <p role="status">Loading…</p>
  {/if}
  {#if !frame}<button disabled={!embed} onclick={() => (frame = true)}>Use embedded player instead</button>{/if}
</section>
