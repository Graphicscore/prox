// Layers register a handler while they are open; Back asks the most recent first.
const stack: (() => boolean)[] = [];

export function pushBack(fn: () => boolean): () => void {
  stack.push(fn);
  return () => { const i = stack.lastIndexOf(fn); if (i >= 0) stack.splice(i, 1); };
}

export function handleBack(): boolean {
  for (let i = stack.length - 1; i >= 0; i--) if (stack[i]()) return true;
  return false;
}
