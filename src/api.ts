import { invoke } from "@tauri-apps/api/core";

export type LanguageMode = "greek" | "english" | "auto" | "multi";
export type CleanupIntensity = "off" | "light" | "normal" | "strong";
export type InsertionMethod = "auto" | "paste" | "type" | "copy_only";
export type OverlayPosition = "bottom_center" | "top_center" | "bottom_right" | "bottom_left" | "custom";

export interface Settings {
  general: { ui_language: string; theme: string; autostart: boolean; first_run_done: boolean; play_sounds: boolean; machine_profiled?: boolean; debug_mode?: boolean };
  hotkeys: { push_to_talk: string; hands_free: string; paste_last: string; tap_toggles_hands_free: boolean; tap_ms: number };
  audio: { device_name: string | null; keep_stream_warm: boolean; preroll_ms: number; min_speech_ms: number; max_recording_seconds: number };
  language: { mode: LanguageMode };
  asr: {
    provider: string; model_id: string; use_gpu: boolean; vad: boolean; threads: number; beam_size: number;
    hints_from_dictionary: boolean; max_hint_terms: number; openai_base_url: string; openai_model: string;
    segment_while_speaking?: boolean; backend?: string;
  };
  cleanup: {
    intensity: CleanupIntensity; remove_fillers: boolean; resolve_self_corrections: boolean; auto_punctuate: boolean;
    auto_capitalize: boolean; llm_enabled: boolean; llm_model_path: string | null; cloud_cleanup_enabled: boolean;
    intonation_questions?: boolean;
  };
  insertion: { method: InsertionMethod; restore_clipboard: boolean; paste_settle_ms: number; trailing_space: boolean; notepad_when_lost: boolean };
  overlay: { position: OverlayPosition; custom_x: number; custom_y: number; monitor_name: string | null; hide_when_idle: boolean; scale: number; style: "full" | "minimal" };
  privacy: {
    keep_history: boolean; retention_days: number | null; keep_audio: boolean; context_awareness: boolean;
    learning_enabled: boolean; learn_from_edits: boolean; redact_logs: boolean;
  };
}

export interface HistoryEntry {
  id: string; created_at: string; raw_text: string; cleaned_text: string; final_text: string; language: string;
  detected_language: string | null; app_name: string | null; app_process: string | null; app_category: string | null;
  cleanup_mode: string; rules_applied: string[]; audio_ms: number; latency_ms: number; inference_ms: number;
  word_count: number; engine: string | null; model: string | null; insertion_method: string | null; status: string;
  audio_path: string | null; context_used: boolean; retried: boolean; undone: boolean; edited_text: string | null;
}

export interface DictionaryRule {
  id: string; wrong: string; correct: string; match_mode: string; case_sensitive: boolean; language: string | null;
  app_scope: string | null; enabled: boolean; use_as_hint: boolean; source: string; created_at: string; updated_at: string;
  apply_count: number; last_applied_at: string | null;
}

export interface Snippet {
  id: string; trigger: string; expansion: string; enabled: boolean; created_at: string; updated_at: string;
  apply_count: number; last_applied_at: string | null;
}

export interface Suggestion {
  id: string; kind: string; wrong: string; correct: string; evidence_count: number; reason: string; status: string;
  created_at: string; updated_at: string;
}

export interface AppStyle {
  id: string; name: string; process_match: string; category: string; trailing_punctuation: boolean;
  capitalize_first: boolean; tone: string; enabled: boolean;
}

export interface DailyStat { day: string; dictations: number; words: number; audio_ms: number; corrections: number; retries: number; edits: number }

export interface StatsSummary {
  total_words: number; total_minutes: number; avg_wpm: number; dictations: number; corrections_applied: number;
  retries: number; edits: number; current_streak: number; longest_streak: number; time_saved_minutes: number;
  time_saved_formula: string; p50_latency_ms: number; p95_latency_ms: number; by_category: [string, number][];
  daily: DailyStat[];
}

export interface EngineInfo { status: "missing" | "starting" | "ready" | "failed" | "stopped"; provider: string; model_id: string; gpu: boolean; message: string | null; warm_ms: number | null; backend?: string }

