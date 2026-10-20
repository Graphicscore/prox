import { test } from "node:test";
import assert from "node:assert/strict";
import { activeLang, coverUrl, episodesWith, languagesOf, myListHits, nearEnd, nextEpisode } from "./helpers.ts";

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

test("activeLang keeps a known language, else falls back to the first", () => {
  assert.equal(activeLang(eps, "gersub"), "gersub");
  assert.equal(activeLang(eps, "gerdub"), "engsub");
  assert.equal(activeLang(eps, ""), "engsub");
  assert.equal(activeLang([], "gerdub"), "");
});

const L = (id: number, state: number, timestamp: number, episode = 0, episodes = 12) =>
  ({ id, cid: id, name: "t" + id, episode, episodes, state, timestamp, medium: "animeseries" });

test("myListHits: watching first, then planned, each newest first; others dropped", () => {
  const hits = myListHits([L(1, 2, 10), L(2, 1, 5, 3), L(3, 0, 99), L(4, 1, 7, 1, 0), L(5, 2, 20), L(6, 3, 50)]);
  assert.deepEqual(hits.map((h) => h.id), [4, 2, 5, 1]);
  assert.equal(hits[0].badge, "1/?");
  assert.equal(hits[1].badge, "3/12");
  assert.equal(hits[1].title, "t2");
});

const pages = [{ page: 1, label: "1 - 50" }, { page: 2, label: "51 - 100" }];

test("nextEpisode: same page, page boundary, last episode", () => {
  assert.deepEqual(nextEpisode(eps, "engsub", 1, pages, 1), { ep: 2 });
  assert.deepEqual(nextEpisode(eps, "gersub", 2, pages, 1), { page: 2 });
  assert.equal(nextEpisode(eps, "gersub", 2, pages, 2), null);
  assert.equal(nextEpisode(eps, "engsub", 3, [], 1), null);
});

test("nearEnd: last 90 s, or last 10 % for videos under 15 min", () => {
  assert.equal(nearEnd(1400, 1440), true);
  assert.equal(nearEnd(1300, 1440), false);
  assert.equal(nearEnd(540, 600), true);
  assert.equal(nearEnd(530, 600), false);
  assert.equal(nearEnd(10, NaN), false);
  assert.equal(nearEnd(0, 0), false);
});
