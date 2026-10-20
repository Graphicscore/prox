<script lang="ts">
  import { onMount } from "svelte";
  import { api, describeError, type CalendarEntry } from "./api";
  import { calendarDays } from "./ui/helpers";
  import PosterRow from "./ui/PosterRow.svelte";
  let { onopen, onauth }: { onopen: (id: number) => void; onauth: () => void } = $props();
  let entries = $state<CalendarEntry[]>([]), status = $state<"loading" | "ready" | "error">("loading"), error = $state("");
  const days = $derived(calendarDays(entries, new Date()));
  async function load() {
    status = "loading"; error = "";
    try { entries = await api.getCalendar(); status = "ready"; }
    catch (x) { if (x === "NotLoggedIn") onauth(); else { error = describeError(x); status = "error"; } }
  }
  onMount(load);
</script>

{#if status !== "ready"}
  <PosterRow title="Calendar" hits={[]} {status} {error} {onopen} onretry={load} />
{:else if !days.length}
  <p class="msg">Nothing scheduled.</p>
{:else}
  {#each days as d (d.key)}<PosterRow title={d.label} hits={d.hits} status="ready" {onopen} onretry={load} />{/each}
{/if}

<style>
  .msg { padding: 18px var(--pad); color: var(--muted); }
</style>
