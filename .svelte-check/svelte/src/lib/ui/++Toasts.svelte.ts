///<reference types="svelte" />
;
import { toasts } from "../toast.svelte";
function $$render() {

  
;
async () => {

 { svelteHTML.createElement("div", {     "class":`toasts`,"role":`status`,"aria-live":`polite`,});
     for(let t of __sveltets_2_ensureArray(toasts)){t.id; { svelteHTML.createElement("p", { "class":`toast fade-in`,});t.text; }}
 }


};
return { props: {} as Record<string, never>, exports: {}, bindings: "", slots: {}, events: {} }}
const Toasts__SvelteComponent_ = __sveltets_2_isomorphic_component(__sveltets_2_with_any_event($$render()));
/*Ωignore_startΩ*/type Toasts__SvelteComponent_ = InstanceType<typeof Toasts__SvelteComponent_>;
/*Ωignore_endΩ*/export default Toasts__SvelteComponent_;