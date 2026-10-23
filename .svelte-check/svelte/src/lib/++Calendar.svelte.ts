///<reference types="svelte" />
;
import { onMount } from "svelte";
import { api, describeError, type CalendarEntry } from "./api";
import { calendarDays } from "./ui/helpers";
import PosterRow from "./ui/PosterRow.svelte";

;type $$ComponentProps =  { onopen: (id: number) => void; onauth: () => void };function $$render() {

  
  
  
  
  let { onopen, onauth }:/*Ωignore_startΩ*/$$ComponentProps/*Ωignore_endΩ*/ = $props();
  let entries = $state<CalendarEntry[]>([]), status = $state<"loading" | "ready" | "error">("loading"), error = $state("");
  const days = $derived(calendarDays(entries, new Date()));
  async function load() {
    status = "loading"; error = "";
    try { entries = await api.getCalendar(); status = "ready"; }
    catch (x) { if (x === "NotLoggedIn") onauth(); else { error = describeError(x); status = "error"; } }
  }
  onMount(load);
;
async () => {

if(status !== "ready"){
   { const $$_woRretsoP0C = __sveltets_2_ensureComponent(PosterRow); new $$_woRretsoP0C({ target: __sveltets_2_any(), props: {         "title":`Calendar`,"hits":[],status,error,onopen,"onretry":load,}});}
} else if (!days.length){
   { svelteHTML.createElement("p", { "class":`msg`,});  }
}else{
     for(let d of __sveltets_2_ensureArray(days)){d.key; { const $$_woRretsoP0C = __sveltets_2_ensureComponent(PosterRow); new $$_woRretsoP0C({ target: __sveltets_2_any(), props: {         "title":d.label,"hits":d.hits,"status":`ready`,onopen,"onretry":load,}});}}
}


};
return { props: {} as any as $$ComponentProps, exports: {}, bindings: __sveltets_$$bindings(''), slots: {}, events: {} }}
const Calendar__SvelteComponent_ = __sveltets_2_fn_component($$render());
/*Ωignore_startΩ*/type Calendar__SvelteComponent_ = ReturnType<typeof Calendar__SvelteComponent_>;
/*Ωignore_endΩ*/export default Calendar__SvelteComponent_;