export type Dir = "up" | "down" | "left" | "right";
export type Box = { left: number; top: number; right: number; bottom: number };

const cx = (b: Box) => (b.left + b.right) / 2;
const cy = (b: Box) => (b.top + b.bottom) / 2;
/** Up/Down never moves sideways: buttons of different heights in one row have (almost) the same center. */
const sameRow = (a: Box, b: Box) => Math.abs(cy(a) - cy(b)) < Math.min(a.bottom - a.top, b.bottom - b.top) / 2;

/** Index of the best candidate in direction `dir` from `from`, or -1. A candidate must lie beyond the
 *  origin's center in that direction. Left/right: score = gap along the axis + 2 x offset across it (lower wins).
 *  Up/down: the nearest row wins, then the smallest offset across. */
export function pickNext(from: Box, cands: Box[], dir: Dir): number {
  const vertical = dir === "up" || dir === "down";
  const scored: { i: number; along: number; across: number }[] = [];
  cands.forEach((c, i) => {
    let along: number, across: number;
    if (dir === "right") { if (cx(c) <= cx(from)) return; along = Math.max(0, c.left - from.right); across = Math.abs(cy(c) - cy(from)); }
    else if (dir === "left") { if (cx(c) >= cx(from)) return; along = Math.max(0, from.left - c.right); across = Math.abs(cy(c) - cy(from)); }
    else if (dir === "down") { if (sameRow(c, from) || cy(c) <= cy(from)) return; along = Math.max(0, c.top - from.bottom); across = Math.abs(cx(c) - cx(from)); }
    else { if (sameRow(c, from) || cy(c) >= cy(from)) return; along = Math.max(0, from.top - c.bottom); across = Math.abs(cx(c) - cx(from)); }
    scored.push({ i, along, across });
  });
  if (!scored.length) return -1;
  let pool = scored;
  if (vertical) { // nearest row first (a short chip row must not be skipped for a wide row further away), then closest across
    const min = Math.min(...scored.map((s) => s.along)), tol = (from.bottom - from.top) / 2;
    pool = scored.filter((s) => s.along <= min + tol);
  }
  let best = pool[0];
  for (const s of pool) {
    const score = vertical ? s.across : s.along + 2 * s.across;
    if (score < (vertical ? best.across : best.along + 2 * best.across)) best = s;
  }
  return best.i;
}

const FOCUSABLE = 'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])';

const boxOf = (e: Element): Box => { const r = e.getBoundingClientRect(); return { left: r.left, top: r.top, right: r.right, bottom: r.bottom }; };
const visible = (e: HTMLElement) => { const r = e.getBoundingClientRect(); return r.width > 0 && r.height > 0; };

/** Move focus one step in `dir`. Returns true if focus moved (the caller then prevents the default scroll). */
export function moveFocus(dir: Dir, root: ParentNode = document): boolean {
  const cur = document.activeElement as HTMLElement | null;
  const all = [...root.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(visible);
  if (!cur || cur === document.body) { all[0]?.focus(); all[0]?.scrollIntoView({ block: "nearest", inline: "nearest" }); return all.length > 0; }
  const els = all.filter((e) => e !== cur);
  const i = pickNext(boxOf(cur), els.map(boxOf), dir);
  if (i < 0) return false;
  els[i].focus({ preventScroll: true }); // the (smooth) scrollIntoView below does the scrolling
  els[i].scrollIntoView({ block: "nearest", inline: "nearest" });
  return true;
}
