/** TV mode: explicit override ("1"/"on"/"0"/"off" from `?tv=` or localStorage `tv`), else the native answer
 * (Android UiModeManager via `AndroidTv.isTv()`), else Android with no touchscreen. */
export function detectTv(ua: string, maxTouchPoints: number, override: string | null, native: boolean | null = null): boolean {
  if (override === "1" || override === "on") return true;
  if (override === "0" || override === "off") return false;
  if (typeof native === "boolean") return native;
  return /Android/i.test(ua) && maxTouchPoints === 0;
}

export function currentTv(): boolean {
  let stored: string | null = null;
  try { stored = localStorage.getItem("tv"); } catch { /* storage unavailable */ }
  const q = new URLSearchParams(location.search).get("tv");
  let native: boolean | null = null;
  try {
    const v = (window as any).AndroidTv?.isTv?.();
    native = typeof v === "boolean" ? v : null;
  } catch { /* no bridge */ }
  return detectTv(navigator.userAgent, navigator.maxTouchPoints, q ?? stored, native);
}
