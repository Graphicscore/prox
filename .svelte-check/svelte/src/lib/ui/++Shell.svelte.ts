///<reference types="svelte" />
;
import type { Snippet } from "svelte";

;type $$ComponentProps =  {
    tab: "home" | "search" | "calendar" | null; onnav: (t: "home" | "search" | "calendar") => void; onlogout: () => void; onupdate: () => void; updating?: boolean; version?: string; children?: Snippet;
  };function $$render() {

  
  let { tab, onnav, onlogout, onupdate, updating = false, version = "", children }:/*Ωignore_startΩ*/$$ComponentProps/*Ωignore_endΩ*/ = $props();
;
async () => {

 { svelteHTML.createElement("header", { "class":`top`,});
   { svelteHTML.createElement("strong", { "class":`logo`,});  }
   { svelteHTML.createElement("button", {     "class":`tab`,"aria-current":tab === "home" ? "page" : undefined,"onclick":() => onnav("home"),});  }
   { svelteHTML.createElement("button", {     "class":`tab`,"aria-current":tab === "search" ? "page" : undefined,"onclick":() => onnav("search"),});  }
   { svelteHTML.createElement("button", {     "class":`tab`,"aria-current":tab === "calendar" ? "page" : undefined,"onclick":() => onnav("calendar"),});  }
   { svelteHTML.createElement("span", { "class":`grow`,}); }
  if(version){ { svelteHTML.createElement("span", { "class":`ver`,});version; }}
   { svelteHTML.createElement("button", {        "class":`ghost icon`,"onclick":onupdate,"title":`Check for updates`,"aria-label":`Check for updates`,});updating;
     { svelteHTML.createElement("svg", {       "viewBox":`0 0 24 24`,"width":`1.2em`,"height":`1.2em`,"aria-hidden":`true`,});  { svelteHTML.createElement("path", {           "fill":`none`,"stroke":`currentColor`,"stroke-width":`2`,"stroke-linecap":`round`,"stroke-linejoin":`round`,"d":`M20 12a8 8 0 1 1-2.34-5.66M20 4v4h-4`,});} }
   }
   { svelteHTML.createElement("button", {       "class":`ghost icon`,"onclick":onlogout,"title":`Log out`,"aria-label":`Log out`,});
     { svelteHTML.createElement("svg", {       "viewBox":`0 0 24 24`,"width":`1.2em`,"height":`1.2em`,"aria-hidden":`true`,});  { svelteHTML.createElement("path", {           "fill":`none`,"stroke":`currentColor`,"stroke-width":`2`,"stroke-linecap":`round`,"stroke-linejoin":`round`,"d":`M9 4H6a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h3M16 17l5-5-5-5M21 12H9`,});} }
   }
 }
 { svelteHTML.createElement("main", {});;__sveltets_2_ensureSnippet(children?.()); }
if(version){ { svelteHTML.createElement("footer", { "class":`foot`,});version; }}


};
return { props: {} as any as $$ComponentProps, exports: {}, bindings: __sveltets_$$bindings(''), slots: {}, events: {} }}
const Shell__SvelteComponent_ = __sveltets_2_fn_component($$render());
/*Ωignore_startΩ*/type Shell__SvelteComponent_ = ReturnType<typeof Shell__SvelteComponent_>;
/*Ωignore_endΩ*/export default Shell__SvelteComponent_;