<script lang="ts">
  import { onDestroy, onMount, tick, untrack } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { pushBack } from "./back";
  import { currentTv } from "./tv";
  import { nearEnd } from "./ui/helpers";
  import { setDpadRoot, setFullscreenMode, setSeekMode } from "./dpad";
  import { api, describeError, safeEmbedUrl, streamLabel, type MediaSource, type Stream } from "./api";
  let { stream, onback, onauth, hasNext = false, onnext = () => {}, onwatched = () => {} }: {
    stream: Stream; onback: () => void; onauth: () => void; hasNext?: boolean; onnext?: () => unknown; onwatched?: () => void;
  } = $props();
  let near = $state(false), nextBtn = $state<HTMLButtonElement | null>(null);
  let src = $state<MediaSource | null>(null), error = $state(""), frame = $state(false), full = $state(false);
  const tv = currentTv(); // TV: the player opens fullscreen and Back closes it (an inline player sits below the fold there)
  let video = $state<HTMLVideoElement | null>(null);
  let section = $state<HTMLElement | null>(null), closeBtn = $state<HTMLButtonElement | null>(null);
  let retried = false, focusedVideo = false, alive = true, at = 0, going = false, started = false;
  const embed = $derived(safeEmbedUrl(stream));
  const opener = document.activeElement as HTMLElement | null; // the stream/Watch button: gets focus back on close

  async function resolve() {
    error = "";
    const s = stream;
    try {
      const m = await api.resolve(s);
      if (alive && s === stream) src = m; // ponytail: expires_at could drive a proactive re-resolve before it passes
    } catch (x) {
      if (!alive || s !== stream) return;
      if (x === "NotLoggedIn") onauth(); else error = describeError(x);
    }
  }
  let shown: Stream | null = null;
  $effect(() => { // first mount and every Next: same instance, so fullscreen and focus scope survive
    if (stream === shown) return;
    shown = stream;
    untrack(() => { release(); src = null; error = ""; frame = false; retried = false; focusedVideo = false; at = 0; near = false; started = false; resolve(); });
  });
  function ontime() {
    if (!video) return;
    const n = nearEnd(video.currentTime, video.duration);
    if (n && !near && document.activeElement === video) tick().then(() => nextBtn?.focus());
    if (!n && near && document.activeElement === nextBtn) video.focus({ preventScroll: true }); // seeked back: the pop-up goes, focus must not
    near = n;
  }
  /** One switch at a time: Next can be pressed in the bar and the pop-up, and the video can end meanwhile. */
  async function advance(watched: boolean, next: boolean) {
    if (going) return;
    going = true;
    try { if (watched) onwatched(); if (next) await onnext(); } finally { going = false; }
  }
  const goNext = () => advance(near && !frame, true);
  const onended = () => advance(true, hasNext);
  function onVideoError() {
    if (!retried) { retried = true; focusedVideo = false; started = false; at = video?.currentTime ?? 0; release(); src = null; resolve(); } // airing URLs expire: scrape again once
    else error = "Playback failed";
  }
  // Android WebView: a removed <video> keeps its connection to the stream host and stalls the next one. Free it first.
  function release() { if (!video) return; video.pause(); video.removeAttribute("src"); video.load(); }
  // Focus the video once per <video>, and only if the user has not moved focus to another control.
  function focusVideoOnce() {
    if (focusedVideo) return;
    focusedVideo = true;
    const a = document.activeElement;
    if (a === document.body || a === closeBtn) video?.focus({ preventScroll: true });
  }
  // Own fullscreen: the player overlay fills the window and the app window goes fullscreen. WebKitGTK's native
  // video fullscreen renders black on some NVIDIA/Wayland setups. Outside Tauri windows the overlay alone still works.
  async function setFull(on: boolean) {
    full = on;
    try { await getCurrentWindow().setFullscreen(on); } catch { /* no window API (mobile/browser): overlay only */ }
  }
  function key(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement | null)?.tagName;
    if (tag === "INPUT") return;
    if (e.key === "f" || e.key === "F") { setFull(!full); e.preventDefault(); return; }
    if (!video) return;
    if (e.key === "ArrowLeft" || e.key === "ArrowRight") { if (tag !== "VIDEO" && !full) return; }
    if (e.key === "ArrowLeft") { video.currentTime = Math.max(0, video.currentTime - 10); e.preventDefault(); }
    else if (e.key === "ArrowRight") { video.currentTime += 10; e.preventDefault(); }
    else if ((e.key === " " && (tag !== "BUTTON" || e.target === nextBtn)) || (e.key === "Enter" && tag === "VIDEO")) { if (video.paused) video.play().catch(() => {}); else video.pause(); e.preventDefault(); }
  }
  onMount(() => {
    closeBtn?.focus(); section?.scrollIntoView({ block: "start" }); // start honors scroll-padding: the toolbar stays below the sticky top bar
    setSeekMode(true);
    if (tv) setFull(true);
    return pushBack(() => { if (full && !tv) setFull(false); else onback(); return true; });
  });
  $effect(() => { setDpadRoot(section); setFullscreenMode(full); }); // the open player keeps D-pad focus to itself
  onDestroy(() => {
    alive = false;
    release();
    const a = document.activeElement; // only if focus was in the player (or lost), not when the user picked another tile
    if (opener?.isConnected && (a === document.body || !a || section?.contains(a))) opener.focus();
    setDpadRoot(null);
    setFullscreenMode(false);
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
      {#if hasNext}<button onclick={goNext}>Next episode</button>{/if}
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
    <video class="media" tabindex="0" bind:this={video} src={src.url} controls controlslist="nofullscreen" autoplay playsinline ontimeupdate={ontime} {onended} onerror={onVideoError} onplaying={() => { retried = false; focusVideoOnce(); }} onplay={(e) => { if (e.currentTarget === video) started = true; }} onpause={(e) => { if (e.currentTarget === video) started = true; }} oncanplay={() => { if (!started) { started = true; video?.play().catch(() => {}); } }} onloadedmetadata={() => { if (at && video) video.currentTime = at; at = 0; }}></video>
  {:else if !frame && !error}
    <p role="status">Loading…</p>
  {/if}
  {#if near && hasNext && src && !frame}<button class="next-pop primary" bind:this={nextBtn} onclick={goNext}>▶ Next episode</button>{/if}
  {#if !frame && (!full || error)}<button disabled={!embed} onclick={() => (frame = true)}>Use embedded player instead</button>{/if}
</section>

<style>
  section { position: relative; }
  .next-pop { position: absolute; right: 1.5rem; bottom: 4.5rem; z-index: 2; }
  section:not(.full) { margin: 18px var(--pad); padding: 12px; background: var(--bg-2); border-radius: var(--radius); }
  .bar { display: flex; gap: 8px; align-items: center; margin-bottom: 8px; flex-wrap: wrap; }
  .media { width: 100%; max-height: 80vh; }
  .frame { aspect-ratio: 16 / 9; border: 0; }
  .full { position: fixed; inset: 0; z-index: 1000; background: #000; display: flex; flex-direction: column; padding: 0.5rem; box-sizing: border-box; }
  .full .media { flex: 1; min-height: 0; max-height: none; height: 100%; }
  .full video { object-fit: contain; }
  .full .frame { aspect-ratio: auto; }
</style>
