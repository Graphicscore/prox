import { invoke } from "@tauri-apps/api/core";
import { makeDemoApi } from "./demo";

export type SearchHit = { id: number; title: string; genres: string[]; kind: string; rating: number | null; badge?: string };
export type Info = { title: string; fields: Record<string, string> };
export type Stream = { code: string; type: string; name: string; replace: string };
export type MediaSource = { url: string; mime: string; expires_at: number | null };
export type Episode = { number: number; langs: string[] };
export type SearchPage = { hits: SearchHit[]; has_next: boolean };
export type PageLink = { page: number; label: string };
export type ListEntry = { id: number; cid: number; name: string; episode: number; episodes: number; state: number; timestamp: number; medium: string };
export type TitleAction = "note" | "favor" | "finish";
export type EpisodeList = { episodes: Episode[]; pages: PageLink[] };
export const streamLabel = (s: Stream) => (s.type === "proxer-stream" ? "Normal" : s.type === "ps-test" ? "Airing" : "Unsupported");

/** Embed URL from untrusted page data; mirrors the Rust is_allowed_embed_url rules. */
export function safeEmbedUrl(s: Stream): string | null {
  try {
    const u = new URL(s.replace.replaceAll("#", s.code));
    const ok = u.protocol === "https:" && !u.username && !u.password && !u.port && u.hostname.endsWith(".proxer.me") && !u.hostname.startsWith(".");
    return ok ? u.href : null;
  } catch { return null; }
}

export function describeError(x: unknown): string {
  const c = String(x);
  if (c === "BadCredentials") return "Wrong username or password";
  if (c === "Network") return "Could not reach Proxer";
  if (c.startsWith("Http:")) return `Proxer is not responding (HTTP ${c.slice(5)}). Try again.`;
  if (c.startsWith("Parse:")) return "Proxer changed something \u2014 the page looks different";
  if (c === "Unsupported") return "This stream type is not supported yet.";
  if (c.startsWith("Proxer:")) return c.slice(7) || "Proxer refused the action";
  return "Something went wrong";
}

export type ListKind = "airing" | "rating" | "clicks";

export type CalendarEntry = { id: number; title: string; episode: number; lang: string; air: number };

const realApi = {
  /** resolves true if credentials were saved to the keychain (or remember=false) */
  login: (username: string, password: string, remember: boolean) => invoke<boolean>("login", { username, password, remember }),
  autoLogin: () => invoke<boolean>("auto_login"),
  logout: () => invoke<void>("logout"),
  search: (query: string, page = 1) => invoke<SearchPage>("search", { query, page }),
  getList: (kind: ListKind) => invoke<SearchPage>("get_list", { kind }),
  getCalendar: () => invoke<CalendarEntry[]>("get_calendar"),
  info: (id: number) => invoke<Info>("get_info", { id }),
  episodes: (id: number, page = 1) => invoke<EpisodeList>("get_episodes", { id, page }),
  streams: (id: number, episode: number, lang: string) => invoke<Stream[]>("get_streams", { id, episode, lang }),
  resolve: (stream: Stream) => invoke<MediaSource>("resolve_stream", { stream }),
  myList: () => invoke<ListEntry[]>("my_list"),
  titleAction: (id: number, action: TitleAction) => invoke<string>("title_action", { id, action }),
  reminder: (id: number, episode: number, lang: string, which: "this" | "next") => invoke<string>("reminder", { id, episode, lang, which }),
  markWatched: (id: number, episode: number) => invoke<void>("mark_watched", { id, episode }),
};

export type Api = typeof realApi;
// Dev only, explicit opt-in: `npm run dev` + `?demo` in the URL uses fake data so screens render without a login.
export const api: Api = import.meta.env.DEV && new URLSearchParams(location.search).has("demo") ? makeDemoApi() : realApi;
