//! Typed commands exposed to the React UI. The UI never touches audio, the
//! clipboard or model binaries directly.

use std::sync::Arc;

use tauri::{Emitter, Manager, State};

use crate::app::AppState;
use crate::db::{AppStyle, DictionaryRule, HistoryEntry, Snippet, StatsSummary, Suggestion};
use crate::pipeline::{PipelineMsg, PipelineSnapshot};
use crate::settings::Settings;

type R<T> = Result<T, String>;

fn e<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}

// ----- settings -----

#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> Settings {
    state.shared.settings.read().clone()
}

#[tauri::command]
pub async fn save_settings(app: tauri::AppHandle, state: State<'_, Arc<AppState>>, settings: Settings) -> R<Settings> {
    // validate shortcuts first
    for (name, chord) in [("push_to_talk", &settings.hotkeys.push_to_talk), ("hands_free", &settings.hotkeys.hands_free), ("paste_last", &settings.hotkeys.paste_last)] {
        crate::hotkey::Chord::parse(chord).map_err(|err| format!("{name}: {err}"))?;
    }
    let old = state.shared.settings.read().clone();
    settings.save(&crate::paths::settings_file()).map_err(e)?;
    *state.shared.settings.write() = settings.clone();
    crate::app::apply_hotkeys(&settings);
    crate::logging::set_redaction(settings.privacy.redact_logs);
    let _ = state.shared.tx.send(PipelineMsg::SettingsChanged);
    if old.asr.model_id != settings.asr.model_id || old.asr.provider != settings.asr.provider || old.asr.use_gpu != settings.asr.use_gpu || old.asr.vad != settings.asr.vad || old.asr.threads != settings.asr.threads {
        let engine = state.shared.engine.clone();
        let s2 = settings.clone();
        let app2 = app.clone();
        tauri::async_runtime::spawn(async move { engine.apply(&app2, &s2).await });
    }
    if old.general.autostart != settings.general.autostart {
        crate::app::set_autostart(&app, settings.general.autostart);
    }
    if let Some(w) = app.get_webview_window("overlay") {
        crate::overlay::place(&w, &settings.overlay, 0);
    }
    Ok(settings)
}

#[tauri::command]
pub fn get_pipeline_snapshot(state: State<'_, Arc<AppState>>) -> PipelineSnapshot {
    state.shared.snapshot.lock().clone()
}

#[tauri::command]
pub fn pipeline_toggle(state: State<'_, Arc<AppState>>) {
    let _ = state.shared.tx.send(PipelineMsg::Toggle);
}

#[tauri::command]
pub fn pipeline_cancel(state: State<'_, Arc<AppState>>) {
    let _ = state.shared.tx.send(PipelineMsg::Cancel);
}

#[tauri::command]
pub fn pipeline_retry(state: State<'_, Arc<AppState>>) {
    let _ = state.shared.tx.send(PipelineMsg::Retry);
}

#[tauri::command]
pub fn paste_last(state: State<'_, Arc<AppState>>) {
    let _ = state.shared.tx.send(PipelineMsg::PasteLast);
}

#[tauri::command]
pub fn paste_history(state: State<'_, Arc<AppState>>, id: String) {
    let _ = state.shared.tx.send(PipelineMsg::PasteHistory(id));
}

// ----- audio -----

#[tauri::command]
pub fn list_microphones() -> Vec<crate::audio::DeviceInfo> {
    crate::audio::list_devices()
}

#[tauri::command]
pub fn mic_level(state: State<'_, Arc<AppState>>) -> serde_json::Value {
    let a = &state.shared.audio;
    serde_json::json!({ "level": a.level(), "open": a.is_open(), "alive": a.is_alive(), "device": a.open_device() })
}

#[tauri::command]
pub async fn mic_test_open(state: State<'_, Arc<AppState>>) -> R<u32> {
    let audio = state.shared.audio.clone();
    let dev = state.shared.settings.read().audio.device_name.clone();
    tokio::task::spawn_blocking(move || audio.open(dev)).await.map_err(e)?
}

// ----- engine and models -----

#[tauri::command]
pub fn engine_info(state: State<'_, Arc<AppState>>) -> crate::asr::EngineInfo {
    state.shared.engine.info()
}

