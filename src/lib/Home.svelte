<script lang="ts">
  import { onMount } from "svelte";
  import { api, describeError, type ListKind, type SearchHit } from "./api";
  import PosterRow from "./ui/PosterRow.svelte";
  import Hero from "./ui/Hero.svelte";
  let { onopen, onauth }: { onopen: (id: number) => void; onauth: () => void } = $props();

  type Row = { kind: ListKind; title: string; status: "loading" | "ready" | "error"; hits: SearchHit[]; error: string };
  let rows = $state<Row[]>([
    { kind: "airing", title: "Airing now", status: "loading", hits: [], error: "" },
    { kind: "rating", title: "Top rated", status: "loading", hits: [], error: "" },
    { kind: "clicks", title: "Most viewed", status: "loading", hits: [], error: "" },
  ]);
  const hero = $derived(rows[0].hits[0] ?? null);

  async function load(r: Row) {
    r.status = "loading"; r.error = "";
    try { r.hits = (await api.getList(r.kind)).hits; r.status = "ready"; }
    catch (x) { if (x === "NotLoggedIn") onauth(); else { r.error = describeError(x); r.status = "error"; } }
  }
  onMount(() => { for (const r of rows) load(r); }); // once; not an $effect, load() writes the state it would track
</script>

{#if hero}
  <Hero id={hero.id} title={hero.title} meta={[hero.kind, hero.rating, hero.genres.slice(0, 3).join(", ")].filter(Boolean).join(" · ")}>
    <button class="primary" onclick={() => onopen(hero.id)}>▶ Open</button>
  </Hero>
{/if}
{#each rows as r (r.kind)}
  <PosterRow title={r.title} hits={r.hits} status={r.status} error={r.error} {onopen} onretry={() => load(r)} />
{/each}
