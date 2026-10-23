///<reference types="svelte" />
;
import { coverUrl } from "./helpers";

;type $$ComponentProps =  { id: number; title?: string; class?: string };function $$render() {

  
  let { id, title = "", class: cls = "" }:/*Ωignore_startΩ*/$$ComponentProps/*Ωignore_endΩ*/ = $props();
  let failedId = $state<number | null>(null);
  const failed = $derived(failedId === id);
;
async () => {

if(failed){
   { svelteHTML.createElement("div", {   "class":`ph ${cls}`,"aria-hidden":`true`,});title.slice(0, 1); }
}else{
   { svelteHTML.createElement("img", {        "class":cls,"src":coverUrl(id),"alt":"","loading":`lazy`,"onerror":() => (failedId = id),});}
}


};
return { props: {} as any as $$ComponentProps, exports: {}, bindings: __sveltets_$$bindings(''), slots: {}, events: {} }}
const Cover__SvelteComponent_ = __sveltets_2_fn_component($$render());
/*Ωignore_startΩ*/type Cover__SvelteComponent_ = ReturnType<typeof Cover__SvelteComponent_>;
/*Ωignore_endΩ*/export default Cover__SvelteComponent_;