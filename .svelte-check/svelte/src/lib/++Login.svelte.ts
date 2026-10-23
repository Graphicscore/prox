///<reference types="svelte" />
;
import { api, describeError } from "./api";

;type $$ComponentProps =  { onok: (warning?: string) => void; onupdate?: () => void; updating?: boolean; version?: string };function $$render() {

  
  let { onok, onupdate, updating = false, version = "" }:/*Ωignore_startΩ*/$$ComponentProps/*Ωignore_endΩ*/ = $props();
  let username = $state(""), password = $state(""), remember = $state(true), error = $state(""), busy = $state(false);
  async function submit(e: Event) {
    e.preventDefault();
    if (busy) return; // the button stays enabled (TV focus): ignore repeats
    // TV input lock makes fields read-only, which skips `required`: check explicitly.
    if (!username.trim() || !password) { error = "Enter your username and password"; return; }
    busy = true; error = "";
    try {
      const saved = await api.login(username, password, remember);
      onok(remember && !saved ? "Logged in, but your password could not be saved to the system keychain" : undefined);
    } catch (x) { error = String(x).startsWith("Parse:") ? "Proxer changed something — the login page looks different" : describeError(x) === "Something went wrong" ? "Login failed" : describeError(x); }
    busy = false;
  }
;
async () => {

 { svelteHTML.createElement("div", { "class":`wrap`,});
   { svelteHTML.createElement("form", {   "class":`card view`,"onsubmit":submit,});
     { svelteHTML.createElement("h1", {});  }
     { svelteHTML.createElement("p", { "class":`sub`,});      }
     { svelteHTML.createElement("label", {});   { svelteHTML.createElement("input", {    "bind:value":username,"autocomplete":`username`,"required":true,});/*Ωignore_startΩ*/() => username = __sveltets_2_any(null);/*Ωignore_endΩ*/} }
     { svelteHTML.createElement("label", {});   { svelteHTML.createElement("input", {      "bind:value":password,"type":`password`,"autocomplete":`current-password`,"required":true,});/*Ωignore_startΩ*/() => password = __sveltets_2_any(null);/*Ωignore_endΩ*/} }
     { svelteHTML.createElement("label", { "class":`check`,}); { svelteHTML.createElement("input", {    "type":`checkbox`,"bind:checked":remember,});/*Ωignore_startΩ*/() => remember = __sveltets_2_any(null);/*Ωignore_endΩ*/}   }
    if(error){ { svelteHTML.createElement("p", { "role":`alert`,});error; }}
     { svelteHTML.createElement("button", { "class":`primary`,});busy ? "Signing in…" : "Log in"; }
    if(onupdate){ { svelteHTML.createElement("button", {     "type":`button`,"class":`ghost update`,"onclick":onupdate,});updating ? "Checking…" : "Check for updates"; }}
    if(version){ { svelteHTML.createElement("p", { "class":`ver`,});version; }}
   }
 }


};
return { props: {} as any as $$ComponentProps, exports: {}, bindings: __sveltets_$$bindings(''), slots: {}, events: {} }}
const Login__SvelteComponent_ = __sveltets_2_fn_component($$render());
/*Ωignore_startΩ*/type Login__SvelteComponent_ = ReturnType<typeof Login__SvelteComponent_>;
/*Ωignore_endΩ*/export default Login__SvelteComponent_;