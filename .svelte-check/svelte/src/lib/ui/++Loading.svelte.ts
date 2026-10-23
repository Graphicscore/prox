///<reference types="svelte" />
;
;type $$ComponentProps =  { label?: string; full?: boolean; compact?: boolean };function $$render() {

  let { label = "Loading…", full = false, compact = false }:/*Ωignore_startΩ*/$$ComponentProps/*Ωignore_endΩ*/ = $props();
;
async () => {

 { svelteHTML.createElement("p", {    "class":`loading`,"role":`status`,});full;compact; { svelteHTML.createElement("span", {   "class":`spin`,"aria-hidden":`true`,}); }label; }


};
return { props: {} as any as $$ComponentProps, exports: {}, bindings: __sveltets_$$bindings(''), slots: {}, events: {} }}
const Loading__SvelteComponent_ = __sveltets_2_fn_component($$render());
/*Ωignore_startΩ*/type Loading__SvelteComponent_ = ReturnType<typeof Loading__SvelteComponent_>;
/*Ωignore_endΩ*/export default Loading__SvelteComponent_;