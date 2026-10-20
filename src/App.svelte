<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./lib/api";
  import Login from "./lib/Login.svelte";
  import Search from "./lib/Search.svelte";
  import Info from "./lib/Info.svelte";
  let screen = $state<"boot" | "login" | "search" | "info">("boot"), id = $state(0), warning = $state("");
  onMount(async () => { screen = (await api.autoLogin().catch(() => false)) ? "search" : "login"; });
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
  :global(form label) { display: block; margin: 0.5rem 0; }
</style>
