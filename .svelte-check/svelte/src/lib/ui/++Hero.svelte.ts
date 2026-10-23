///<reference types="svelte" />
;
import type { Snippet } from "svelte";
import Cover from "./Cover.svelte";
import { coverUrl } from "./helpers";

;type $$ComponentProps =  { id: number; title: string; meta?: string; children?: Snippet };function $$render() {

  
  
  
  let { id, title, meta = "", children }:/*Ωignore_startΩ*/$$ComponentProps/*Ωignore_endΩ*/ = $props();
;
async () => {

 { svelteHTML.createElement("header", { "class":`hero`,});
   { svelteHTML.createElement("div", {  "class":`bg`,});__sveltets_2_ensureType(String, Number, `url(${coverUrl(id)})`); }
   { svelteHTML.createElement("div", { "class":`fade`,}); }
   { svelteHTML.createElement("div", { "class":`in`,});
     { svelteHTML.createElement("div", { "class":`p`,}); { const $$_revoC3C = __sveltets_2_ensureComponent(Cover); new $$_revoC3C({ target: __sveltets_2_any(), props: {  id,title,}});} }
     { svelteHTML.createElement("div", { "class":`txt`,});
       { svelteHTML.createElement("h1", {});title; }
      if(meta){ { svelteHTML.createElement("p", { "class":`meta`,});meta; }}
       { svelteHTML.createElement("div", { "class":`actions`,});;__sveltets_2_ensureSnippet(children?.()); }
     }
   }
 }


};
return { props: {} as any as $$ComponentProps, exports: {}, bindings: __sveltets_$$bindings(''), slots: {}, events: {} }}
const Hero__SvelteComponent_ = __sveltets_2_fn_component($$render());
/*Ωignore_startΩ*/type Hero__SvelteComponent_ = ReturnType<typeof Hero__SvelteComponent_>;
/*Ωignore_endΩ*/export default Hero__SvelteComponent_;