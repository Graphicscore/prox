<script lang="ts">
  import { api, describeError } from "./api";
  let { onok }: { onok: (warning?: string) => void } = $props();
  let username = $state(""), password = $state(""), remember = $state(true), error = $state(""), busy = $state(false);
  async function submit(e: Event) {
    e.preventDefault();
    // TV input lock makes fields read-only, which skips `required`: check explicitly.
    if (!username.trim() || !password) { error = "Enter your username and password"; return; }
    busy = true; error = "";
    try {
      const saved = await api.login(username, password, remember);
      onok(remember && !saved ? "Logged in, but your password could not be saved to the system keychain" : undefined);
    } catch (x) { error = String(x).startsWith("Parse:") ? "Proxer changed something \u2014 the login page looks different" : describeError(x) === "Something went wrong" ? "Login failed" : describeError(x); }
    busy = false;
  }
</script>

<form onsubmit={submit}>
  <h1>Proxer</h1>
  <label>Username <input bind:value={username} autocomplete="username" required /></label>
  <label>Password <input bind:value={password} type="password" autocomplete="current-password" required /></label>
  <label><input type="checkbox" bind:checked={remember} /> Remember me</label>
  {#if error}<p role="alert">{error}</p>{/if}
  <button disabled={busy}>Log in</button>
</form>
