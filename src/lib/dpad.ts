import { moveFocus, type Dir } from "./spatialNav.ts";

let seekMode = false;
/** The player turns this on so Left/Right seek instead of moving focus. */
export const setSeekMode = (on: boolean) => { seekMode = on; };

let root: ParentNode | null = null;
/** Restrict navigation to a subtree (the fullscreen player); null = whole document. */
export const setDpadRoot = (el: ParentNode | null) => { root = el; };

/** Left/Right seek (instead of moving focus) only while the video itself is focused, or in fullscreen. */
export const shouldSeek = (seekMode: boolean, targetTag: string | undefined, hasRoot: boolean) =>
  seekMode && (targetTag === "VIDEO" || hasRoot);

const dirs: Record<string, Dir> = { ArrowUp: "up", ArrowDown: "down", ArrowLeft: "left", ArrowRight: "right" };

export function installDpad(): () => void {
  const onKey = (e: KeyboardEvent) => {
    const dir = dirs[e.key];
    if (!dir || e.defaultPrevented || e.altKey || e.ctrlKey || e.metaKey) return;
    const tag = (e.target as HTMLElement | null)?.tagName;
    const horizontal = dir === "left" || dir === "right";
    if (horizontal && (tag === "INPUT" || tag === "TEXTAREA")) return; // caret movement
    if (horizontal && shouldSeek(seekMode, tag, root !== null)) return;                              // the player seeks
    if (moveFocus(dir, root ?? document)) e.preventDefault();
  };
  window.addEventListener("keydown", onKey);
  return () => window.removeEventListener("keydown", onKey);
}
