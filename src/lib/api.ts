import { invoke } from "@tauri-apps/api/core";

export type SearchHit = { id: number; title: string; genres: string[]; kind: string; rating: number | null };
export type Info = { title: string; fields: Record<string, string> };
export type Stream = { code: string; type: string; name: string; replace: string };
export type Episode = { number: number; langs: string[] };
export const isAiring = (s: Stream) => s.type === "ps-test";

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
