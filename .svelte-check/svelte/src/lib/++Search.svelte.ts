///<reference types="svelte" />
;
import { onMount, tick } from "svelte";
import { api, describeError, type SearchHit } from "./api";
import PosterGrid from "./ui/PosterGrid.svelte";
import Loading from "./ui/Loading.svelte";

;type $$ComponentProps =  { onopen: (id: number) => void; onauth: () => void };function $$render() {

  
  
  
  
  let { onopen, onauth }:/*Ωignore_startΩ*/$$ComponentProps/*Ωignore_endΩ*/ = $props();
  let q = $state(""), hits = $state<SearchHit[]>([]), error = $state("");
  let term = $state(""), page = $state(1), hasNext = $state(false), busy = $state(false), searched = $state(false);
  async function load(query: string, p: number) {
    if (busy) return; // buttons stay enabled (TV focus); ignore repeats while a search runs
    error = ""; busy = true;
    try {
      const r = await api.search(query, p);
      hits = r.hits; hasNext = r.has_next; term = query; page = p; searched = true;
    } catch (x) { if (x === "NotLoggedIn") onauth(); else error = describeError(x); }
    busy = false;
    await tick(); if (searched && nearEnd()) loadMore(); // first page shorter than the screen
  }
  // More results load by themselves near the end of the grid (remote focus scrolls there too): a sentinel below
  // the grid, watched with some margin, asks for the next Proxer page.
  let sentinel = $state<HTMLElement>(), more = $state(false);
  const nearEnd = () => !!sentinel && sentinel.offsetParent !== null && sentinel.getBoundingClientRect().top < innerHeight + 800;
  async function loadMore() {
    if (busy || more || !hasNext) return;
    more = true;
    try {
      const r = await api.search(term, page + 1);
      const seen = new Set(hits.map((h) => h.id));
      hits = [...hits, ...r.hits.filter((h) => !seen.has(h.id))];
      page += 1; hasNext = r.has_next && r.hits.length > 0;
    } catch (x) { if (x === "NotLoggedIn") onauth(); else error = describeError(x); hasNext = false; }
    more = false;
    await tick();
    if (nearEnd()) loadMore(); // still at the end (short page or tall screen): keep filling
  }
  $effect(() => {
    if (!sentinel) return;
    const io = new IntersectionObserver((es) => { if (es.some((e) => e.isIntersecting)) loadMore(); }, { rootMargin: "0px 0px 800px 0px" });
    io.observe(sentinel);
    return () => io.disconnect();
  });
  if (import.meta.env.DEV) onMount(() => { const d = new URLSearchParams(location.search).get("q"); if (d) { q = d; load(d, 1); } }); // dev demo hook
  function go(e: Event) { e.preventDefault(); load(q, 1); }
;
async () => {

 { svelteHTML.createElement("form", {   "class":`bar`,"onsubmit":go,});
   { svelteHTML.createElement("input", {      "bind:value":q,"placeholder":`Search anime`,"aria-label":`Search anime`,});/*Ωignore_startΩ*/() => q = __sveltets_2_any(null);/*Ωignore_endΩ*/}
   { svelteHTML.createElement("button", { "class":`primary`,});  }
 }
if(busy){ { const $$_gnidaoL0C = __sveltets_2_ensureComponent(Loading); new $$_gnidaoL0C({ target: __sveltets_2_any(), props: {  "label":`Searching…`,}});}}
if(error){ { svelteHTML.createElement("p", {   "role":`alert`,"class":`msg`,});error; }}
if(searched && !hits.length && !error){ { svelteHTML.createElement("p", { "class":`msg`,});  }}
 { const $$_dirGretsoP0C = __sveltets_2_ensureComponent(PosterGrid); new $$_dirGretsoP0C({ target: __sveltets_2_any(), props: {  hits,onopen,}});}
 { const $$_div0 = svelteHTML.createElement("div", {   "aria-hidden":`true`,});sentinel = $$_div0; }
if(more){ { const $$_gnidaoL0C = __sveltets_2_ensureComponent(Loading); new $$_gnidaoL0C({ target: __sveltets_2_any(), props: {  "label":`Loading more…`,}});}}


};
return { props: {} as any as $$ComponentProps, exports: {}, bindings: __sveltets_$$bindings(''), slots: {}, events: {} }}
const Search__SvelteComponent_ = __sveltets_2_fn_component($$render());
/*Ωignore_startΩ*/type Search__SvelteComponent_ = ReturnType<typeof Search__SvelteComponent_>;
/*Ωignore_endΩ*/export default Search__SvelteComponent_;