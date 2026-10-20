<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { pushBack } from "./back";
  import { setDpadRoot, setSeekMode } from "./dpad";
  import { api, describeError, safeEmbedUrl, streamLabel, type MediaSource, type Stream } from "./api";
  let { stream, onback, onauth }: { stream: Stream; onback: () => void; onauth: () => void } = $props();
  let src = $state<MediaSource | null>(null), error = $state(""), frame = $state(false), full = $state(false);
  let video = $state<HTMLVideoElement | null>(null);
  let section = $state<HTMLElement | null>(null), closeBtn = $state<HTMLButtonElement | null>(null);
  let retried = false, focusedVideo = false, alive = true, at = 0;
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
    if (!retried) { retried = true; focusedVideo = false; at = video?.currentTime ?? 0; src = null; resolve(); } // airing URLs expire: scrape again once
    else error = "Playback failed";
  }
  // Focus the video once per <video>, and only if the user has not moved focus to another control.
  function focusVideoOnce() {
    if (focusedVideo) return;
    focusedVideo = true;
    const a = document.activeElement;
    if (a === document.body || a === closeBtn) video?.focus();
  }
  // Own fullscreen: the player overlay fills the window and the app window goes fullscreen. WebKitGTK's native
  // video fullscreen renders black on some NVIDIA/Wayland setups. Outside Tauri windows the overlay alone still works.
  async function setFull(on: boolean) {
    full = on;
    try { await getCurrentWindow().setFullscreen(on); } catch { /* no window API (mobile/browser): overlay only */ }
  }
  function key(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement | null)?.tagName;
    if (e.key === "Escape") { if (full) setFull(false); else onback(); return; }
    if (tag === "INPUT") return;
    if (e.key === "f" || e.key === "F") { setFull(!full); e.preventDefault(); return; }
    if (!video) return;
    if (e.key === "ArrowLeft" || e.key === "ArrowRight") { if (tag !== "VIDEO" && !full) return; }
    if (e.key === "ArrowLeft") { video.currentTime = Math.max(0, video.currentTime - 10); e.preventDefault(); }
    else if (e.key === "ArrowRight") { video.currentTime += 10; e.preventDefault(); }
    else if ((e.key === " " && tag !== "BUTTON") || (e.key === "Enter" && tag === "VIDEO")) { if (video.paused) video.play().catch(() => {}); else video.pause(); e.preventDefault(); }
  }
  onMount(() => {
    closeBtn?.focus(); section?.scrollIntoView({ block: "center" }); resolve();
    setSeekMode(true);
    return pushBack(() => { if (full) setFull(false); else onback(); return true; });
  });
  $effect(() => { setDpadRoot(full ? section : null); });
  onDestroy(() => {
    alive = false;
    setDpadRoot(null);
    setSeekMode(false);
    if (full) getCurrentWindow().setFullscreen(false).catch(() => {}); // never leave the window fullscreen behind
  });
</script>

<svelte:window onkeydown={key} />

<section aria-label="Player" bind:this={section} class:full>
  {#if !full}
    <div class="bar">
      <button bind:this={closeBtn} onclick={onback}>Close player</button>
      <button aria-pressed={full} onclick={() => setFull(!full)}>Fullscreen</button>
      <span>{stream.name} [{streamLabel(stream)}]</span>
    </div>
  {/if}
  {#if error}<p role="alert">{error}</p>{/if}
  {#if !embed}<p role="alert">The embedded player address is not allowed.</p>{/if}
  {#if frame && embed}
    <iframe class="media frame" title="Embedded player" src={embed} allowfullscreen sandbox="allow-scripts allow-same-origin allow-presentation"></iframe>
  {:else if src}
    <!-- ponytail: src.mime unused, no HLS path; add hls.js only if a probe shows m3u8 sources -->
    <!-- svelte-ignore a11y_media_has_caption -->
    <video class="media" tabindex="0" bind:this={video} src={src.url} controls controlslist="nofullscreen" autoplay playsinline onerror={onVideoError} onplaying={() => { retried = false; focusVideoOnce(); }} onloadedmetadata={() => { if (at && video) video.currentTime = at; at = 0; }}></video>
  {:else if !frame && !error}
    <p role="status">Loading…</p>
  {/if}
  {#if !frame && !full}<button disabled={!embed} onclick={() => (frame = true)}>Use embedded player instead</button>{/if}
</section>

<style>
  .media { width: 100%; max-height: 80vh; }
  .frame { aspect-ratio: 16 / 9; border: 0; }
  .full { position: fixed; inset: 0; z-index: 1000; background: #000; display: flex; flex-direction: column; padding: 0.5rem; box-sizing: border-box; }
  .full .media { flex: 1; min-height: 0; max-height: none; height: 100%; }
  .full video { object-fit: contain; }
  .full .frame { aspect-ratio: auto; }
</style>
