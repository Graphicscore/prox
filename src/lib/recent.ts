// "Recent" Home row: titles played in this app, newest first. Per device (localStorage), at most MAX.
export type Recent = { id: number; title: string; ep: number; at: number };
const KEY = "recent", MAX = 20;

/** `r` moves to the front; an older entry of the same title is dropped. */
export const addRecent = (list: Recent[], r: Recent): Recent[] => [r, ...list.filter((x) => x.id !== r.id)].slice(0, MAX);

const valid = (x: unknown): x is Recent =>
  !!x && typeof x === "object" && Number.isInteger((x as Recent).id) && typeof (x as Recent).title === "string" && Number.isInteger((x as Recent).ep);

export function loadRecent(): Recent[] {
  try { const v = JSON.parse(localStorage.getItem(KEY) ?? "[]"); return Array.isArray(v) ? v.filter(valid) : []; }
  catch { return []; } // storage blocked or corrupt: the row just stays empty
}

export function saveRecent(r: Recent) {
  try { localStorage.setItem(KEY, JSON.stringify(addRecent(loadRecent(), r))); } catch { /* storage unavailable */ }
}

/** Logout: the next account on this device must not see this history. */
export function clearRecent() { try { localStorage.removeItem(KEY); } catch { /* storage unavailable */ } }
