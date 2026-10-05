// Sample content for the site's interface pictures (press/app-shots).
//
// The real app, drawn by its own code, with the same neutral sample content
// the older static pictures carried (press/make_shots.py): a small-business
// morning of dictation, six Dictionary rules, four suggestions. Two sets:
// English for the page at /, Greek for /el/. The Greek set is written in
// Greek, interface included; it is not a translation of the English one.
//
// The language and the memory-rail state come from the address bar:
//   ?lang=en|el            which set
//   &gauge=full            the card that ran out of room (7-card-full)

import type {
  DailyStat, DeviceInfo, DictionaryRule, EngineInfo, GpuGaugeInfo, HistoryEntry, LocalApiStatus,
  ModelStatus, PackStatus, PipelineSnapshot, RuntimeStatus, Settings, StatsSummary, Suggestion, UpdateInfo,
} from "../../src/api";

const q = new URLSearchParams(globalThis.location?.search ?? "");
export const LANG: "en" | "el" = q.get("lang") === "el" ? "el" : "en";
const GAUGE_FULL = q.get("gauge") === "full";

export const APP_VERSION = "0.9.14";

/** "Today" is the day of the run, so the Home "today" row reads today's numbers. */
const now = new Date();
export const TODAY = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;

// ------------------------------------------------------------------ settings

export const settings: Settings = {
  general: {
    ui_language: LANG, theme: "dark", skin: "carbon", autostart: true, first_run_done: true, language_confirmed: true,
    play_sounds: true, machine_profiled: true, debug_mode: false, rated_on_github: true,
  },
  hotkeys: { push_to_talk: "Ctrl+Win", hands_free: "RAlt", paste_last: "Shift+LAlt+Z", tap_toggles_hands_free: true, tap_ms: 280 },
  audio: { device_name: null, keep_stream_warm: true, preroll_ms: 350, min_speech_ms: 200, max_recording_seconds: 600 },
  language: LANG === "el" ? { mode: "primary", primary: "el" } : { mode: "english", primary: "en" },
  asr: {
    provider: "whisper_local", model_id: "large-v3-q5_0", use_gpu: true, vad: true, threads: 8, beam_size: 3,
    hints_from_dictionary: true, max_hint_terms: 40, segment_while_speaking: true, backend: "auto",
    openai_base_url: "https://api.openai.com/v1", openai_model: "whisper-1",
  },
  cleanup: {
    intensity: "normal", remove_fillers: true, resolve_self_corrections: true, auto_punctuate: true, auto_capitalize: true,
    llm_enabled: false, llm_model_path: null, cloud_cleanup_enabled: false, intonation_questions: true,
  },
  insertion: { method: "auto", restore_clipboard: true, paste_settle_ms: 60, trailing_space: true, notepad_when_lost: false },
  overlay: { position: "bottom_center", custom_x: 0, custom_y: 0, monitor_name: null, hide_when_idle: true, scale: 1.0, style: "full", dock_edge: "bottom", dock_along: 0.5 },
  privacy: {
    keep_history: true, retention_days: null, keep_audio: false, context_awareness: false, learning_enabled: true,
    learn_from_edits: true, redact_logs: true, local_api: false, local_api_port: 47600,
    share_dictionary: true, share_dictionary_asked: true, share_install_id: "",
  },
  // The Greek set has the ready-made Greek dictionary in; the English one has
  // not looked yet (there is no English list today).
  packs: LANG === "el" ? { enabled: true, asked: true, language: "el", version: 1 } : { enabled: false, asked: true, language: "", version: 0 },
} as unknown as Settings;

export const packStatus: PackStatus | null = LANG === "el" ? { language: "el", rules: 248, version: 1, online: true } : null;

// ------------------------------------------------------------- engine, models

export const engine: EngineInfo = {
  status: "ready", provider: "whisper_local", model_id: "large-v3-q5_0", gpu: true, message: null, warm_ms: 1700, backend: "vulkan",
} as EngineInfo;

export const snapshot: PipelineSnapshot = { phase: "idle", hands_free: false, last_error: null, last_transcript: null, mic_open: false } as PipelineSnapshot;