#[tauri::command]
pub async fn engine_restart(app: tauri::AppHandle, state: State<'_, Arc<AppState>>) -> R<()> {
    let settings = state.shared.settings.read().clone();
    state.shared.engine.apply(&app, &settings).await;
    Ok(())
}

#[tauri::command]
pub fn list_models() -> Vec<crate::models::ModelStatus> {
    crate::models::list_status()
}

#[tauri::command]
pub fn runtime_status() -> serde_json::Value {
    serde_json::json!({
        "installed": crate::models::runtime_installed(),
        "cuda_driver": crate::models::cuda_driver_present(),
        "machine": crate::hw::detect(),
        "engines": {
            "vulkan": crate::asr::whisper_server::find_runtime_exe_for("vulkan").is_some(),
            "cuda": crate::asr::whisper_server::find_runtime_exe_for("cuda").is_some(),
        },
        "vad_model": crate::models::vad_path().is_some(),
        "spec": crate::models::runtime_spec(),
    })
}

#[tauri::command]
pub async fn download_model(app: tauri::AppHandle, id: String) -> R<()> {
    crate::models::download_model(&app, &id).await.map_err(e)
}

#[tauri::command]
pub async fn install_runtime(app: tauri::AppHandle) -> R<()> {
    crate::models::install_runtime(&app).await.map_err(e)
}

#[tauri::command]
pub fn remove_model(id: String) -> R<()> {
    crate::models::remove_model(&id).map_err(e)
}

#[tauri::command]
pub async fn verify_model(id: String) -> R<bool> {
    let (spec, path) = crate::models::find_model(&id).ok_or("unknown model")?;
    tokio::task::spawn_blocking(move || crate::models::verify(&path, &spec.sha256)).await.map_err(e)?.map_err(e)
}

#[tauri::command]
pub fn set_cloud_api_key(key: String) -> R<()> {
    if key.trim().is_empty() {
        crate::asr::openai_compat::OpenAiCompat::delete_key()
    } else {
        crate::asr::openai_compat::OpenAiCompat::store_key(key.trim())
    }
}

#[tauri::command]
pub fn has_cloud_api_key() -> bool {
    crate::asr::openai_compat::OpenAiCompat::has_key()
}

// ----- history -----

#[tauri::command]
pub fn list_history(state: State<'_, Arc<AppState>>, search: Option<String>, limit: Option<u32>, offset: Option<u32>) -> R<Vec<HistoryEntry>> {
    state.shared.db.list_history(search.as_deref(), limit.unwrap_or(200), offset.unwrap_or(0)).map_err(|err| {
        tracing::error!("list_history failed: {err}");
        e(err)
    })
}

#[tauri::command]
pub fn delete_history(state: State<'_, Arc<AppState>>, id: String) -> R<()> {
    if let Some(p) = state.shared.db.delete_history(&id).map_err(e)? {
        let _ = std::fs::remove_file(p);
    }
    Ok(())
}

#[tauri::command]
pub fn delete_all_history(state: State<'_, Arc<AppState>>) -> R<()> {
    for p in state.shared.db.delete_all_history().map_err(e)? {
        let _ = std::fs::remove_file(p);
    }
    let _ = std::fs::remove_dir_all(crate::paths::local_dir().join("audio"));
    let _ = std::fs::remove_dir_all(crate::paths::recovery_dir());
    let _ = std::fs::create_dir_all(crate::paths::recovery_dir());
    Ok(())
}

#[tauri::command]
pub fn reclean_history(state: State<'_, Arc<AppState>>, id: String) -> R<HistoryEntry> {
    let h = state.shared.db.get_history(&id).map_err(e)?.ok_or("not found")?;
    let settings = state.shared.settings.read().clone();
    let opts = crate::cleanup::CleanupOptions {
        intensity: settings.cleanup.intensity.clone(),
        remove_fillers: settings.cleanup.remove_fillers,
        resolve_self_corrections: settings.cleanup.resolve_self_corrections,
        auto_punctuate: settings.cleanup.auto_punctuate,
        auto_capitalize: settings.cleanup.auto_capitalize,
        trailing_punctuation: true,
        capitalize_first: true,
        language: h.detected_language.clone().unwrap_or_else(|| h.language.clone()),
    };
    let out = {
        let d = state.shared.dict.read();
        let s = state.shared.snippets.read();
        crate::cleanup::run_deterministic(&h.raw_text, &opts, &d, &s)
    };
    state.shared.db.update_history_texts(&id, &out.cleaned, &out.cleaned, &out.applied).map_err(e)?;
    state.shared.db.get_history(&id).map_err(e)?.ok_or_else(|| "not found".into())
}

