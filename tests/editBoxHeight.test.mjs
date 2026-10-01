import { test } from "node:test";
import assert from "node:assert/strict";
import { editBoxHeight, EDIT_BOX_FLOOR, EDIT_BOX_RESERVE } from "../src/editBoxHeight.ts";

test("a short transcript keeps the smallest box", () => {
  assert.equal(editBoxHeight(40, 800), EDIT_BOX_FLOOR);
});

test("a long transcript opens tall enough to show all of it", () => {
  assert.equal(editBoxHeight(412.4, 800), 413);
});

test("a transcript taller than the window stops short of it, so the buttons stay in sight", () => {
  assert.equal(editBoxHeight(3000, 800), 800 - EDIT_BOX_RESERVE);
});

test("a very small window never shrinks the box under its smallest size", () => {
  assert.equal(editBoxHeight(3000, 250), EDIT_BOX_FLOOR);
});
