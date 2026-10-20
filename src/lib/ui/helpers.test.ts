import { test } from "node:test";
import assert from "node:assert/strict";
import { coverUrl, episodesWith, languagesOf } from "./helpers.ts";

const eps = [
  { number: 2, langs: ["gersub", "engsub"] },
  { number: 1, langs: ["engsub", "engdub", "xxsub"] },
  { number: 3, langs: ["engsub"] },
];

test("coverUrl", () => {
  assert.equal(coverUrl(53), "https://cdn.proxer.me/cover/53.jpg");
});

test("languagesOf orders known languages first, unknown alphabetically", () => {
  assert.deepEqual(languagesOf(eps), ["engsub", "engdub", "gersub", "xxsub"]);
  assert.deepEqual(languagesOf([]), []);
});

test("episodesWith lists episode numbers for a language, ascending", () => {
  assert.deepEqual(episodesWith(eps, "engsub"), [1, 2, 3]);
  assert.deepEqual(episodesWith(eps, "engdub"), [1]);
  assert.deepEqual(episodesWith(eps, "none"), []);
});
