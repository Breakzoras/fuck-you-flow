import { test } from "node:test";
import assert from "node:assert/strict";
import { withTimeout } from "../src/async.ts";

test("a stalled backend call reaches the recovery path", async () => {
  await assert.rejects(withTimeout(new Promise(() => {}), 10), /too long/);
});

test("a successful backend call returns its settings", async () => {
  const settings = { general: { ui_language: "el" } };
  assert.equal(await withTimeout(Promise.resolve(settings), 100), settings);
});

test("a backend error remains available to the retry loop", async () => {
  const error = new Error("backend starting");
  await assert.rejects(withTimeout(Promise.reject(error), 100), error);
});

test("late completion cannot replace a timed out result", async () => {
  let finish;
  const pending = new Promise((resolve) => { finish = resolve; });
  await assert.rejects(withTimeout(pending, 10), /too long/);
  finish("old settings");
  assert.equal(await withTimeout(Promise.resolve("new settings"), 100), "new settings");
});
