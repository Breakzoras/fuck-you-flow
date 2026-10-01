import { test } from "node:test";
import assert from "node:assert/strict";
import { testBuildStamp } from "../src/testBuild.ts";

test("a public build carries no stamp and shows nothing", () => {
  assert.equal(testBuildStamp(undefined), null);
  assert.equal(testBuildStamp(null), null);
  assert.equal(testBuildStamp(""), null);
  assert.equal(testBuildStamp("   "), null);
});

test("a value that the local build script did not write shows nothing", () => {
  assert.equal(testBuildStamp("0.9.14"), null);
  assert.equal(testBuildStamp("true"), null);
  assert.equal(testBuildStamp("test:"), null);
  assert.equal(testBuildStamp("test:   "), null);
});

test("a local test build shows its stamp", () => {
  assert.equal(testBuildStamp("test:52484bd 01-10 12:41"), "52484bd 01-10 12:41");
  assert.equal(testBuildStamp("  test: abc1234 02-10 09:05 "), "abc1234 02-10 09:05");
});