const HF = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/";

// The three models the older picture listed: two in, the smallest one still to download.
export const models: ModelStatus[] = [
  {
    id: "large-v3-q5_0", file_name: "ggml-large-v3-q5_0.bin", display_name: "Whisper large-v3 (q5_0)", url: `${HF}ggml-large-v3-q5_0.bin`,
    sha256: "d75795ecff3f83b5faa89d1900604ad8c780abd5739fae406de19f23ecd98ad1", size_bytes: 1_081_140_203, vram_mb: 1960, ram_mb: 2400,
    greek_errors_pct: 18, median_ms: 783, languages_key: "model_langs_99", notes_key: "model_note_large_v3", recommended: true, installed: true, verified: true,
  },
  {
    id: "large-v3-turbo-q5_0", file_name: "ggml-large-v3-turbo-q5_0.bin", display_name: "Whisper large-v3-turbo (q5_0)", url: `${HF}ggml-large-v3-turbo-q5_0.bin`,
    sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2", size_bytes: 574_041_195, vram_mb: 860, ram_mb: 1400,
    greek_errors_pct: 25, median_ms: 304, languages_key: "model_langs_99", notes_key: "model_note_turbo_q5", recommended: false, installed: true, verified: true,
  },
  {
    id: "medium-q5_0", file_name: "ggml-medium-q5_0.bin", display_name: "Whisper medium (q5_0)", url: `${HF}ggml-medium-q5_0.bin`,
    sha256: "19fea4b380c3a618ec4723c3eef2eb785ffba0d0538cf43f8f235e7b3b34220f", size_bytes: 539_212_467, vram_mb: 927, ram_mb: 1300,
    greek_errors_pct: 22, median_ms: 560, languages_key: "model_langs_99", notes_key: "model_note_medium", recommended: false, installed: false, verified: false,
  },
].map((m) => ({ ...m, path: m.installed ? `C:\\Users\\demo\\AppData\\Roaming\\FuckYouFlow\\models\\${m.file_name}` : null })) as unknown as ModelStatus[];

export const machine = {
  gpus: [{ name: "NVIDIA GeForce RTX 3070", vendor: "nvidia", vram_mb: 8017 }],
  logical_cores: 16,
  ram_mb: 32768,
  vulkan_runtime: true,
  cuda_driver: true,
};

export const runtime = {
  installed: true,
  cuda_driver: true,
  vad_model: true,
  machine,
  engines: { vulkan: true, cuda: true },
  spec: {
    url: "https://github.com/ggml-org/whisper.cpp/releases/download/b4938/whisper-cublas-12.4.0-bin-x64.zip",
    sha256: "c1b17166e1e31a91cc8e9c1f910d3785e3ce757bb2958bf9dce13fdb4880005f",
    size_bytes: 671_045_732,
    version: "b4938 (CUDA 12.4)",
  },
} as unknown as RuntimeStatus;

// The memory rail, from numbers measured on an RTX 3070: room to spare, and
// the afternoon of 7 September 2026 when other programs had filled the card
// and 1825 MB of the model had been pushed out to system RAM.
const TOTAL = 8017;
export const gauge: GpuGaugeInfo = (GAUGE_FULL
  ? {
      card: { total_mb: TOTAL, used_mb: 6160, free_mb: TOTAL - 6160 }, model_id: "large-v3-q5_0", model_name: "Whisper large-v3 (q5_0)",
      needs_mb: 1960, greek_errors_pct: 18, median_ms: 783, on_card_mb: 133, pushed_out_mb: 1825, state: "spilled",
      fits_instead: "medium-q5_0", fits_instead_name: "Whisper medium (q5_0)",
    }
  : {
      card: { total_mb: TOTAL, used_mb: 2100, free_mb: TOTAL - 2100 }, model_id: "large-v3-q5_0", model_name: "Whisper large-v3 (q5_0)",
      needs_mb: 1960, greek_errors_pct: 18, median_ms: 783, on_card_mb: 1960, pushed_out_mb: 0, state: "ok",
      fits_instead: null, fits_instead_name: null,
    }) as unknown as GpuGaugeInfo;

