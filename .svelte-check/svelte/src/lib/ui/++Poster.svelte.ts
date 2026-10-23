///<reference types="svelte" />
;
import Cover from "./Cover.svelte";
import { listStates } from "../liststate.svelte";

;type $$ComponentProps =  { id: number; title: string; rating?: number | string | null; onclick: () => void };function $$render() {

  
  
  const STATE = ["Geschaut", "Am Schauen", "Wird noch geschaut", "Abgebrochen"];
  let { id, title, rating = null, onclick }:/*Ωignore_startΩ*/$$ComponentProps/*Ωignore_endΩ*/ = $props();
;
async () => {

 { svelteHTML.createElement("button", {      "class":`poster`,...__sveltets_2_empty({"data-id":id}),onclick,"aria-label":title,});
   { svelteHTML.createElement("span", { "class":`img`,}); { const $$_revoC2C = __sveltets_2_ensureComponent(Cover); new $$_revoC2C({ target: __sveltets_2_any(), props: {  id,title,}});}if(listStates[id] !== undefined){ { svelteHTML.createElement("span", {   "class":`corner s${listStates[id]}`,"title":STATE[listStates[id]],}); }}if(rating){ { svelteHTML.createElement("span", { "class":`badge`,});rating; }} }
   { svelteHTML.createElement("span", { "class":`t`,});title; }
 }


};
return { props: {} as any as $$ComponentProps, exports: {}, bindings: __sveltets_$$bindings(''), slots: {}, events: {} }}
const Poster__SvelteComponent_ = __sveltets_2_fn_component($$render());
/*Ωignore_startΩ*/type Poster__SvelteComponent_ = ReturnType<typeof Poster__SvelteComponent_>;
/*Ωignore_endΩ*/export default Poster__SvelteComponent_;