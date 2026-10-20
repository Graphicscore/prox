import { test } from "node:test";
import assert from "node:assert/strict";
import { pickNext, type Box } from "./spatialNav.ts";

const box = (left: number, top: number, w = 100, h = 50): Box => ({ left, top, right: left + w, bottom: top + h });

test("grid: right and down from top-left", () => {
  const from = box(0, 0);
  const cands = [box(150, 0), box(0, 100), box(150, 100)]; // TR, BL, BR
  assert.equal(pickNext(from, cands, "right"), 0);
  assert.equal(pickNext(from, cands, "down"), 1);
});

test("aligned far candidate beats near offset candidate", () => {
  const from = box(0, 0);
  const far = box(300, 0);          // aligned, 200 away
  const near = box(120, 200);       // 20 away along x but 200 across
  assert.equal(pickNext(from, [near, far], "right"), 1);
});

test("nothing in direction gives -1 and candidates behind are ignored", () => {
  const from = box(300, 300);
  assert.equal(pickNext(from, [box(0, 300), box(300, 0)], "right"), -1);
  assert.equal(pickNext(from, [box(0, 300)], "up"), -1);
  assert.equal(pickNext(from, [], "left"), -1);
});

test("left and up pick the nearest aligned box", () => {
  const from = box(300, 300);
  assert.equal(pickNext(from, [box(0, 300), box(150, 300)], "left"), 1);
  assert.equal(pickNext(from, [box(300, 0), box(300, 150)], "up"), 1);
});