export const microphones: DeviceInfo[] = [{ name: "Microphone (USB Headset)", is_default: true } as DeviceInfo];
export const localApi: LocalApiStatus = { enabled: false, listening: false, port: 47600, error: null };
export const update: UpdateInfo = { available: false, version: "", current: APP_VERSION, notes: null, date: null, small_download: true } as UpdateInfo;

// ------------------------------------------------------------ sample content

interface Draft { time: string; app: string; process: string; category: string; latency: number; text: string }

const TEXT: Record<"en" | "el", { drafts: Draft[]; rules: [string, string, string, string | null, number][]; suggestions: [string, string, string, number][] }> = {
  en: {
    drafts: [
      { time: "11:42", app: "Word", process: "WINWORD.EXE", category: "document", latency: 350,
        text: "Good morning, send the accountant the September invoice and put the order number in the subject line. If nobody answers by midday, call and ask them to confirm it arrived." },
      { time: "11:39", app: "Chrome", process: "chrome.exe", category: "browser", latency: 590,
        text: "The meeting with the client moves to Thursday at eleven in the morning. We will need the presentation and two printed copies of the quote." },
      { time: "11:36", app: "Outlook", process: "OUTLOOK.EXE", category: "email", latency: 410,
        text: "I need a short summary of the meeting to file with the rest of the notes." },
      { time: "11:31", app: "Slack", process: "slack.exe", category: "chat", latency: 1120,
        text: "Please push the fix to the main branch and open a pull request before noon." },
      { time: "11:28", app: "Outlook", process: "OUTLOOK.EXE", category: "email", latency: 610,
        text: "Let us put the quote together with three options and send it out today." },
    ],
    rules: [
      ["get hub", "GitHub", "phrase", null, 4],
      ["what's app", "WhatsApp", "phrase", null, 3],
      ["vee es code", "VS Code", "phrase", null, 9],
      ["claud", "Claude", "whole_word", null, 2],
      ["ay pee eye", "API", "phrase", null, 0],
      ["en eight en", "n8n", "phrase", null, 1],
    ],
    suggestions: [
      ["get hub", "GitHub", "accepted", 3],
      ["what's app", "WhatsApp", "accepted", 2],
      ["en eight en", "n8n", "pending", 1],
      ["claud", "Claude", "pending", 1],
    ],
  },
  el: {
    drafts: [
      { time: "11:42", app: "Word", process: "WINWORD.EXE", category: "document", latency: 350,
        text: "Καλημέρα, στείλε στον λογιστή το τιμολόγιο του Σεπτεμβρίου και βάλε στο θέμα τον αριθμό της παραγγελίας. Αν δεν απαντήσει μέχρι το μεσημέρι, πάρ' τον τηλέφωνο και ζήτα επιβεβαίωση ότι το έλαβε." },
      { time: "11:39", app: "Chrome", process: "chrome.exe", category: "browser", latency: 590,
        text: "Το ραντεβού με τον πελάτη μετακινείται για την Πέμπτη στις έντεκα το πρωί. Θα χρειαστούμε την παρουσίαση και δύο αντίγραφα της προσφοράς." },
      { time: "11:36", app: "Outlook", process: "OUTLOOK.EXE", category: "email", latency: 410,
        text: "Θέλω μια σύντομη περίληψη της συνάντησης για να την περάσω στο αρχείο μας." },
      { time: "11:31", app: "Slack", process: "slack.exe", category: "chat", latency: 1120,
        text: "Please push the fix to the main branch and open a pull request before noon." },
      { time: "11:28", app: "Outlook", process: "OUTLOOK.EXE", category: "email", latency: 610,
        text: "Να ετοιμάσουμε την προσφορά με τρεις επιλογές και να τη στείλουμε σήμερα." },
    ],
    rules: [
      ["τιμολόγειο", "τιμολόγιο", "whole_word", null, 4],
      ["ουάτσαπ", "WhatsApp", "phrase", null, 3],
      ["μέηλ", "email", "whole_word", "el", 9],
      ["εξέλ", "Excel", "whole_word", null, 2],
      ["άι ντι", "ID", "phrase", null, 0],
      ["εν οκτώ εν", "n8n", "phrase", null, 1],
    ],
    suggestions: [
      ["τιμολόγειο", "τιμολόγιο", "accepted", 3],
      ["ουάτσαπ", "WhatsApp", "accepted", 2],
      ["εν οκτώ εν", "n8n", "pending", 1],
      ["Κλοντ", "Claude", "pending", 1],
    ],
  },
};