/// The user edited the inserted text. Record the pair for learning (opt-in).
#[tauri::command]
pub fn record_edit(state: State<'_, Arc<AppState>>, id: String, edited: String) -> R<Vec<Suggestion>> {
    let h = state.shared.db.get_history(&id).map_err(e)?.ok_or("not found")?;
    state.shared.db.set_history_edit(&id, &edited).map_err(e)?;
    let _ = state.shared.db.bump_daily_counter("edits");
    let settings = state.shared.settings.read().clone();
    if !settings.privacy.learning_enabled {
        return Ok(vec![]);
    }
    crate::learning::learn_from_edit(&state.shared.db, &id, &h.final_text, &edited).map_err(e)
}

// ----- dictionary -----

#[tauri::command]
pub fn list_rules(state: State<'_, Arc<AppState>>) -> R<Vec<DictionaryRule>> {
    state.shared.db.list_rules().map_err(e)
}

#[tauri::command]
pub fn save_rule(state: State<'_, Arc<AppState>>, mut rule: DictionaryRule) -> R<DictionaryRule> {
    if rule.wrong.trim().is_empty() || rule.correct.trim().is_empty() {
        return Err("both fields are required".into());
    }
    if rule.id.is_empty() {
        rule.id = crate::db::new_id();
        rule.created_at = crate::db::ts_now();
        if rule.source.is_empty() {
            rule.source = "user".into();
        }
    }
    if rule.match_mode.is_empty() {
        rule.match_mode = "whole_word".into();
    }
    rule.updated_at = crate::db::ts_now();
    state.shared.db.upsert_rule(&rule).map_err(e)?;
    crate::app::reload_engines(&state);
    Ok(rule)
}

#[tauri::command]
pub fn delete_rule(state: State<'_, Arc<AppState>>, id: String) -> R<()> {
    state.shared.db.delete_rule(&id).map_err(e)?;
    crate::app::reload_engines(&state);
    Ok(())
}

#[tauri::command]
pub fn add_rule_exception(state: State<'_, Arc<AppState>>, rule_id: String, context: String) -> R<()> {
    state.shared.db.add_rule_exception(&rule_id, &context).map_err(e)?;
    crate::app::reload_engines(&state);
    Ok(())
}

#[tauri::command]
pub fn test_rules(state: State<'_, Arc<AppState>>, text: String) -> crate::cleanup::dictionary::DictResult {
    state.shared.dict.read().apply(&text, "auto")
}

#[tauri::command]
pub fn import_rules(state: State<'_, Arc<AppState>>, rules: Vec<DictionaryRule>) -> R<u32> {
    let mut n = 0;
    for mut r in rules {
        if r.wrong.trim().is_empty() || r.correct.trim().is_empty() {
            continue;
        }
        r.id = crate::db::new_id();
        if r.created_at.is_empty() {
            r.created_at = crate::db::ts_now();
        }
        r.updated_at = crate::db::ts_now();
        if r.match_mode.is_empty() {
            r.match_mode = "whole_word".into();
        }
        state.shared.db.upsert_rule(&r).map_err(e)?;
        n += 1;
    }
    crate::app::reload_engines(&state);
    Ok(n)
}

// ----- snippets -----

#[tauri::command]
pub fn list_snippets(state: State<'_, Arc<AppState>>) -> R<Vec<Snippet>> {
    state.shared.db.list_snippets().map_err(e)
}

#[tauri::command]
pub fn save_snippet(state: State<'_, Arc<AppState>>, mut snippet: Snippet) -> R<Snippet> {
    if snippet.trigger.trim().is_empty() || snippet.expansion.is_empty() {
        return Err("both fields are required".into());
    }
    if snippet.id.is_empty() {
        snippet.id = crate::db::new_id();
        snippet.created_at = crate::db::ts_now();
    }
    snippet.updated_at = crate::db::ts_now();
    state.shared.db.upsert_snippet(&snippet).map_err(e)?;
    crate::app::reload_engines(&state);
    Ok(snippet)
}

