// Which model and engine suit one person: what they speak, what matters to
// them, and what their machine can carry. Pure functions, so the rule can be
// tested without a window.
//
// Where the ranking comes from (23 September 2026): on Greek, Whisper large-v3
// makes clearly fewer mistakes than large-v3-turbo (published FLEURS and Common
// Voice figures, 10.9 against 13.0 and 13.7 against 20.6 errors per 100 words),
// while on English turbo comes close to large-v3 and is several times faster.
// medium is older and weaker than turbo at a similar size, so it is never the
// pick; it stays in the list for machines where nothing else loads.

export type SpeakLang = "el_mixed" | "english" | "other";
export type Priority = "accuracy" | "speed" | "light";
export type Backend = "auto" | "vulkan" | "cuda" | "cpu";

export interface Machine {
  /** Memory of the strongest graphics card, 0 when there is none. */
  vramMb: number;
  /** A card this program can use (Vulkan or CUDA present). */
  usableGpu: boolean;
}

export interface Pick {
  modelId: string;
  backend: Backend;
  /** i18n key of the one sentence that says why. */
  whyKey: string;
}

/** Room for large-v3 (about 2 GB) next to whatever else the card is showing. */
const BIG_CARD_MB = 3000;
/** Room for turbo (about 1 GB). Below this the processor does the work. */
const SMALL_CARD_MB = 1500;

export function machineFrom(profile: { gpus?: { vram_mb: number }[]; vulkan_runtime?: boolean; cuda_driver?: boolean } | undefined | null): Machine {
  const vramMb = Math.max(0, ...(profile?.gpus ?? []).map((g) => g.vram_mb || 0));
  return { vramMb, usableGpu: vramMb > 0 && !!(profile?.vulkan_runtime || profile?.cuda_driver) };
}

/** What the person speaks, read from the language settings they already have. */
export function speakLangFrom(language: { mode: string; primary?: string } | undefined | null): SpeakLang {
  if (!language) return "el_mixed";
  if (language.mode === "english") return "english";
  const primary = language.primary ?? "el";
  if (primary === "en") return "english";
  return primary === "el" ? "el_mixed" : "other";
}

export function recommend(m: Machine, lang: SpeakLang, priority: Priority): Pick {
  const card = !m.usableGpu || m.vramMb < SMALL_CARD_MB ? "none" : m.vramMb >= BIG_CARD_MB ? "big" : "small";
  if (card === "none") {
    return { modelId: "large-v3-turbo-q5_0", backend: "cpu", whyKey: "guide_why_cpu" };
  }
  if (priority === "light") {
    return { modelId: "large-v3-turbo-q5_0", backend: "auto", whyKey: "guide_why_light" };
  }
  if (card === "small") {
    return { modelId: priority === "accuracy" ? "large-v3-turbo-q8_0" : "large-v3-turbo-q5_0", backend: "auto", whyKey: "guide_why_small_card" };
  }
  if (lang === "english") {
    return { modelId: priority === "accuracy" ? "large-v3-turbo-q8_0" : "large-v3-turbo-q5_0", backend: "auto", whyKey: "guide_why_english" };
  }
  if (priority === "speed") {
    return { modelId: "large-v3-turbo-q8_0", backend: "auto", whyKey: "guide_why_speed" };
  }
  return { modelId: "large-v3-q5_0", backend: "auto", whyKey: "guide_why_accuracy" };
}

/** The plain explanation shown next to each model: who it is for, what it gives, what it costs. */
export const MODEL_EXPLAIN: Record<string, { who: string; gain: string; cost: string }> = {
  "large-v3-q5_0": { who: "mx_large_who", gain: "mx_large_gain", cost: "mx_large_cost" },
  "large-v3-turbo-q8_0": { who: "mx_turbo8_who", gain: "mx_turbo8_gain", cost: "mx_turbo8_cost" },
  "large-v3-turbo-q5_0": { who: "mx_turbo5_who", gain: "mx_turbo5_gain", cost: "mx_turbo5_cost" },
  "medium-q5_0": { who: "mx_medium_who", gain: "mx_medium_gain", cost: "mx_medium_cost" },
};

/** The plain explanation of each place the engine can run. */
export const BACKEND_EXPLAIN: Record<Backend, string> = {
  auto: "bx_auto",
  vulkan: "bx_vulkan",
  cuda: "bx_cuda",
  cpu: "bx_cpu",
};
