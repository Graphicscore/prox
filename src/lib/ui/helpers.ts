import type { Episode, ListEntry, PageLink, SearchHit } from "../api.ts";

/** Cover image for a title id; built in the browser, no parsing needed. */
export const coverUrl = (id: number) => `https://cdn.proxer.me/cover/${id}.jpg`;

const ORDER = ["engsub", "engdub", "gersub", "gerdub"];

export function languagesOf(episodes: Episode[]): string[] {
  const all = [...new Set(episodes.flatMap((e) => e.langs))];
  const rank = (l: string) => (ORDER.includes(l) ? ORDER.indexOf(l) : ORDER.length);
  return all.sort((a, b) => rank(a) - rank(b) || a.localeCompare(b));
}

export function episodesWith(episodes: Episode[], lang: string): number[] {
  return episodes.filter((e) => e.langs.includes(lang)).map((e) => e.number).sort((a, b) => a - b);
}

/** The language the picker shows: the chosen one if present, else the first available. */
export function activeLang(episodes: Episode[], lang: string): string {
  const langs = languagesOf(episodes);
  return langs.includes(lang) ? lang : (langs[0] ?? "");
}

/** Home "My list": Am Schauen (1) first, then Wird noch geschaut (2), each by last change, newest first. */
export function myListHits(entries: ListEntry[]): SearchHit[] {
  const by = (s: number) => entries.filter((e) => e.state === s).sort((a, b) => b.timestamp - a.timestamp);
  return [...by(1), ...by(2)].map((e) => ({ id: e.id, title: e.name, genres: [], kind: e.medium, rating: null, badge: `${e.episode}/${e.episodes || "?"}` }));
}

/** The episode after `ep` in `lang`: on this page, else the next page (to load first), else null. */
export function nextEpisode(episodes: Episode[], lang: string, ep: number, pages: PageLink[], epPage: number): { ep: number } | { page: number } | null {
  const n = episodesWith(episodes, lang).find((x) => x > ep);
  if (n !== undefined) return { ep: n };
  return pages.some((p) => p.page === epPage + 1) ? { page: epPage + 1 } : null;
}

/** Near-end window for the Next-episode pop-up: last 90 s, or the last 10 % of videos shorter than 15 min. */
export function nearEnd(t: number, duration: number): boolean {
  if (!Number.isFinite(duration) || duration <= 0) return false;
  return duration - t <= (duration < 900 ? duration * 0.1 : 90);
}
