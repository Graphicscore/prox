<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { api, describeError, type ListKind, type SearchHit } from "./api";
  import { myListHits } from "./ui/helpers";
  import PosterRow from "./ui/PosterRow.svelte";
  import Hero from "./ui/Hero.svelte";
  let { onopen, onauth, active = true }: { onopen: (id: number) => void; onauth: () => void; active?: boolean } = $props();

  type Row = { kind: ListKind | "mine"; title: string; status: "loading" | "ready" | "error"; hits: SearchHit[]; error: string };
  let rows = $state<Row[]>([
    { kind: "mine", title: "My list", status: "loading", hits: [], error: "" },
    { kind: "airing", title: "Airing now", status: "loading", hits: [], error: "" },
    { kind: "rating", title: "Top rated", status: "loading", hits: [], error: "" },
    { kind: "clicks", title: "Most viewed", status: "loading", hits: [], error: "" },
  ]);
  const hero = $derived(rows.find((r) => r.kind === "airing")?.hits[0] ?? null);

  /** `quiet`: refresh in place (posters stay, focus survives); a failure keeps what is shown. */
  async function load(r: Row, quiet = false) {
    if (!quiet) { r.status = "loading"; r.error = ""; }
    try { r.hits = r.kind === "mine" ? myListHits(await api.myList()) : (await api.getList(r.kind)).hits; r.status = "ready"; r.error = ""; }
    catch (x) { if (x === "NotLoggedIn") onauth(); else if (!quiet) { r.error = describeError(x); r.status = "error"; } }
  }
  onMount(() => { for (const r of rows) load(r); }); // once; not an $effect, load() writes the state it would track
  // Home stays mounted behind an open title: back on Home, refresh only My list (watching/Note/Finished change it).
  let wasActive = true;
  $effect(() => { const a = active; untrack(() => { if (a && !wasActive) load(rows[0], true); wasActive = a; }); });
</script>

{#if hero}
  <Hero id={hero.id} title={hero.title} meta={[hero.kind, hero.rating, hero.genres.slice(0, 3).join(", ")].filter(Boolean).join(" · ")}>
    <button class="primary" onclick={() => onopen(hero.id)}>▶ Open</button>
  </Hero>
{/if}
{#each rows as r (r.kind)}
  <PosterRow title={r.title} hits={r.hits} status={r.status} error={r.error} {onopen} onretry={() => load(r)} />
{/each}
