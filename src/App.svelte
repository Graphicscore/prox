<script lang="ts">
  import { onMount, tick } from "svelte";
  import { api } from "./lib/api";
  import Login from "./lib/Login.svelte";
  import Home from "./lib/Home.svelte";
  import Search from "./lib/Search.svelte";
  import Calendar from "./lib/Calendar.svelte";
  import Title from "./lib/Title.svelte";
  import Shell from "./lib/ui/Shell.svelte";
  import Loading from "./lib/ui/Loading.svelte";
  import { clearRecent } from "./lib/recent";
  import { exit, onBackButtonPress } from "@tauri-apps/api/app";
  import { currentTv } from "./lib/tv";
  import { handleBack, pushBack } from "./lib/back";
  import { installDpad } from "./lib/dpad";
  import { installTvInputs } from "./lib/tvInput";
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
</script>

{#if screen === "boot"}
  <Loading full label="Signing in…" />
{:else if screen === "login"}
  <Login onok={(w) => { warning = w ?? ""; screen = "home"; }} />
{:else}
  <Shell tab={screen === "info" ? null : screen} onnav={(t) => (screen = t)} onlogout={logout}>
    {#if warning}<p role="status" class="warn">{warning}</p>{/if}
    {#if seen.home}<div class="view" hidden={screen !== "home"}><Home active={screen === "home"} onopen={open} onauth={auth} /></div>{/if}
    {#if seen.search}<div class="view" hidden={screen !== "search"}><Search onopen={open} onauth={auth} /></div>{/if}
    {#if seen.calendar}<div class="view" hidden={screen !== "calendar"}><Calendar onopen={open} onauth={auth} /></div>{/if}
    {#if screen === "info"}<div class="view"><Title {id} onback={back} onauth={auth} /></div>{/if}
  </Shell>
{/if}

<style>
  .warn { padding: 8px var(--pad); color: var(--muted); }
</style>
