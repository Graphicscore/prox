<script lang="ts">
  import { onMount } from "svelte";
  import { api, describeError, type ListKind, type SearchHit } from "./api";
  import { myListHits } from "./ui/helpers";
  import PosterRow from "./ui/PosterRow.svelte";
  import Hero from "./ui/Hero.svelte";
  let { onopen, onauth }: { onopen: (id: number) => void; onauth: () => void } = $props();

  type Row = { kind: ListKind | "mine"; title: string; status: "loading" | "ready" | "error"; hits: SearchHit[]; error: string };
  let rows = $state<Row[]>([
    { kind: "mine", title: "My list", status: "loading", hits: [], error: "" },
    { kind: "airing", title: "Airing now", status: "loading", hits: [], error: "" },
    { kind: "rating", title: "Top rated", status: "loading", hits: [], error: "" },
    { kind: "clicks", title: "Most viewed", status: "loading", hits: [], error: "" },
  ]);
  const hero = $derived(rows.find((r) => r.kind === "airing")?.hits[0] ?? null);

  async function load(r: Row) {
    r.status = "loading"; r.error = "";
    try { r.hits = r.kind === "mine" ? myListHits(await api.myList()) : (await api.getList(r.kind)).hits; r.status = "ready"; }
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
