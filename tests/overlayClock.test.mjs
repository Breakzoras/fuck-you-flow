import { test } from "node:test";
import assert from "node:assert/strict";
import { recordingClock, limitWarningKey, piecesLeftKey } from "../src/overlay/clock.ts";

test("a short dictation keeps its tenths of a second", () => {
  assert.equal(recordingClock(0), "0.0s");
  assert.equal(recordingClock(7.34), "7.3s");
  assert.equal(recordingClock(119.9), "119.9s");
});

test("from two minutes the badge shows minutes and seconds", () => {
  assert.equal(recordingClock(120), "2:00");
  assert.equal(recordingClock(1205.7), "20:05");
  assert.equal(recordingClock(3725), "1:02:05");
});

test("the warning comes in the last minute of a part and says what happens", () => {
  assert.equal(limitWarningKey(61, true), null);
  assert.equal(limitWarningKey(60, true), "msg_limit_soon_continue");
  assert.equal(limitWarningKey(5, false), "msg_limit_soon_stop");
  assert.equal(limitWarningKey(undefined, true), null);
});

test("the still-writing line counts what is left and vanishes at zero", () => {
  assert.equal(piecesLeftKey(0), null);
  assert.deepEqual(piecesLeftKey(1), { key: "msg_pieces_left_one", vars: { n: 1 } });
  assert.deepEqual(piecesLeftKey(4), { key: "msg_pieces_left", vars: { n: 4 } });
});
