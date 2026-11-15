import { test } from "node:test";
import assert from "node:assert/strict";
import { engineExitReason } from "../src/engineReason.ts";

test("an old processor gets its own sentence", () => {
  assert.equal(engineExitReason("whisper-server exited early (exit code: 0xc000001d)"), "engine_exit_cpu");
  assert.equal(engineExitReason("exit code: 0xC000001D"), "engine_exit_cpu");
});

test("a missing file and a crash get theirs", () => {
  assert.equal(engineExitReason("whisper-server exited early (exit code: 0xc0000135)"), "engine_exit_dll");
  assert.equal(engineExitReason("exit status 3221225477"), "engine_exit_crash");
});

test("an unknown message keeps its own text", () => {
  assert.equal(engineExitReason("whisper-server did not become ready in 120 s"), null);
  assert.equal(engineExitReason("exit code: 1"), null);
  assert.equal(engineExitReason(null), null);
});
