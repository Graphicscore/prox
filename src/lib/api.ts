import { invoke } from "@tauri-apps/api/core";

export type SearchHit = { id: number; title: string; genres: string[]; kind: string; rating: number | null };
export type Info = { title: string; fields: Record<string, string> };
export type Stream = { code: string; type: string; name: string; replace: string };
export type Episode = { number: number; langs: string[] };
export const streamLabel = (s: Stream) => (s.type === "proxer-stream" ? "Normal" : s.type === "ps-test" ? "Airing" : "Unsupported");

export function describeError(x: unknown): string {
  const c = String(x);
  if (c === "BadCredentials") return "Wrong username or password";
  if (c === "Network") return "Could not reach Proxer";
  if (c.startsWith("Http:")) return `Proxer is not responding (HTTP ${c.slice(5)}). Try again.`;
  if (c.startsWith("Parse:")) return "Proxer changed something \u2014 the page looks different";
  return "Something went wrong";
}

export const api = {
  /** resolves true if credentials were saved to the keychain (or remember=false) */
  login: (username: string, password: string, remember: boolean) => invoke<boolean>("login", { username, password, remember }),
  autoLogin: () => invoke<boolean>("auto_login"),
  logout: () => invoke<void>("logout"),
  search: (query: string) => invoke<SearchHit[]>("search", { query }),
  info: (id: number) => invoke<Info>("get_info", { id }),
  episodes: (id: number) => invoke<Episode[]>("get_episodes", { id }),
  streams: (id: number, episode: number, lang: string) => invoke<Stream[]>("get_streams", { id, episode, lang }),
};
