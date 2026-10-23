///<reference types="svelte" />
;
import Loading from "./Loading.svelte";
import { streamLabel, type Episode, type PageLink, type Stream } from "../api";
import { activeLang, episodesWith, languagesOf, pageWindow, streamThumb } from "./helpers";

;type $$ComponentProps =  {
    episodes: Episode[]; pages: PageLink[]; epPage: number; watchedUpTo?: number; loadingPage?: number; loadingStreams?: boolean; selected: { ep: number; lang: string } | null; streams: Stream[];
    onpage: (p: number) => void; onpick: (ep: number, lang: string) => void; onplay: (s: Stream) => void; onreminder: (which: "this" | "next") => void; lang?: string;
  };function $$render() {

  
  
  
  let { episodes, pages, epPage, selected, streams, watchedUpTo = 0, loadingPage = 0, loadingStreams = false, onpage, onpick, onplay, onreminder, lang = $bindable("") }:/*Ωignore_startΩ*/$$ComponentProps/*Ωignore_endΩ*/ = $props()/*Ωignore_startΩ*/;lang;/*Ωignore_endΩ*/;
  const langs = $derived(languagesOf(episodes));
  const active = $derived(activeLang(episodes, lang));
  const nums = $derived(episodesWith(episodes, active));
;
async () => {

 { svelteHTML.createElement("section", { "class":`picker`,});
   { svelteHTML.createElement("div", { "class":`chips`,});
     { svelteHTML.createElement("h2", {});  }
       for(let l of __sveltets_2_ensureArray(langs)){l;
       { svelteHTML.createElement("button", {     "class":`chip`,"aria-pressed":l === active,"onclick":() => (lang = l),});l; }
    }
    if(pages.length > 1){
       { svelteHTML.createElement("span", { "class":`sep`,}); }
          for(let p of __sveltets_2_ensureArray((pageWindow(pages, epPage)))){let i = 1;p ? p.page : "gap" + i;
        if(!p){ { svelteHTML.createElement("span", { "class":`gap`,});  }}else{
         { svelteHTML.createElement("button", {     "class":`chip`,"aria-current":p.page === epPage ? "page" : undefined,"onclick":() => p.page !== epPage && onpage(p.page),});p.label; }}
      }
    }
   }
  if(loadingPage){ { const $$_gnidaoL1C = __sveltets_2_ensureComponent(Loading); new $$_gnidaoL1C({ target: __sveltets_2_any(), props: {  "compact":true,"label":`Loading episodes…`,}});}}
   { svelteHTML.createElement("div", {  "class":`tiles`,});loadingPage;
       for(let n of __sveltets_2_ensureArray(nums)){n;
       { svelteHTML.createElement("button", {      "class":`tile`,"aria-pressed":selected?.ep === n && selected?.lang === active,"onclick":() => onpick(n, active),});n <= watchedUpTo;n; }
    } { svelteHTML.createElement("p", { "class":`muted`,});  }
   }
  if(selected){
     { svelteHTML.createElement("h3", {});   selected.ep; selected.lang;  }
    const thumb = streams.map(streamThumb).find(Boolean);
    if(thumb){ { svelteHTML.createElement("img", {      "class":`thumb fade-in`,"src":thumb,"alt":`Episode ${selected.ep} preview`,});}}
     { svelteHTML.createElement("div", { "class":`streams fade-in`,});
        for(let s of __sveltets_2_ensureArray(streams)){
        const label = streamLabel(s);
         { svelteHTML.createElement("button", {     "class":`stream`,"disabled":label === "Unsupported","onclick":() => onplay(s),}); s.name;  { svelteHTML.createElement("span", { "class":`badge ${label.toLowerCase()}`,});label; } }
      }if(loadingStreams){ { const $$_gnidaoL2C = __sveltets_2_ensureComponent(Loading); new $$_gnidaoL2C({ target: __sveltets_2_any(), props: {  "compact":true,"label":`Loading streams…`,}});}}else{ { svelteHTML.createElement("p", { "class":`muted`,});   }}
       { svelteHTML.createElement("span", { "class":`sep`,}); }
       { svelteHTML.createElement("button", { "onclick":() => onreminder("this"),});   }
       { svelteHTML.createElement("button", { "onclick":() => onreminder("next"),});   }
     }
  }
 }


};
return { props: {} as any as $$ComponentProps, exports: {}, bindings: __sveltets_$$bindings('lang'), slots: {}, events: {} }}
const EpisodePicker__SvelteComponent_ = __sveltets_2_fn_component($$render());
/*Ωignore_startΩ*/type EpisodePicker__SvelteComponent_ = ReturnType<typeof EpisodePicker__SvelteComponent_>;
/*Ωignore_endΩ*/export default EpisodePicker__SvelteComponent_;