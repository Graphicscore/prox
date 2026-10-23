///<reference types="svelte" />
;
import type { SearchHit } from "../api";
import Poster from "./Poster.svelte";

;type $$ComponentProps =  { hits: SearchHit[]; onopen: (id: number) => void };function $$render() {

  
  
  let { hits, onopen }:/*Ωignore_startΩ*/$$ComponentProps/*Ωignore_endΩ*/ = $props();
;
async () => {

 { svelteHTML.createElement("div", { "class":`grid`,});
     for(let h of __sveltets_2_ensureArray(hits)){h.id; { const $$_retsoP1C = __sveltets_2_ensureComponent(Poster); new $$_retsoP1C({ target: __sveltets_2_any(), props: {        "id":h.id,"title":h.title,"rating":h.rating,"onclick":() => onopen(h.id),}});}}
 }


};
return { props: {} as any as $$ComponentProps, exports: {}, bindings: __sveltets_$$bindings(''), slots: {}, events: {} }}
const PosterGrid__SvelteComponent_ = __sveltets_2_fn_component($$render());
/*Ωignore_startΩ*/type PosterGrid__SvelteComponent_ = ReturnType<typeof PosterGrid__SvelteComponent_>;
/*Ωignore_endΩ*/export default PosterGrid__SvelteComponent_;