const T = TEXT[LANG];
const words = (s: string) => s.split(/\s+/).filter(Boolean).length;

export const history: HistoryEntry[] = T.drafts.map((d, i) => ({
  id: `h-${i + 1}`,
  created_at: `${TODAY}T${d.time}:${String(10 + i * 7).padStart(2, "0")}`,
  raw_text: d.text,
  cleaned_text: d.text,
  final_text: d.text,
  language: LANG === "el" ? "primary" : "english",
  detected_language: LANG,
  app_name: d.app,
  app_process: d.process,
  app_category: d.category,
  cleanup_mode: "normal",
  rules_applied: [],
  audio_ms: Math.round(words(d.text) * 400 + 700),
  latency_ms: d.latency,
  inference_ms: Math.max(150, d.latency - 70),
  word_count: words(d.text),
  engine: "whisper_local",
  model: "large-v3-q5_0",
  insertion_method: "paste",
  status: "success",
  audio_path: null,
  context_used: false,
  retried: false,
  undone: false,
  edited_text: null,
})) as unknown as HistoryEntry[];

export const rules: DictionaryRule[] = T.rules.map(([wrong, correct, match_mode, language, count], i) => ({
  id: `r-${i + 1}`, wrong, correct, match_mode, case_sensitive: false, language, app_scope: null, enabled: true, use_as_hint: false,
  source: "user", created_at: `${TODAY}T08:00:00`, updated_at: `${TODAY}T08:00:00`, apply_count: count,
  last_applied_at: count > 0 ? `${TODAY}T11:${String(40 - i).padStart(2, "0")}:00` : null,
}));

export const suggestions: Suggestion[] = T.suggestions.map(([wrong, correct, status, n], i) => ({
  id: `g-${i + 1}`, kind: "dictionary", wrong, correct, evidence_count: n,
  reason: `You changed "${wrong}" to "${correct}" after dictating (${n} time(s)).`, status,
  created_at: `${TODAY}T09:${String(10 + i).padStart(2, "0")}:00`, updated_at: `${TODAY}T11:${String(10 + i).padStart(2, "0")}:00`,
}));

// ---------------------------------------------------------------- statistics

// The older picture's numbers: 2,684 words in 21 minutes of speech, 54
// dictations, 46 minutes saved at 40 typed words a minute.
// The first day of use: the chart shows the last 30 days, all quiet but today.
const today: DailyStat = { day: TODAY, dictations: 54, words: 2684, audio_ms: Math.round(21.1 * 60_000), corrections: 2, retries: 0, edits: 6 };
const quietDays: DailyStat[] = Array.from({ length: 29 }, (_, i) => {
  const d = new Date(now.getFullYear(), now.getMonth(), now.getDate() - (29 - i));
  const day = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
  return { day, dictations: 0, words: 0, audio_ms: 0, corrections: 0, retries: 0, edits: 0 };
});

export const stats: StatsSummary = {
  total_words: 2684,
  total_minutes: 21.1,
  avg_wpm: 126,
  dictations: 54,
  corrections_applied: 2,
  retries: 0,
  edits: 6,
  current_streak: 1,
  longest_streak: 1,
  time_saved_minutes: 2684 / 40 - 21.1,
  time_saved_formula: "(words / 40 words per minute of typing) - minutes of speech",
  p50_latency_ms: 350,
  p95_latency_ms: 1644,
  by_category: [["chat", 1412], ["browser", 806], ["editor", 466]],
  daily: [...quietDays, today],
};

export const diagnostics: Record<string, unknown> = {
  version: APP_VERSION,
  os: "Windows 11 Pro 23H2",
  engine,
  machine,
  history_entries: history.length,
  dictionary_rules: rules.length,
};
