import { test } from "node:test";
import assert from "node:assert/strict";
import { localDayKey, todayFacts, TYPING_WPM } from "../src/todayStats.ts";

const day = (d, words, audio_ms) => ({ day: d, dictations: 1, words, audio_ms, corrections: 0, retries: 0, edits: 0 });

test("the today row counts today only, never the all-time total", () => {
  // 1200 words today in 6 minutes of speech: 1200 / 40 = 30 minutes of typing, minus 6 = 24 saved.
  const daily = [day("2026-09-25", 50000, 20 * 60000), day("2026-09-26", 1200, 6 * 60000)];
  assert.equal(TYPING_WPM, 40);
  assert.deepEqual(todayFacts(daily, "2026-09-26"), { words: 1200, savedMinutes: 24 });
});

test("a day without dictations reads zero, even if the last active day had many", () => {
  assert.deepEqual(todayFacts([day("2026-09-25", 900, 60000)], "2026-09-26"), { words: 0, savedMinutes: 0 });
});

test("time saved never goes below zero", () => {
  assert.equal(todayFacts([day("2026-09-26", 10, 5 * 60000)], "2026-09-26").savedMinutes, 0);
});

test("the day key is the local calendar day, as the backend writes it", () => {
  assert.equal(localDayKey(new Date(2026, 8, 6, 23, 59)), "2026-09-06");
});
