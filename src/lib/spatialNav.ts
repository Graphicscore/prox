export type Dir = "up" | "down" | "left" | "right";
export type Box = { left: number; top: number; right: number; bottom: number };

const cx = (b: Box) => (b.left + b.right) / 2;
const cy = (b: Box) => (b.top + b.bottom) / 2;

/** Index of the best candidate in direction `dir` from `from`, or -1. A candidate must lie beyond the
 *  origin's center in that direction; score = gap along the axis + 2 x offset across it (lower wins). */
export function pickNext(from: Box, cands: Box[], dir: Dir): number {
  let best = -1, bestScore = Infinity;
  cands.forEach((c, i) => {
    let along: number, across: number;
    if (dir === "right") { if (cx(c) <= cx(from)) return; along = Math.max(0, c.left - from.right); across = Math.abs(cy(c) - cy(from)); }
    else if (dir === "left") { if (cx(c) >= cx(from)) return; along = Math.max(0, from.left - c.right); across = Math.abs(cy(c) - cy(from)); }
    else if (dir === "down") { if (cy(c) <= cy(from)) return; along = Math.max(0, c.top - from.bottom); across = Math.abs(cx(c) - cx(from)); }
    else { if (cy(c) >= cy(from)) return; along = Math.max(0, from.top - c.bottom); across = Math.abs(cx(c) - cx(from)); }
    const score = along + 2 * across;
    if (score < bestScore) { bestScore = score; best = i; }
  });
  return best;
}

const FOCUSABLE = 'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])';

const boxOf = (e: Element): Box => { const r = e.getBoundingClientRect(); return { left: r.left, top: r.top, right: r.right, bottom: r.bottom }; };
const visible = (e: HTMLElement) => { const r = e.getBoundingClientRect(); return r.width > 0 && r.height > 0; };

/** Move focus one step in `dir`. Returns true if focus moved (the caller then prevents the default scroll). */
export function moveFocus(dir: Dir, root: ParentNode = document): boolean {
  const cur = document.activeElement as HTMLElement | null;
  const all = [...root.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(visible);
  if (!cur || cur === document.body) { all[0]?.focus(); return all.length > 0; }
  const els = all.filter((e) => e !== cur);
  const i = pickNext(boxOf(cur), els.map(boxOf), dir);
  if (i < 0) return false;
  els[i].focus();
  els[i].scrollIntoView({ block: "nearest" });
  return true;
}
