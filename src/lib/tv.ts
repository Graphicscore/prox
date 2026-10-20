/** TV mode: explicit override ("1"/"on"/"0"/"off" from `?tv=` or localStorage `tv`), else Android with no touchscreen. */
export function detectTv(ua: string, maxTouchPoints: number, override: string | null): boolean {
  if (override === "1" || override === "on") return true;
  if (override === "0" || override === "off") return false;
  return /Android/i.test(ua) && maxTouchPoints === 0;
}

export function currentTv(): boolean {
  let stored: string | null = null;
  try { stored = localStorage.getItem("tv"); } catch { /* storage unavailable */ }
  const q = new URLSearchParams(location.search).get("tv");
  return detectTv(navigator.userAgent, navigator.maxTouchPoints, q ?? stored);
}
