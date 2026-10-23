///<reference types="svelte" />
;
import { onDestroy, onMount, tick, untrack } from "svelte";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { pushBack } from "./back";
import { currentTv } from "./tv";
import { fmtTime, nearEnd } from "./ui/helpers";
import { setDpadRoot, setFullscreenMode, setSeekMode } from "./dpad";
import { api, describeError, safeEmbedUrl, streamLabel, type MediaSource, type Stream } from "./api";

;type $$ComponentProps =  {
    stream: Stream; onback: () => void; onauth: () => void; hasNext?: boolean; onnext?: () => unknown; onwatched?: () => void;
  };function $$render() {

  
  
  
  
  
  
  
  let { stream, onback, onauth, hasNext = false, onnext = () => {}, onwatched = () => {} }:/*Ωignore_startΩ*/$$ComponentProps/*Ωignore_endΩ*/ = $props();
  let near = $state(false), nextBtn = $state<HTMLButtonElement | null>(null);
  // Fullscreen: focus sits on the player frame, not the <video> (a focused video keeps the native controls and an
  // outline on screen). Our own overlay (top, clear of the native bar) shows time and progress for a moment after a seek.
  let osd = $state(false), ct = $state(0), dur = $state(0), paused = $state(false), osdTimer: ReturnType<typeof setTimeout> | undefined;
  function poke() { osd = true; clearTimeout(osdTimer); osdTimer = setTimeout(() => (osd = false), 2500); }
  const main = () => (full ? section : video); // where focus belongs while watching
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
    ct = video.currentTime; dur = video.duration;
    const n = nearEnd(video.currentTime, video.duration), a = document.activeElement;
    if (n && !near && (a === video || a === section)) tick().then(() => nextBtn?.focus());
    if (!n && near && a === nextBtn) main()?.focus({ preventScroll: true }); // seeked back: the pop-up goes, focus must not
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
    if (a === document.body || a === closeBtn) main()?.focus({ preventScroll: true });
  }
  // Own fullscreen: the player overlay fills the window and the app window goes fullscreen. WebKitGTK's native
  // video fullscreen renders black on some NVIDIA/Wayland setups. Outside Tauri windows the overlay alone still works.
  async function setFull(on: boolean) {
    full = on;
    tick().then(() => { // move focus between frame and video so the native controls can hide in fullscreen
      const a = document.activeElement;
      if (on && (a === video || a === document.body)) section?.focus({ preventScroll: true });
      else if (!on && a === section) video?.focus({ preventScroll: true });
    });
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
    else if ((e.key === " " && (tag !== "BUTTON" || e.target === nextBtn)) || (e.key === "Enter" && (tag === "VIDEO" || e.target === section))) { if (video.paused) video.play().catch(() => {}); else video.pause(); e.preventDefault(); }
    if (full && e.defaultPrevented && (e.key === "ArrowLeft" || e.key === "ArrowRight")) poke(); // pause shows the native controls itself
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
    clearTimeout(osdTimer);
    release();
    const a = document.activeElement; // only if focus was in the player (or lost), not when the user picked another tile
    if (opener?.isConnected && (a === document.body || !a || section?.contains(a))) opener.focus();
    setDpadRoot(null);
    setFullscreenMode(false);
    setSeekMode(false);
    if (full) getCurrentWindow().setFullscreen(false).catch(() => {}); // never leave the window fullscreen behind
  });
;
async () => {

 { svelteHTML.createElement("svelte:window", {  "onkeydown":key,});}

 { const $$_section0 = svelteHTML.createElement("section", {     "aria-label":`Player`,"tabindex":-1,});section = $$_section0;full;
  if(!full){
     { svelteHTML.createElement("div", { "class":`bar`,});
       { const $$_button2 = svelteHTML.createElement("button", {   "onclick":onback,});closeBtn = $$_button2;  }
       { svelteHTML.createElement("button", {   "aria-pressed":full,"onclick":() => setFull(!full),});  }
      if(hasNext){ { svelteHTML.createElement("button", { "onclick":goNext,});  }}
       { svelteHTML.createElement("span", {});stream.name; streamLabel(stream);  }
     }
  }
  if(error){ { svelteHTML.createElement("p", { "role":`alert`,});error; }}
  if(!embed){ { svelteHTML.createElement("p", { "role":`alert`,});       }}
  if(frame && embed){
     { svelteHTML.createElement("iframe", {       "class":`media frame`,"title":`Embedded player`,"src":embed,"allowfullscreen":true,"sandbox":`allow-scripts allow-same-origin allow-presentation`,}); }
  } else if (src){
    
    
     { const $$_video1 = svelteHTML.createElement("video", {                       "class":`media`,"tabindex":full ? -1 : 0,"src":src.url,"controls":true,"controlslist":`nofullscreen`,"autoplay":true,"playsinline":true,"ontimeupdate":ontime,onended,"onerror":onVideoError,"onplaying":() => { retried = false; focusVideoOnce(); },"onplay":(e) => { if (e.currentTarget === video) { started = true; paused = false; } },"onpause":(e) => { if (e.currentTarget === video) { started = true; paused = true; } },"oncanplay":() => { if (!started) { started = true; video?.play().catch(() => {}); } },"onloadedmetadata":() => { if (at && video) video.currentTime = at; at = 0; },});video = $$_video1; }
  } else if (!frame && !error){
     { svelteHTML.createElement("p", { "role":`status`,});  }
  }
  if(full && src && !frame && osd){
     { svelteHTML.createElement("div", {   "class":`osd fade-in`,"aria-hidden":`true`,}); { svelteHTML.createElement("span", {});paused ? "❚❚" : "▶"; } { svelteHTML.createElement("span", {});fmtTime(ct);  fmtTime(dur); } { svelteHTML.createElement("span", { "class":`track`,}); { svelteHTML.createElement("i", { "style":`width: ${dur ? (ct / dur) * 100 : 0}%`,}); } } }
  }
  if(near && hasNext && src && !frame){ { const $$_button1 = svelteHTML.createElement("button", {    "class":`next-pop primary`,"onclick":goNext,});nextBtn = $$_button1;   }}
  if(!frame && (!full || error)){ { svelteHTML.createElement("button", {   "disabled":!embed,"onclick":() => (frame = true),});    }}
 }


};
return { props: {} as any as $$ComponentProps, exports: {}, bindings: __sveltets_$$bindings(''), slots: {}, events: {} }}
const Player__SvelteComponent_ = __sveltets_2_fn_component($$render());
/*Ωignore_startΩ*/type Player__SvelteComponent_ = ReturnType<typeof Player__SvelteComponent_>;
/*Ωignore_endΩ*/export default Player__SvelteComponent_;