///<reference types="svelte" />
;
import { onMount, tick } from "svelte";
import { api, describeError } from "./lib/api";
import Login from "./lib/Login.svelte";
import Home from "./lib/Home.svelte";
import Search from "./lib/Search.svelte";
import Calendar from "./lib/Calendar.svelte";
import Title from "./lib/Title.svelte";
import Shell from "./lib/ui/Shell.svelte";
import Loading from "./lib/ui/Loading.svelte";
import Toasts from "./lib/ui/Toasts.svelte";
import { toast } from "./lib/toast.svelte";
import { clearRecent } from "./lib/recent";
import { exit, getVersion, onBackButtonPress } from "@tauri-apps/api/app";
import { versionLabel } from "./lib/ui/helpers";
import { currentTv } from "./lib/tv";
import { handleBack, pushBack } from "./lib/back";
import { installDpad } from "./lib/dpad";
import { installTvInputs } from "./lib/tvInput";
function $$render() {

  
  
  
  
  
  
  
  
  
  
  
  
  
  
  
  
  
  
  type Screen = "boot" | "login" | "home" | "search" | "calendar" | "info";
  let screen = $state<Screen>("boot"), from = $state<"home" | "search" | "calendar">("home"), id = $state(0), warning = $state("");
  // Home and Search stay mounted (hidden) while a title is open: Back keeps their data, scroll and focused poster.
  let seen = $state({ home: false, search: false, calendar: false }), returnFocus: HTMLElement | null = null;
  $effect(() => { if (screen === "home" || screen === "search" || screen === "calendar") seen[screen] = true; });
  const startScreen = (): Screen => {
    if (import.meta.env.DEV) { // dev only: ?screen=home|search|title&id=N for screenshots in the demo mode
      const q = new URLSearchParams(location.search);
      const s = q.get("screen");
      if (q.get("id")) id = Number(q.get("id"));
      if (s === "search") return "search";
      if (s === "calendar") return "calendar";
      if (s === "title") return "info";
    }
    return "home";
  };
  onMount(() => {
    document.documentElement.classList.toggle("tv", currentTv());
    const offDpad = installDpad();
    const offInputs = currentTv() ? installTvInputs() : () => {};
    let offBack: (() => void) | undefined, gone = false;
    // Android Back: close the top-most layer, else leave the app. Not available on desktop: ignore.
    onBackButtonPress(() => { if (!handleBack()) exit(0).catch(() => {}); })
      .then((l) => { if (gone) void l.unregister(); else offBack = () => void l.unregister(); })
      .catch(() => {});
    // Desktop Back: Esc (outside text fields), Alt+Left, the browser/mouse back button.
    const onKey = (e: KeyboardEvent) => {
      if (e.defaultPrevented || !(e.key === "Escape" || e.key === "BrowserBack" || (e.altKey && e.key === "ArrowLeft"))) return;
      const t = e.target as HTMLElement | null;
      if (e.key === "Escape" && (t?.tagName === "INPUT" || t?.tagName === "TEXTAREA")) { e.preventDefault(); return; } // Esc in a field is not Back
      if (handleBack()) e.preventDefault();
    };
    const onMouse = (e: MouseEvent) => { if (e.button === 3 && handleBack()) e.preventDefault(); };
    window.addEventListener("keydown", onKey);
    window.addEventListener("mouseup", onMouse);
    // Bottom of the Back stack: Search/Calendar go back to Home; Home leaves the app.
    const offTabs = pushBack(() => {
      if (screen !== "search" && screen !== "calendar") return false;
      screen = "home";
      tick().then(() => { const a = document.activeElement; if (!a || a === document.body || !(a as HTMLElement).offsetParent) document.querySelector<HTMLElement>("header .tab")?.focus(); });
      return true;
    });
    (async () => {
      const ok = await api.autoLogin().catch(() => false);
      screen = ok && !(import.meta.env.DEV && new URLSearchParams(location.search).get("screen") === "login") ? startScreen() : "login"; // dev: ?screen=login shows Login
    })();
    return () => { gone = true; offDpad(); offInputs(); offBack?.(); offTabs(); window.removeEventListener("keydown", onKey); window.removeEventListener("mouseup", onMouse); };
  });
  async function logout() { await api.logout(); clearRecent(); warning = ""; seen = { home: false, search: false, calendar: false }; screen = "login"; }
  const open = (i: number) => {
    returnFocus = document.activeElement as HTMLElement | null;
    from = screen === "search" || screen === "calendar" ? screen : "home"; id = i; screen = "info";
  };
  async function back() { screen = from; await tick(); if (returnFocus?.isConnected) returnFocus.focus(); }
  const auth = () => (screen = "login");
  // Update button: check the latest GitHub release; download it and hand it to the installer (Android) or swap
  // the AppImage and restart (Linux). Without a self-install path (deb, dev build) point to the release page.
  let updating = $state(false), version = $state("");
  getVersion().then((v) => (version = versionLabel(v))).catch(() => {}); // no Tauri (demo in a browser): no label
  async function update() {
    if (updating) return;
    updating = true;
    try {
      const u = await api.checkUpdate();
      if (!u) { toast("You have the latest build"); return; }
      if (!u.url) { toast(`Build ${u.build} is available: ${u.page}`, 8000); return; }
      toast(`Downloading build ${u.build}…`);
      const path = await api.downloadUpdate(u.url);
      const bridge = (window as unknown as { AndroidTv?: { installUpdate?: (p: string) => string } }).AndroidTv;
      if (!path || !bridge?.installUpdate) return; // Linux: the app restarts itself
      const r = bridge.installUpdate(path);
      if (r === "permission") toast("Allow \"Install unknown apps\" for Proxer, then press update again", 8000);
      else if (r !== "started") toast(`Update could not be installed (${r})`, 6000);
    } catch (x) { toast(describeError(x), 6000); }
    finally { updating = false; }
  }
;
async () => {

if(screen === "boot"){
   { const $$_gnidaoL0C = __sveltets_2_ensureComponent(Loading); new $$_gnidaoL0C({ target: __sveltets_2_any(), props: {  "full":true,"label":`Signing in…`,}});}
} else if (screen === "login"){
   { const $$_nigoL0C = __sveltets_2_ensureComponent(Login); new $$_nigoL0C({ target: __sveltets_2_any(), props: {      "onok":(w) => { warning = w ?? ""; screen = "home"; },"onupdate":update,updating,version,}});}
}else{
   { const $$_llehS0C = __sveltets_2_ensureComponent(Shell); new $$_llehS0C({ target: __sveltets_2_any(), props: {         children:() => { return __sveltets_2_any(0); },"tab":screen === "info" ? null : screen,"onnav":(t) => (screen = t),"onlogout":logout,"onupdate":update,updating,version,}});
    if(warning){ { svelteHTML.createElement("p", {   "role":`status`,"class":`warn`,});warning; }}
    if(seen.home){ { svelteHTML.createElement("div", {   "class":`view`,"hidden":screen !== "home",}); { const $$_emoH2C = __sveltets_2_ensureComponent(Home); new $$_emoH2C({ target: __sveltets_2_any(), props: {      "active":screen === "home","onopen":open,"onauth":auth,}});} }}
    if(seen.search){ { svelteHTML.createElement("div", {   "class":`view`,"hidden":screen !== "search",}); { const $$_hcraeS2C = __sveltets_2_ensureComponent(Search); new $$_hcraeS2C({ target: __sveltets_2_any(), props: {    "onopen":open,"onauth":auth,}});} }}
    if(seen.calendar){ { svelteHTML.createElement("div", {   "class":`view`,"hidden":screen !== "calendar",}); { const $$_radnelaC2C = __sveltets_2_ensureComponent(Calendar); new $$_radnelaC2C({ target: __sveltets_2_any(), props: {    "onopen":open,"onauth":auth,}});} }}
    if(screen === "info"){ { svelteHTML.createElement("div", { "class":`view`,}); { const $$_eltiT2C = __sveltets_2_ensureComponent(Title); new $$_eltiT2C({ target: __sveltets_2_any(), props: {     id,"onback":back,"onauth":auth,}});} }}
   Shell}
}

 { const $$_stsaoT0C = __sveltets_2_ensureComponent(Toasts); new $$_stsaoT0C({ target: __sveltets_2_any(), props: {}});}


};
return { props: {} as Record<string, never>, exports: {}, bindings: __sveltets_$$bindings(''), slots: {}, events: {} }}
const App__SvelteComponent_ = __sveltets_2_fn_component($$render());
/*Ωignore_startΩ*/type App__SvelteComponent_ = ReturnType<typeof App__SvelteComponent_>;
/*Ωignore_endΩ*/export default App__SvelteComponent_;