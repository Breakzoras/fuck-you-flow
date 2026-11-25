import { test } from "node:test";
import assert from "node:assert/strict";
import { WHATS_NEW_ID, mayBringFront, wantsWhatsNew } from "../src/whatsNewRule.ts";

const updated = { firstRunDone: true, seen: "" };

test("after an update the card shows and may bring the window forward once", () => {
  assert.equal(wantsWhatsNew(updated), true);
  assert.equal(mayBringFront(updated), true);
  const shown = { ...updated, seen: `${WHATS_NEW_ID}~shown` };
  assert.equal(wantsWhatsNew(shown), true);
  assert.equal(mayBringFront(shown), false);
});

test("someone who dealt with the previous card gets the new one, window forward included", () => {
  for (const seen of ["app-mic", "app-mic~shown"]) {
    assert.equal(wantsWhatsNew({ ...updated, seen }), true);
    assert.equal(mayBringFront({ ...updated, seen }), true);
  }
});

test("a dealt-with card and an unfinished setup show nothing", () => {
  for (const s of [
    { ...updated, seen: WHATS_NEW_ID },
    { ...updated, firstRunDone: false },
  ]) {
    assert.equal(wantsWhatsNew(s), false);
    assert.equal(mayBringFront(s), false);
  }
});
