<script lang="ts">
  import { api, isAiring, type Info, type Episode, type Stream } from "./api";
  let { id, onback, onauth }: { id: number; onback: () => void; onauth: () => void } = $props();
  let info = $state<Info | null>(null), episodes = $state<Episode[]>([]), streams = $state<Stream[]>([]);
  let selected = $state<{ ep: number; lang: string } | null>(null), error = $state("");
  function fail(x: unknown) { if (x === "NotLoggedIn") onauth(); else error = "Could not load"; }
  $effect(() => {
    const cur = id;
    (async () => {
      try { const i = await api.info(cur); const e = await api.episodes(cur); info = i; episodes = e; } catch (x) { fail(x); }
    })();
  });
  async function pick(ep: number, lang: string) {
    selected = { ep, lang }; streams = [];
    try { streams = await api.streams(id, ep, lang); } catch (x) { if (x !== "Parse:streams") fail(x); }
  }
</script>

<button onclick={onback}>Back</button>
{#if error}<p role="alert">{error}</p>{/if}
{#if info}
  <h2>{info.title}</h2>
  <p>Status: {info.fields["Status"] ?? "–"}</p>
  <h3>Episodes</h3>
  <ul>
    {#each episodes as e (e.number)}
      <li>{e.number}: {#each e.langs as l (l)}<button onclick={() => pick(e.number, l)}>{l}</button> {/each}</li>
    {:else}<li>No episodes</li>{/each}
  </ul>
  {#if selected}
    <h3>Streams for episode {selected.ep} ({selected.lang})</h3>
    <ul>
      {#each streams as s (s.code)}<li>{s.name} <small>[{isAiring(s) ? "Airing" : "Normal"}]</small></li>{:else}<li>No streams (yet)</li>{/each}
    </ul>
  {/if}
{/if}
