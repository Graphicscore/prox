// Short messages at the bottom of the screen (also over the fullscreen player). Rendered once by ui/Toasts.svelte.
export const toasts = $state<{ id: number; text: string }[]>([]);
let n = 0;

export function toast(text: string, ms = 3000) {
  const id = ++n;
  toasts.push({ id, text });
  setTimeout(() => { const i = toasts.findIndex((t) => t.id === id); if (i >= 0) toasts.splice(i, 1); }, ms);
}