export interface ModelStatus {
  id: string; file_name: string; display_name: string; url: string; sha256: string; size_bytes: number; vram_mb: number;
  ram_mb: number; greek_errors_pct: number | null; median_ms: number | null;
  languages_key: string; notes_key: string; recommended: boolean; installed: boolean; verified: boolean; path: string | null;
}

export interface GpuMemory { total_mb: number; used_mb: number; free_mb: number }

/// What the sidebar gauge draws. Every number is measured at the moment of the
/// call: a card with room in the morning can be full by the afternoon.
export interface GpuGaugeInfo {
  card: GpuMemory | null;
  model_id: string;
  model_name: string;
  needs_mb: number;
  greek_errors_pct: number | null;
  median_ms: number | null;
  on_card_mb: number | null;
  pushed_out_mb: number | null;
  state: "ok" | "tight" | "spilled";
  fits_instead: string | null;
  fits_instead_name: string | null;
}

export interface DeviceInfo { name: string; is_default: boolean }
export interface PipelineSnapshot { phase: "idle" | "recording" | "processing"; hands_free: boolean; last_error: string | null; last_transcript: string | null; mic_open: boolean }
export interface RuntimeStatus { installed: boolean; cuda_driver: boolean; vad_model: boolean; spec: { url: string; sha256: string; size_bytes: number; version: string } }
export interface DownloadProgress { id: string; received: number; total: number; phase: string; message: string | null }

/// What the update server says. `small_download` is false when the models are
/// still inside the install folder, which turns a 68 MB update into 1.6 GB.
export interface UpdateInfo {
  available: boolean; version: string; current: string;
  notes: string | null; date: string | null; small_download: boolean;
}

