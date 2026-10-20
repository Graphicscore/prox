<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./lib/api";
  import Login from "./lib/Login.svelte";
  import Home from "./lib/Home.svelte";
  import Search from "./lib/Search.svelte";
  import Title from "./lib/Title.svelte";
  import Shell from "./lib/ui/Shell.svelte";
  import { exit, onBackButtonPress } from "@tauri-apps/api/app";
  import { currentTv } from "./lib/tv";
  import { handleBack } from "./lib/back";
  import { installDpad } from "./lib/dpad";
  import { installTvInputs } from "./lib/tvInput";
  type Screen = "boot" | "login" | "home" | "search" | "info";
  let screen = $state<Screen>("boot"), from = $state<"home" | "search">("home"), id = $state(0), warning = $state("");
  const startScreen = (): Screen => {
    if (import.meta.env.DEV) { // dev only: ?screen=home|search|title&id=N for screenshots in the demo mode
      const q = new URLSearchParams(location.search);
      const s = q.get("screen");
      if (q.get("id")) id = Number(q.get("id"));
      if (s === "search") return "search";
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
    (async () => {
      const ok = await api.autoLogin().catch(() => false);
      screen = ok && !(import.meta.env.DEV && new URLSearchParams(location.search).get("screen") === "login") ? startScreen() : "login"; // dev: ?screen=login shows Login
    })();
    return () => { gone = true; offDpad(); offInputs(); offBack?.(); };
  });
  async function logout() { await api.logout(); warning = ""; screen = "login"; }
  const open = (i: number) => { from = screen === "search" ? "search" : "home"; id = i; screen = "info"; };
  const auth = () => (screen = "login");
</script>

{#if screen === "login"}
  <Login onok={(w) => { warning = w ?? ""; screen = "home"; }} />
{:else if screen !== "boot"}
  <Shell tab={screen === "info" ? null : screen} onnav={(t) => (screen = t)} onlogout={logout}>
    {#if warning}<p role="status" class="warn">{warning}</p>{/if}
    {#if screen === "home"}
      <Home onopen={open} onauth={auth} />
    {:else if screen === "search"}
      <Search onopen={open} onauth={auth} />
    {:else}
      <Title {id} onback={() => (screen = from)} onauth={auth} />
    {/if}
  </Shell>
{/if}

<style>
  .warn { padding: 8px var(--pad); color: var(--muted); }
</style>
