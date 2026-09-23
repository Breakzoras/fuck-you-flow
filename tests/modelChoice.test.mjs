import { test } from "node:test";
import assert from "node:assert/strict";
import { machineFrom, recommend, speakLangFrom, MODEL_EXPLAIN, BACKEND_EXPLAIN } from "../src/modelChoice.ts";

const rtx3070 = machineFrom({ gpus: [{ vram_mb: 8017 }], vulkan_runtime: true, cuda_driver: true });
const laptop4gb = machineFrom({ gpus: [{ vram_mb: 2048 }], vulkan_runtime: true });
const noCard = machineFrom({ gpus: [], vulkan_runtime: false });

test("a Greek speaker with a big card who wants accuracy gets large-v3 on the card", () => {
  assert.deepEqual(recommend(rtx3070, "el_mixed", "accuracy"), { modelId: "large-v3-q5_0", backend: "auto", whyKey: "guide_why_accuracy" });
});

test("wanting speed or a light machine trades Greek accuracy for turbo", () => {
  assert.equal(recommend(rtx3070, "el_mixed", "speed").modelId, "large-v3-turbo-q8_0");
  assert.equal(recommend(rtx3070, "el_mixed", "light").modelId, "large-v3-turbo-q5_0");
});

test("an English speaker gets turbo, which comes close to large-v3 there", () => {
  assert.equal(recommend(rtx3070, "english", "accuracy").modelId, "large-v3-turbo-q8_0");
  assert.equal(recommend(rtx3070, "english", "accuracy").whyKey, "guide_why_english");
});

test("a small card never gets the model that does not fit", () => {
  for (const p of ["accuracy", "speed", "light"]) {
    assert.notEqual(recommend(laptop4gb, "el_mixed", p).modelId, "large-v3-q5_0");
  }
});

test("without a usable card the processor runs the lightest good model", () => {
  assert.deepEqual(recommend(noCard, "el_mixed", "accuracy"), { modelId: "large-v3-turbo-q5_0", backend: "cpu", whyKey: "guide_why_cpu" });
  // A card with memory but no driver the engine can use counts as none.
  assert.equal(recommend(machineFrom({ gpus: [{ vram_mb: 8000 }] }), "el_mixed", "accuracy").backend, "cpu");
});

test("medium is never the pick", () => {
  for (const m of [rtx3070, laptop4gb, noCard]) {
    for (const l of ["el_mixed", "english", "other"]) {
      for (const p of ["accuracy", "speed", "light"]) {
        assert.notEqual(recommend(m, l, p).modelId, "medium-q5_0");
      }
    }
  }
});

test("the spoken language comes from the settings the user already has", () => {
  assert.equal(speakLangFrom({ mode: "multi", primary: "el" }), "el_mixed");
  assert.equal(speakLangFrom({ mode: "english" }), "english");
  assert.equal(speakLangFrom({ mode: "primary", primary: "de" }), "other");
  assert.equal(speakLangFrom({ mode: "auto", primary: "en" }), "english");
});

test("every model and engine the guide can name has an explanation", () => {
  for (const m of [rtx3070, laptop4gb, noCard]) {
    for (const l of ["el_mixed", "english", "other"]) {
      for (const p of ["accuracy", "speed", "light"]) {
        const pick = recommend(m, l, p);
        assert.ok(MODEL_EXPLAIN[pick.modelId], pick.modelId);
        assert.ok(BACKEND_EXPLAIN[pick.backend], pick.backend);
      }
    }
  }
});
