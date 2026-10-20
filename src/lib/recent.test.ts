import { test } from "node:test";
import assert from "node:assert/strict";
import { addRecent } from "./recent.ts";

test("addRecent: last played first, one entry per title, capped at 20", () => {
  const R = (id: number, ep = 1) => ({ id, title: "t" + id, ep, at: id });
  assert.deepEqual(addRecent([R(1), R(2)], R(3)).map((r) => r.id), [3, 1, 2]);
  const moved = addRecent([R(1), R(2), R(3)], R(2, 5));
  assert.deepEqual(moved.map((r) => [r.id, r.ep]), [[2, 5], [1, 1], [3, 1]]);
  const full = Array.from({ length: 20 }, (_, i) => R(i + 1));
  assert.equal(addRecent(full, R(99)).length, 20);
  assert.equal(addRecent(full, R(99))[19].id, 19);
});
