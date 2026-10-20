import { test } from "node:test";
import assert from "node:assert/strict";
import { detectTv } from "./tv.ts";

const TV_UA = "Mozilla/5.0 (Linux; Android 14; Google TV) AppleWebKit/537.36 Chrome/120 Mobile Safari/537.36";
const DESKTOP_UA = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 Version/17 Safari/605.1.15";

test("override wins", () => {
  assert.equal(detectTv(DESKTOP_UA, 0, "1"), true);
  assert.equal(detectTv(DESKTOP_UA, 0, "on"), true);
  assert.equal(detectTv(TV_UA, 0, "0"), false);
  assert.equal(detectTv(TV_UA, 0, "off"), false);
});

test("auto: Android without touch is a TV", () => {
  assert.equal(detectTv(TV_UA, 0, null), true);
  assert.equal(detectTv(TV_UA, 5, null), false);   // phone
  assert.equal(detectTv(DESKTOP_UA, 0, null), false);
});