#[tauri::command]
pub fn delete_snippet(state: State<'_, Arc<AppState>>, id: String) -> R<()> {
    state.shared.db.delete_snippet(&id).map_err(e)?;
    crate::app::reload_engines(&state);
    Ok(())
}

// ----- suggestions / learning -----

#[tauri::command]
pub fn list_suggestions(state: State<'_, Arc<AppState>>) -> R<Vec<Suggestion>> {
    state.shared.db.list_suggestions().map_err(e)
}

#[tauri::command]
pub fn resolve_suggestion(state: State<'_, Arc<AppState>>, id: String, action: String) -> R<()> {
    let status = match action.as_str() {
        "accept" => "accepted",
        "dismiss" => "dismissed",
        "ignore" => "ignored",
        _ => return Err("bad action".into()),
    };
    let s = state.shared.db.set_suggestion_status(&id, status).map_err(e)?;
    if status == "accepted" {
        if let Some(s) = s {
            let rule = DictionaryRule {
                id: crate::db::new_id(),
                wrong: s.wrong,
                correct: s.correct,
                match_mode: "whole_word".into(),
                case_sensitive: false,
                language: None,
                app_scope: None,
                enabled: true,
                use_as_hint: true,
                source: "suggested".into(),
                created_at: crate::db::ts_now(),
                updated_at: crate::db::ts_now(),
                apply_count: 0,
                last_applied_at: None,
            };
            state.shared.db.upsert_rule(&rule).map_err(e)?;
            crate::app::reload_engines(&state);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn delete_learning_data(state: State<'_, Arc<AppState>>) -> R<()> {
    state.shared.db.delete_learning_data().map_err(e)
}

// ----- app styles -----

#[tauri::command]
pub fn list_app_styles(state: State<'_, Arc<AppState>>) -> R<Vec<AppStyle>> {
    state.shared.db.list_app_styles().map_err(e)
}

#[tauri::command]
pub fn save_app_style(state: State<'_, Arc<AppState>>, mut style: AppStyle) -> R<AppStyle> {
    if style.id.is_empty() {
        style.id = crate::db::new_id();
    }
    state.shared.db.upsert_app_style(&style).map_err(e)?;
    Ok(style)
}

#[tauri::command]
pub fn delete_app_style(state: State<'_, Arc<AppState>>, id: String) -> R<()> {
    state.shared.db.delete_app_style(&id).map_err(e)
}

// ----- stats, privacy, diagnostics -----

#[tauri::command]
pub fn get_stats(state: State<'_, Arc<AppState>>) -> R<StatsSummary> {
    state.shared.db.stats_summary(40.0).map_err(e)
}

#[tauri::command]
pub fn export_all_data(state: State<'_, Arc<AppState>>) -> R<String> {
    let mut v = state.shared.db.export_all().map_err(e)?;
    v["settings"] = serde_json::to_value(state.shared.settings.read().clone()).map_err(e)?;
    serde_json::to_string_pretty(&v).map_err(e)
}

#[tauri::command]
pub fn delete_all_data(state: State<'_, Arc<AppState>>) -> R<()> {
    state.shared.db.wipe_everything().map_err(e)?;
    let _ = std::fs::remove_dir_all(crate::paths::local_dir().join("audio"));
    let _ = std::fs::remove_dir_all(crate::paths::recovery_dir());
    let _ = std::fs::create_dir_all(crate::paths::recovery_dir());
    crate::app::reload_engines(&state);
    Ok(())
}

#[tauri::command]
pub fn open_data_folder() -> R<()> {
    let p = crate::paths::config_dir();
    std::process::Command::new("explorer").arg(p).spawn().map(|_| ()).map_err(e)
}

#[tauri::command]
pub fn open_logs_folder() -> R<()> {
    let p = crate::paths::logs_dir();
    std::process::Command::new("explorer").arg(p).spawn().map(|_| ()).map_err(e)
}

#[tauri::command]
pub fn diagnostics(state: State<'_, Arc<AppState>>) -> serde_json::Value {
    let s = state.shared.settings.read().clone();
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "config_dir": crate::paths::config_dir(),
        "local_dir": crate::paths::local_dir(),
        "engine": state.shared.engine.info(),
        "runtime_installed": crate::models::runtime_installed(),
        "cuda_driver": crate::models::cuda_driver_present(),
        "mic_open": state.shared.audio.is_open(),
        "mic_device": state.shared.audio.open_device(),
        "hotkeys": s.hotkeys,
        "provider": s.asr.provider,
        "model": s.asr.model_id,
    })
}

/// Problem lines from the newest log file, newest first, so the Diagnostics
/// page shows what went wrong without opening the log folder.
#[tauri::command]
pub fn recent_problems() -> Vec<String> {
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(crate::paths::logs_dir())
        .map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_file()).collect())
        .unwrap_or_default();
    files.sort();
    let Some(path) = files.pop() else { return Vec::new() };
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let is_problem = |l: &str| {
        l.contains(" WARN ") || l.contains(" ERROR ") || l.contains("PANIC") || l.contains("dictation failed")
            || l.contains("dictation copied") || l.contains("PasteNotConsumed") || l.contains("transcription failed")
    };
    let mut out: Vec<String> = text.lines().filter(|l| is_problem(l)).map(|l| l.chars().take(300).collect()).collect();
    out.reverse();
    out.truncate(40);
    out
}

