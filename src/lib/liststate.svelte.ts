import type { ListEntry } from "./api";

// Title id -> list state (0 Geschaut, 1 Am Schauen, 2 Wird noch geschaut, 3 Abgebrochen) for the poster corners.
// Filled whenever the app reads the user's list (Home's My list row, a title page).
export const listStates: Record<number, number> = $state({});

export function setListStates(entries: ListEntry[]) {
  for (const k of Object.keys(listStates)) delete listStates[Number(k)];
  for (const e of entries) listStates[e.id] = e.state;
}
