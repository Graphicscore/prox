import { test } from "node:test";
import assert from "node:assert/strict";
import { activeLang, versionLabel, pageFor, resumeEpisode, fmtTime, streamThumb, calendarDays, pageWindow, coverUrl, episodesWith, languagesOf, myListHits, nearEnd, nextEpisode } from "./helpers.ts";

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
  // Next follows the playing episode's page, searching from that episode (not from 0) on the next page
  const page2 = [{ number: 51, langs: ["engsub"] }, { number: 52, langs: ["engsub"] }];
  assert.deepEqual(nextEpisode(page2, "engsub", 50, pages, 2), { ep: 51 });
  assert.deepEqual(nextEpisode(page2, "engsub", 51, pages, 2), { ep: 52 });
});

test("nearEnd: last 90 s, or last 10 % for videos under 15 min", () => {
  assert.equal(nearEnd(1400, 1440), true);
  assert.equal(nearEnd(1300, 1440), false);
  assert.equal(nearEnd(540, 600), true);
  assert.equal(nearEnd(530, 600), false);
  assert.equal(nearEnd(10, NaN), false);
  assert.equal(nearEnd(0, 0), false);
});

test("pageWindow: first, last and two around the current page", () => {
  const P = Array.from({ length: 24 }, (_, i) => ({ page: i + 1, label: String(i + 1) }));
  const nums = (cur: number) => pageWindow(P, cur).map((p) => p?.page ?? "…");
  assert.deepEqual(nums(6), [1, "…", 4, 5, 6, 7, 8, "…", 24]);
  assert.deepEqual(nums(1), [1, 2, 3, "…", 24]);
  assert.deepEqual(nums(24), [1, "…", 22, 23, 24]);
  assert.deepEqual(nums(3), [1, 2, 3, 4, 5, "…", 24]);
  assert.equal(pageWindow(P.slice(0, 7), 1).length, 7);
});

test("calendarDays: local days, Today/Tomorrow/date labels, sorted by time, merged same-title episodes", () => {
  const at = (d: number, h: number, m = 0) => new Date(2026, 9, d, h, m).getTime() / 1000;
  const E = (id: number, episode: number, air: number) => ({ id, title: "t" + id, episode, lang: "engsub", air });
  const now = new Date(2026, 9, 20, 12, 0);
  const days = calendarDays([E(3, 1, at(22, 8)), E(1, 4, at(20, 21, 30)), E(2, 7, at(20, 9, 5)), E(1, 5, at(20, 22)), E(4, 2, at(21, 0, 15))], now);
  assert.deepEqual(days.map((d) => d.label), ["Today", "Tomorrow", "Thursday 22 October"]);
  assert.deepEqual(days[0].hits.map((h) => [h.id, h.badge]), [[2, "09:05 · Ep 7"], [1, "21:30 · Ep 4, 5"]]);
  assert.deepEqual(days[1].hits.map((h) => [h.id, h.badge]), [[4, "00:15 · Ep 2"]]);
  assert.equal(days[0].hits[0].title, "t2");
  assert.equal(calendarDays([], now).length, 0);
});

test("streamThumb: airing streams only, safe codes only", () => {
  const S = (type: string, code: string) => ({ code, type, name: "n", replace: "" });
  assert.equal(streamThumb(S("ps-test", "hdarrnfzlxyw")), "https://stream-service-cdn.proxer.me/thumb/hdarrnfzlxyw.jpg");
  assert.equal(streamThumb(S("proxer-stream", "z6iu152riwx4")), null);
  assert.equal(streamThumb(S("ps-test", "../x")), null);
});

test("fmtTime", () => {
  assert.equal(fmtTime(0), "0:00");
  assert.equal(fmtTime(65.9), "1:05");
  assert.equal(fmtTime(1420), "23:40");
  assert.equal(fmtTime(3725), "1:02:05");
  assert.equal(fmtTime(NaN), "0:00");
});

test("resumeEpisode: next unwatched, a later started episode wins, restart when finished", () => {
  assert.equal(resumeEpisode(0, 12, null), 1);
  assert.equal(resumeEpisode(3, 12, null), 4);
  assert.equal(resumeEpisode(3, 12, 3), 4);   // ep 3 counted as watched
  assert.equal(resumeEpisode(2, 12, 5), 5);   // started ep 5 here, list behind
  assert.equal(resumeEpisode(12, 12, 12), 1); // finished: from the start
  assert.equal(resumeEpisode(7, 0, null), 8); // airing, unknown total
});

test("pageFor finds the page by its label range", () => {
  const P = [{ page: 1, label: "1 - 50" }, { page: 2, label: "51 - 100" }, { page: 3, label: "101 - 120" }];
  assert.equal(pageFor(P, 1), 1);
  assert.equal(pageFor(P, 51), 2);
  assert.equal(pageFor(P, 120), 3);
  assert.equal(pageFor(P, 121), null);
  assert.equal(pageFor([], 3), null);
});

test("versionLabel", () => {
  assert.equal(versionLabel("0.1.42"), "Build 42 · v0.1.42");
  assert.equal(versionLabel("0.1.0"), "Dev build · v0.1.0");
});
