const TEXT = /^(text|password|search|email|url|tel)$/;
const isText = (e: EventTarget | null): e is HTMLInputElement | HTMLTextAreaElement =>
  (e instanceof HTMLInputElement && TEXT.test(e.type)) || e instanceof HTMLTextAreaElement;

/** TV only: D-pad focus landing on a text field must not open the soft keyboard (it then swallows the D-pad).
 * Focused fields are made readonly (no IME); Enter/Select unlocks and re-focuses so the IME opens; focusout re-locks. */
export function installTvInputs(): () => void {
  const ours = new WeakSet<Element>();
  const lock = (el: HTMLInputElement | HTMLTextAreaElement) => { if (!el.readOnly) { el.readOnly = true; ours.add(el); } };
  let refocusing = false;
  const onIn = (e: Event) => { if (!refocusing && isText(e.target)) lock(e.target); };
  const onOut = onIn;
  const onKey = (e: KeyboardEvent) => {
    const el = e.target;
    if (e.key !== "Enter" || !isText(el) || !ours.has(el)) return;
    e.preventDefault(); // do not submit the form on the unlocking press
    ours.delete(el);
    el.readOnly = false;
    refocusing = true; // blur/focus without re-locking, so the editable field gets focus and the IME opens
    el.blur();
    el.focus();
    refocusing = false;
  };
  document.addEventListener("focusin", onIn, true);
  document.addEventListener("focusout", onOut, true);
  document.addEventListener("keydown", onKey);
  return () => {
    document.removeEventListener("focusin", onIn, true);
    document.removeEventListener("focusout", onOut, true);
    document.removeEventListener("keydown", onKey);
  };
}
