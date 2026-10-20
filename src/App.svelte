<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./lib/api";
  import Login from "./lib/Login.svelte";
  import Search from "./lib/Search.svelte";
  import Info from "./lib/Info.svelte";
  import { exit, onBackButtonPress } from "@tauri-apps/api/app";
  import { currentTv } from "./lib/tv";
  import { handleBack } from "./lib/back";
  import { installDpad } from "./lib/dpad";
  let screen = $state<"boot" | "login" | "search" | "info">("boot"), id = $state(0), warning = $state("");
  onMount(() => {
    document.documentElement.classList.toggle("tv", currentTv());
    const offDpad = installDpad();
    let offBack: (() => void) | undefined, gone = false;
    // Android Back: close the top-most layer, else leave the app. Not available on desktop: ignore.
    onBackButtonPress(() => { if (!handleBack()) exit(0).catch(() => {}); })
      .then((l) => { if (gone) void l.unregister(); else offBack = () => void l.unregister(); })
      .catch(() => {});
    (async () => { screen = (await api.autoLogin().catch(() => false)) ? "search" : "login"; })();
    return () => { gone = true; offDpad(); offBack?.(); };
  });
  async function logout() { await api.logout(); warning = ""; screen = "login"; }
</script>

{#if screen === "login"}
  <Login onok={(w) => { warning = w ?? ""; screen = "search"; }} />
{:else if screen !== "boot"}
  <nav><button onclick={() => (screen = "search")}>Search</button> <button onclick={logout}>Log out</button></nav>
  {#if warning}<p role="status">{warning}</p>{/if}
  {#if screen === "search"}
    <Search onopen={(i) => { id = i; screen = "info"; }} onauth={() => (screen = "login")} />
  {:else}
    <Info {id} onback={() => (screen = "search")} onauth={() => (screen = "login")} />
  {/if}
{/if}

<style>
  :global(body) { background: #111; color: #eee; font-family: system-ui, sans-serif; margin: 1rem; }
  :global(button:focus-visible, input:focus-visible, a:focus-visible) { outline: 3px solid #4af; outline-offset: 2px; }
  :global(html.tv) { font-size: 24px; }
  :global(html.tv body) { margin: 5vh 5vw; }
  :global(html.tv button:focus-visible, html.tv input:focus-visible) { outline-width: 5px; }
  @media (pointer: coarse) { :global(button), :global(input:not([type="checkbox"]):not([type="radio"])) { min-height: 44px; } }
  :global(form label) { display: block; margin: 0.5rem 0; }
</style>
