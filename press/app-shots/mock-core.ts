// Stand-in for "@tauri-apps/api/core" in the interface-picture build
// (press/app-shots). Answers the commands in src/api.ts with the sample
// content in data.ts. Nothing here ever rejects: a rejection would turn into a
// red toast inside the picture. An unknown command is logged, and capture.mjs
// refuses to keep a picture that asked for one.

import * as D from "./data";

const clone = <T>(v: T): T => structuredClone(v);

let settings = clone(D.settings);
let rules = clone(D.rules);
const suggestions = clone(D.suggestions);

type Args = Record<string, unknown> | undefined;

const handlers: Record<string, (a: Args) => unknown> = {
  // settings and pipeline
  get_settings: () => clone(settings),
  save_settings: (a) => { settings = clone((a?.settings as typeof settings) ?? settings); return clone(settings); },
  overlay_preview: () => null,
  get_pipeline_snapshot: () => clone(D.snapshot),
  pipeline_toggle: () => null,
  pipeline_cancel: () => null,
  pipeline_retry: () => null,
  paste_last: () => null,
  paste_history: () => null,
  pick_audio_file: () => null,
  transcribe_audio_file: () => "",
  // audio
  list_microphones: () => clone(D.microphones),
  mic_level: () => ({ level: 0.1, open: false, alive: true, device: null }),
  mic_test_open: () => 48000,
  // engine and models
  engine_info: () => clone(D.engine),
  engine_restart: () => null,
  list_models: () => clone(D.models),
  runtime_status: () => clone(D.runtime),
  gpu_gauge: () => clone(D.gauge),
  download_model: () => null,
  install_runtime: () => null,
  remove_model: () => null,
  verify_model: () => true,
  set_cloud_api_key: () => null,
  has_cloud_api_key: () => false,
  // history
  list_history: (a) => {
    const offset = Number(a?.offset ?? 0);
    const limit = Number(a?.limit ?? 200);
    return clone(D.history.slice(offset, offset + limit));
  },
  delete_history: () => null,
  delete_all_history: () => null,
  reclean_history: (a) => clone(D.history.find((h) => h.id === a?.id) ?? D.history[0]),
  record_edit: () => [],
  // dictionary and the ready-made dictionary
  list_rules: () => clone(rules),
  save_rule: (a) => {
    const r = clone(a?.rule as (typeof rules)[number]);
    if (!r.id) r.id = `r-${Date.now()}`;
    rules = [r, ...rules.filter((x) => x.id !== r.id)];
    return clone(r);
  },
  delete_rule: () => null,
  add_rule_exception: () => null,
  test_rules: (a) => ({ text: String(a?.text ?? ""), applied: [], rule_ids: [] }),
  import_rules: () => 0,
  pack_status: () => clone(D.packStatus),
  pack_install: () => ({ added: 0, updated: 0, withdrawn: 0 }),
  pack_decline: () => null,
  pack_remove: () => null,
  share_dictionary_preview: () => [],
  share_dictionary_set: () => clone(settings),
  share_dictionary_forget: () => clone(settings),
  mark_rated_on_github: () => clone(settings),
  // snippets
  list_snippets: () => [],
  save_snippet: (a) => clone(a?.snippet),
  delete_snippet: () => null,
  // learning
  list_suggestions: () => clone(suggestions),
  resolve_suggestion: () => null,
  accept_suggestion_as: () => null,
  delete_learning_data: () => null,
  list_app_styles: () => [],
  save_app_style: (a) => clone(a?.style),
  delete_app_style: () => null,
  // stats and data
  get_stats: () => clone(D.stats),
  export_all_data: () => "{}",
  delete_all_data: () => null,
  open_data_folder: () => null,
  open_logs_folder: () => null,
  // diagnostics and reports
  diagnostics: () => clone(D.diagnostics),
  recent_problems: () => [],
  report_preview: () => "",
  send_report: () => null,
  debug_mode_get: () => false,
  debug_mode_set: () => null,
  debug_events: () => [],
  debug_bundle: () => "",
  recent_keys: () => [],
  current_foreground_app: () => ({ friendly_name: "Word", process: "WINWORD.EXE", category: "document" }),
  record_shortcut: () => "RAlt",
  // app
  app_version: () => D.APP_VERSION,
  linux_input_status: () => ({ linux: false, keyboard: true, virtual_keyboard: true }),
  local_api_status: () => clone(D.localApi),
  check_for_update: () => clone(D.update),
  install_update: () => null,
  quit_app: () => null,
};

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const h = handlers[cmd];
  if (!h) {
    console.warn(`[shots] unknown command: ${cmd}`);
    return null as T;
  }
  try {
    return h(args) as T;
  } catch (e) {
    console.warn(`[shots] command ${cmd} failed quietly:`, e);
    return null as T;
  }
}

// Other names some code imports from core; harmless stand-ins.
export function transformCallback(): number { return 0; }
export function convertFileSrc(path: string): string { return path; }
export function isTauri(): boolean { return false; }
