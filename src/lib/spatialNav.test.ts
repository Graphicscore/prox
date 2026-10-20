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

test("up from a tile on the right lands in the short chip row above, not the wide button row further up", () => {
  const tile = box(800, 300, 80, 50);
  const chips = [box(0, 230, 80, 40), box(100, 230, 80, 40), box(200, 230, 80, 40)];
  const watch = box(700, 100, 150, 50); // horizontally aligned, but a row further away
  assert.equal(pickNext(tile, [watch, ...chips], "up"), 3);
});

test("down from a top-bar tab skips a slightly offset button in the same row", () => {
  const tab = { left: 160, top: 8, right: 226, bottom: 40 };
  const logout = { left: 880, top: 7.01, right: 940, bottom: 41.02 }; // taller button, center 1/128 px lower
  const hero = { left: 170, top: 300, right: 260, bottom: 340 };
  assert.equal(pickNext(tab, [logout, hero], "down"), 1);
  assert.equal(pickNext(tab, [logout], "down"), -1);
});