#[tauri::command]
pub fn current_foreground_app() -> crate::context::AppContext {
    crate::context::build_context(crate::insertion::capture_target())
}

/// Shortcut recorder: report the keys currently held, as a display string.
#[tauri::command]
pub async fn record_shortcut(app: tauri::AppHandle) -> R<String> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u16>>();
    crate::hotkey::start_recording_keys(tx);
    let _ = app.emit_to("main", "lalia://recording-shortcut", true);
    let mut best: Vec<u16> = Vec::new();
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(6);
    loop {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Some(keys)) => {
                if keys.len() >= best.len() {
                    best = keys;
                }
                // stop when a non-modifier key is present or 3 keys reached
                let has_main = best.iter().any(|k| !matches!(*k, 0x10 | 0x11 | 0x12 | 0xA0..=0xA5 | 0x5B | 0x5C));
                if has_main || best.len() >= 3 {
                    break;
                }
                // otherwise keep listening briefly for more keys
                match tokio::time::timeout(std::time::Duration::from_millis(700), rx.recv()).await {
                    Ok(Some(keys)) => {
                        if keys.len() >= best.len() {
                            best = keys;
                        }
                    }
                    _ => break,
                }
            }
            _ => break,
        }
    }
    crate::hotkey::stop_recording_keys();
    let _ = app.emit_to("main", "lalia://recording-shortcut", false);
    if best.is_empty() {
        return Err("no key pressed".into());
    }
    let mut order: Vec<u16> = best.clone();
    order.sort_by_key(|k| match *k {
        0x11 | 0xA2 | 0xA3 => 0,
        0x10 | 0xA0 | 0xA1 => 1,
        0x12 | 0xA4 | 0xA5 => 2,
        0x5B | 0x5C => 3,
        _ => 4,
    });
    let names: Vec<String> = order
        .into_iter()
        .map(|k| match k {
            0x11 | 0xA2 => "Ctrl".to_string(),
            0xA3 => "RCtrl".to_string(),
            0x10 | 0xA0 => "Shift".to_string(),
            0xA1 => "RShift".to_string(),
            0x12 | 0xA4 => "Alt".to_string(),
            0xA5 => "RAlt".to_string(),
            0x5B | 0x5C => "Win".to_string(),
            other => crate::hotkey::vk_name(other),
        })
        .collect();
    let mut dedup: Vec<String> = Vec::new();
    for n in names {
        if !dedup.contains(&n) {
            dedup.push(n);
        }
    }
    let display = dedup.join("+");
    crate::hotkey::Chord::parse(&display)?;
    Ok(display)
}

#[tauri::command]
pub fn show_main_window(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle, state: State<'_, Arc<AppState>>) {
    let _ = state.shared.tx.send(PipelineMsg::Shutdown);
    crate::app::drop_tray(&app);
    let engine = state.shared.engine.clone();
    tauri::async_runtime::block_on(async move { engine.stop().await });
    app.exit(0);
}
