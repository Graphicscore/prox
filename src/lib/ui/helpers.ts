import type { Episode } from "../api.ts";

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
