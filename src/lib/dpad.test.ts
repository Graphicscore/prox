import { test } from "node:test";
import assert from "node:assert/strict";
import { shouldSeek } from "./dpad.ts";

test("shouldSeek", () => {
  assert.equal(shouldSeek(true, "VIDEO", false), true);
  assert.equal(shouldSeek(true, "BUTTON", false), false);
  assert.equal(shouldSeek(true, "BODY", true), true);
  assert.equal(shouldSeek(false, "VIDEO", true), false);
});
