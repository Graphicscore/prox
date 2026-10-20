import { invoke } from "@tauri-apps/api/core";

export type SearchHit = { id: number; title: string; genres: string[]; kind: string; rating: number | null };
export type Info = { title: string; fields: Record<string, string> };
export type Stream = { code: string; type: string; name: string; replace: string };
export type MediaSource = { url: string; mime: string; expires_at: number | null };
export type Episode = { number: number; langs: string[] };
export type SearchPage = { hits: SearchHit[]; has_next: boolean };
export type PageLink = { page: number; label: string };
export type EpisodeList = { episodes: Episode[]; pages: PageLink[] };
export const streamLabel = (s: Stream) => (s.type === "proxer-stream" ? "Normal" : s.type === "ps-test" ? "Airing" : "Unsupported");

export function describeError(x: unknown): string {
  const c = String(x);
  if (c === "BadCredentials") return "Wrong username or password";
  if (c === "Network") return "Could not reach Proxer";
  if (c.startsWith("Http:")) return `Proxer is not responding (HTTP ${c.slice(5)}). Try again.`;
  if (c.startsWith("Parse:")) return "Proxer changed something \u2014 the page looks different";
  if (c === "Unsupported") return "This stream type is not supported yet.";
  return "Something went wrong";
}

export const api = {
  /** resolves true if credentials were saved to the keychain (or remember=false) */
  login: (username: string, password: string, remember: boolean) => invoke<boolean>("login", { username, password, remember }),
  autoLogin: () => invoke<boolean>("auto_login"),
  logout: () => invoke<void>("logout"),
  search: (query: string, page = 1) => invoke<SearchPage>("search", { query, page }),
  info: (id: number) => invoke<Info>("get_info", { id }),
  episodes: (id: number, page = 1) => invoke<EpisodeList>("get_episodes", { id, page }),
  streams: (id: number, episode: number, lang: string) => invoke<Stream[]>("get_streams", { id, episode, lang }),
  resolve: (stream: Stream) => invoke<MediaSource>("resolve_stream", { stream }),
};
