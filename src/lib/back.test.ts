import { test } from "node:test";
import assert from "node:assert/strict";
import { handleBack, pushBack } from "./back.ts";

test("most recent handler first, first true wins", () => {
  const calls: string[] = [];
  const off1 = pushBack(() => { calls.push("a"); return true; });
  const off2 = pushBack(() => { calls.push("b"); return true; });
  assert.equal(handleBack(), true);
  assert.deepEqual(calls, ["b"]);
  off2();
  assert.equal(handleBack(), true);
  assert.deepEqual(calls, ["b", "a"]);
  off1();
});

test("falls through handlers that decline; false when nobody handles", () => {
  const calls: string[] = [];
  const off1 = pushBack(() => { calls.push("a"); return true; });
  const off2 = pushBack(() => { calls.push("b"); return false; });
  assert.equal(handleBack(), true);
  assert.deepEqual(calls, ["b", "a"]);
  off1(); off2();
  assert.equal(handleBack(), false);
});
