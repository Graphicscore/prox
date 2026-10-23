///<reference types="svelte" />
;
import type { SearchHit } from "../api";
import Poster from "./Poster.svelte";

;type $$ComponentProps =  {
    title: string; hits: SearchHit[]; status: "loading" | "ready" | "error"; error?: string;
    onopen: (id: number) => void; onretry: () => void;
    /** `more`: another page exists; `onmore` is asked for it when focus or scrolling gets near the end. */
    more?: boolean; loadingMore?: boolean; onmore?: () => void;
  };function $$render() {

  
  
  let { title, hits, status, error = "", onopen, onretry, more = false, loadingMore = false, onmore = () => {} }:/*Ωignore_startΩ*/$$ComponentProps/*Ωignore_endΩ*/ = $props();
  let strip = $state<HTMLElement>();
  const NEAR = 6; // posters before the end that trigger the next page
  function focusin(e: FocusEvent) {
    const id = (e.target as HTMLElement).closest("[data-id]")?.getAttribute("data-id");
    const i = hits.findIndex((h) => String(h.id) === id);
    if (more && !loadingMore && i >= 0 && hits.length - i <= NEAR) onmore();
  }
  function scroll() {
    if (more && !loadingMore && strip && strip.scrollLeft + strip.clientWidth >= strip.scrollWidth - strip.clientWidth / 2) onmore();
  }
;
async () => {

if(status !== "ready" || hits.length > 0){
   { svelteHTML.createElement("section", { "class":`row`,});
     { svelteHTML.createElement("h2", {});title; }
    if(status === "error"){
       { svelteHTML.createElement("p", { "role":`alert`,});error;  { svelteHTML.createElement("button", { "onclick":onretry,});  } }
    } else if (status === "loading"){
       { svelteHTML.createElement("div", {   "class":`strip`,"aria-busy":`true`,});    for(let _ of __sveltets_2_ensureArray(Array(8))){let i = 1;i; { svelteHTML.createElement("div", { "class":`skel`,}); }} }
    }else{
       { const $$_div1 = svelteHTML.createElement("div", {      "class":`strip`,"onfocusin":focusin,"onscroll":scroll,});strip = $$_div1;
           for(let h of __sveltets_2_ensureArray(hits)){h.id; { const $$_retsoP2C = __sveltets_2_ensureComponent(Poster); new $$_retsoP2C({ target: __sveltets_2_any(), props: {        "id":h.id,"title":h.title,"rating":h.badge ?? h.rating,"onclick":() => onopen(h.id),}});}}
        if(loadingMore){ { svelteHTML.createElement("div", {   "class":`skel`,"aria-busy":`true`,}); }}
       }
    }
   }
}


};
return { props: {} as any as $$ComponentProps, exports: {}, bindings: __sveltets_$$bindings(''), slots: {}, events: {} }}
const PosterRow__SvelteComponent_ = __sveltets_2_fn_component($$render());
/*Ωignore_startΩ*/type PosterRow__SvelteComponent_ = ReturnType<typeof PosterRow__SvelteComponent_>;
/*Ωignore_endΩ*/export default PosterRow__SvelteComponent_;