export const api = {
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  snapshot: () => invoke<PipelineSnapshot>("get_pipeline_snapshot"),
  toggle: () => invoke<void>("pipeline_toggle"),
  cancel: () => invoke<void>("pipeline_cancel"),
  retry: () => invoke<void>("pipeline_retry"),
  pasteLast: () => invoke<void>("paste_last"),
  pasteHistory: (id: string) => invoke<void>("paste_history", { id }),
  // A recording the user already has, turned into text. The picker is native
  // and lives in Rust, so no extra package is needed here.
  pickAudioFile: () => invoke<string | null>("pick_audio_file"),
  transcribeAudioFile: (path: string) => invoke<string>("transcribe_audio_file", { path }),
  microphones: () => invoke<DeviceInfo[]>("list_microphones"),
  micLevel: () => invoke<{ level: number; open: boolean; alive: boolean; device: string | null }>("mic_level"),
  micTestOpen: () => invoke<number>("mic_test_open"),
  engineInfo: () => invoke<EngineInfo>("engine_info"),
  engineRestart: () => invoke<void>("engine_restart"),
  models: () => invoke<ModelStatus[]>("list_models"),
  runtimeStatus: () => invoke<RuntimeStatus>("runtime_status"),
  gpuGauge: () => invoke<GpuGaugeInfo>("gpu_gauge"),
  downloadModel: (id: string) => invoke<void>("download_model", { id }),
  installRuntime: () => invoke<void>("install_runtime"),
  removeModel: (id: string) => invoke<void>("remove_model", { id }),
  verifyModel: (id: string) => invoke<boolean>("verify_model", { id }),
  setCloudKey: (key: string) => invoke<void>("set_cloud_api_key", { key }),
  hasCloudKey: () => invoke<boolean>("has_cloud_api_key"),
  history: (search?: string, limit = 200, offset = 0) => invoke<HistoryEntry[]>("list_history", { search, limit, offset }),
  deleteHistory: (id: string) => invoke<void>("delete_history", { id }),
  deleteAllHistory: () => invoke<void>("delete_all_history"),
  recleanHistory: (id: string) => invoke<HistoryEntry>("reclean_history", { id }),
  recordEdit: (id: string, edited: string) => invoke<Suggestion[]>("record_edit", { id, edited }),
  rules: () => invoke<DictionaryRule[]>("list_rules"),
  saveRule: (rule: DictionaryRule) => invoke<DictionaryRule>("save_rule", { rule }),
  deleteRule: (id: string) => invoke<void>("delete_rule", { id }),
  addRuleException: (ruleId: string, context: string) => invoke<void>("add_rule_exception", { ruleId, context }),
  testRules: (text: string) => invoke<{ text: string; applied: string[]; rule_ids: string[] }>("test_rules", { text }),
  importRules: (rules: Partial<DictionaryRule>[]) => invoke<number>("import_rules", { rules }),
  snippets: () => invoke<Snippet[]>("list_snippets"),
  saveSnippet: (snippet: Snippet) => invoke<Snippet>("save_snippet", { snippet }),
  deleteSnippet: (id: string) => invoke<void>("delete_snippet", { id }),
  suggestions: () => invoke<Suggestion[]>("list_suggestions"),
  resolveSuggestion: (id: string, action: "accept" | "dismiss" | "ignore") => invoke<void>("resolve_suggestion", { id, action }),
  deleteLearningData: () => invoke<void>("delete_learning_data"),
  appStyles: () => invoke<AppStyle[]>("list_app_styles"),
  saveAppStyle: (style: AppStyle) => invoke<AppStyle>("save_app_style", { style }),
  deleteAppStyle: (id: string) => invoke<void>("delete_app_style", { id }),
  stats: () => invoke<StatsSummary>("get_stats"),
  exportAll: () => invoke<string>("export_all_data"),
  deleteAll: () => invoke<void>("delete_all_data"),
  openDataFolder: () => invoke<void>("open_data_folder"),
  openLogsFolder: () => invoke<void>("open_logs_folder"),
  diagnostics: () => invoke<Record<string, unknown>>("diagnostics"),
  recentProblems: () => invoke<string[]>("recent_problems"),
  debugModeGet: () => invoke<boolean>("debug_mode_get"),
  debugModeSet: (on: boolean) => invoke<void>("debug_mode_set", { on }),
  debugEvents: (limit?: number) => invoke<string[]>("debug_events", { limit }),
  debugBundle: () => invoke<string>("debug_bundle"),
  foregroundApp: () => invoke<Record<string, unknown>>("current_foreground_app"),
  recordShortcut: () => invoke<string>("record_shortcut"),
  // Updates: the check only asks, the install is a separate yes.
  appVersion: () => invoke<string>("app_version"),
  checkForUpdate: () => invoke<UpdateInfo>("check_for_update"),
  installUpdate: () => invoke<void>("install_update"),
  quit: () => invoke<void>("quit_app"),
};

export function emptyRule(): DictionaryRule {
  return { id: "", wrong: "", correct: "", match_mode: "whole_word", case_sensitive: false, language: null, app_scope: null, enabled: true, use_as_hint: true, source: "user", created_at: "", updated_at: "", apply_count: 0, last_applied_at: null };
}

export function emptySnippet(): Snippet {
  return { id: "", trigger: "", expansion: "", enabled: true, created_at: "", updated_at: "", apply_count: 0, last_applied_at: null };
}

export function emptyStyle(): AppStyle {
  return { id: "", name: "", process_match: "", category: "chat", trailing_punctuation: false, capitalize_first: true, tone: "neutral", enabled: true };
}

export function fmtBytes(n: number): string {
  if (n > 1e9) return (n / 1e9).toFixed(2) + " GB";
  if (n > 1e6) return (n / 1e6).toFixed(0) + " MB";
  return (n / 1e3).toFixed(0) + " KB";
}

export function fmtDate(iso: string, lang: string): string {
  try {
    return new Date(iso).toLocaleString(lang === "el" ? "el-GR" : "en-GB", { dateStyle: "medium", timeStyle: "short" });
  } catch {
    return iso;
  }
}

export function dayKey(iso: string, lang: string): string {
  try {
    return new Date(iso).toLocaleDateString(lang === "el" ? "el-GR" : "en-GB", { weekday: "long", day: "numeric", month: "long", year: "numeric" });
  } catch {
    return iso.slice(0, 10);
  }
}
