<script lang="ts">
  import { api, describeError } from "./api";
  let { onok, onupdate, updating = false }: { onok: (warning?: string) => void; onupdate?: () => void; updating?: boolean } = $props();
  let username = $state(""), password = $state(""), remember = $state(true), error = $state(""), busy = $state(false);
  async function submit(e: Event) {
    e.preventDefault();
    if (busy) return; // the button stays enabled (TV focus): ignore repeats
    // TV input lock makes fields read-only, which skips `required`: check explicitly.
    if (!username.trim() || !password) { error = "Enter your username and password"; return; }
    busy = true; error = "";
    try {
      const saved = await api.login(username, password, remember);
      onok(remember && !saved ? "Logged in, but your password could not be saved to the system keychain" : undefined);
    } catch (x) { error = String(x).startsWith("Parse:") ? "Proxer changed something — the login page looks different" : describeError(x) === "Something went wrong" ? "Login failed" : describeError(x); }
    busy = false;
  }
</script>

<div class="wrap">
  <form class="card view" onsubmit={submit}>
    <h1>Proxer</h1>
    <p class="sub">Sign in with your Proxer account</p>
    <label>Username <input bind:value={username} autocomplete="username" required /></label>
    <label>Password <input bind:value={password} type="password" autocomplete="current-password" required /></label>
    <label class="check"><input type="checkbox" bind:checked={remember} /> Remember me</label>
    {#if error}<p role="alert">{error}</p>{/if}
    <button class="primary">{busy ? "Signing in…" : "Log in"}</button>
    {#if onupdate}<button type="button" class="ghost update" onclick={onupdate}>{updating ? "Checking…" : "Check for updates"}</button>{/if}
  </form>
</div>

<style>
  .wrap { min-height: 100vh; display: flex; align-items: center; justify-content: center; padding: 4vh var(--pad); box-sizing: border-box;
    background: radial-gradient(ellipse at 50% 0%, #1d2b4d 0%, var(--bg) 65%); }
  .card { width: 100%; max-width: 26rem; box-sizing: border-box; padding: 2rem; border-radius: calc(var(--radius) * 2);
    background: var(--bg-2); box-shadow: 0 20px 60px #0009; }
  h1 { font-size: 2rem; color: var(--text-hi); }
  .sub { color: var(--muted); margin: 0.3rem 0 1.4rem; }
  label { margin: 1rem 0; color: var(--muted); font-size: 0.9rem; }
  label input:not([type="checkbox"]) { display: block; width: 100%; box-sizing: border-box; margin-top: 6px; font-size: 1rem; }
  .check { display: flex; align-items: center; gap: 0.5rem; color: var(--text); font-size: 1rem; }
  .check input { width: 1.1em; height: 1.1em; margin: 0; accent-color: var(--accent); }
  [role="alert"] { margin: 0.75rem 0; padding: 0.6em 0.8em; border-radius: var(--radius); background: #d9534f22; color: #ff8a86; }
  button { width: 100%; margin-top: 0.75rem; padding: 0.7em 1em; }
  .update { margin-top: 0.4rem; color: var(--muted); font-size: 0.9rem; }
  @media (max-width: 480px) { .card { padding: 1.4rem; } }
  /* TV: 24px base on a 540px-high viewport; keep the whole card on screen */
  :global(html.tv) .card { padding: 1rem 1.5rem; max-width: 24rem; }
  :global(html.tv) h1 { font-size: 1.5rem; }
  :global(html.tv) .sub { margin-bottom: 0.6rem; }
  :global(html.tv) label { margin: 0.55rem 0; }
  :global(html.tv) .wrap { padding: 2vh var(--pad); }
</style>
