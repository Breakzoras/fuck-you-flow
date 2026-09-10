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
pub async fn save_settings(app: tauri::AppHandle, state: State<'_, Arc<AppState>>, mut settings: Settings) -> R<Settings> {
    // Validate the shortcuts first. One bad one used to throw away the whole
    // save, including every change on every other tab, and the message named a
    // field the user could not see. It now says which shortcut and in a form the
    // interface can translate.
    for (name, chord) in [("push_to_talk", &settings.hotkeys.push_to_talk), ("hands_free", &settings.hotkeys.hands_free), ("paste_last", &settings.hotkeys.paste_last)] {
        if chord.trim().is_empty() {
            // An empty box means the user wants that shortcut switched off,
            // which is a decision, not a mistake.
            continue;
        }
        crate::hotkey::Chord::parse(chord).map_err(|err| format!("bad_shortcut|{name}|{err}"))?;
    }
    // Zero here means "delete everything older than zero days", which is
    // everything. The box on screen refuses it but the box is only HTML, and a
    // saved zero quietly emptied the whole history and its recordings on the
    // next start.
    if settings.privacy.retention_days == Some(0) {
        tracing::warn!("retention of 0 days would erase the whole history; keeping 1");
        settings.privacy.retention_days = Some(1);
    }
    let old = state.shared.settings.read().clone();
    // Once the user has touched the card switch or the acceleration, startup
    // stops correcting them: their choice outranks our hardware guess.
    if old.asr.use_gpu != settings.asr.use_gpu || old.asr.backend != settings.asr.backend {
        settings.general.gpu_choice_by_user = true;
    }
    settings.save(&crate::paths::settings_file()).map_err(e)?;
    *state.shared.settings.write() = settings.clone();
    crate::app::apply_hotkeys(&settings);
    crate::logging::set_redaction(settings.privacy.redact_logs);
    let _ = state.shared.tx.send(PipelineMsg::SettingsChanged);
    // `backend` belongs in this list: changing the acceleration on its own used
    // to leave the old engine running, so the setting looked ignored.
    if old.asr.model_id != settings.asr.model_id
        || old.asr.provider != settings.asr.provider
        || old.asr.use_gpu != settings.asr.use_gpu
        || old.asr.backend != settings.asr.backend
        || old.asr.vad != settings.asr.vad
        || old.asr.threads != settings.asr.threads
    {
        let engine = state.shared.engine.clone();
        let s2 = settings.clone();
        let app2 = app.clone();
        tauri::async_runtime::spawn(async move { engine.apply(&app2, &s2).await });
    }
    if old.general.autostart != settings.general.autostart {
        if !crate::app::set_autostart(&app, settings.general.autostart) {
            // Nothing was written to Windows, so the switch goes back off and
            // the saved file follows it. Leaving it on promised a start that
            // was never going to happen.
            settings.general.autostart = false;
            let _ = settings.save(&crate::paths::settings_file());
            *state.shared.settings.write() = settings.clone();
            tracing::warn!("the startup switch was turned back off: this copy cannot claim it");
        }
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

/// Everything the memory gauge draws, in one poll.
#[derive(serde::Serialize)]
pub struct GpuGauge {
    /// `None` on a machine whose card will not report its memory.
    pub card: Option<crate::hw::GpuMemory>,
    pub model_id: String,
    pub model_name: String,
    /// What this model has to fit inside.
    pub needs_mb: u32,
    pub greek_errors_pct: Option<u32>,
    pub median_ms: Option<u32>,
    /// Of the model, how much is still on the card and how much Windows pushed
    /// out to system RAM. `None` until the engine is running locally.
    pub on_card_mb: Option<u64>,
    pub pushed_out_mb: Option<u64>,
    /// "ok", "tight" or "spilled". "spilled" is the state that makes dictation
    /// take thirty seconds instead of one.
    pub state: String,
    /// The most accurate model already installed that would fit in the room
    /// there is, offered only when the current one does not fit.
    pub fits_instead: Option<String>,
    pub fits_instead_name: Option<String>,
}

/// Read on a timer by the gauge in the sidebar, and once after each dictation.
/// Every number here is measured at the moment of the call: a card that had
/// room this morning can be full by the afternoon without anything in this app
/// changing, which is exactly the failure this gauge exists to show.
// Runs off the window thread. A Tauri command declared without `async`
// is executed on the thread that pumps the window, so while this one
// worked the whole dashboard stopped repainting.
#[tauri::command(async)]
pub fn gpu_gauge(state: State<'_, Arc<AppState>>) -> GpuGauge {
    let settings = state.shared.settings.read().clone();
    let id = settings.asr.model_id.clone();
    let spec = crate::models::catalog().into_iter().find(|m| m.id == id);
    let needs_mb = spec.as_ref().map(|s| s.vram_mb).unwrap_or(0);

    let card = crate::hw::gpu_memory();
    let residency = state.shared.engine.local_pid().and_then(crate::hw::process_gpu_memory);
    let (on_card_mb, pushed_out_mb) = match residency {
        Some((on, out)) => (Some(on), Some(out)),
        None => (None, None),
    };

    // What the card could offer this model: what is free now, plus whatever the
    // model is already holding there (it would give that back on a reload).
    let free_mb = card.map(|c| c.free_mb).unwrap_or(0);
    let room_mb = free_mb + on_card_mb.unwrap_or(0);

    let state_word = match (residency, card) {
        // Measured: the model is running and part of it is off the card.
        (Some((on, out)), _) if on + out > 0 => {
            let resident = on as f64 / (on + out) as f64;
            if resident < 0.95 {
                "spilled"
            } else if room_mb < (needs_mb as f64 * 1.15) as u64 {
                "tight"
            } else {
                "ok"
            }
        }
        // Not running yet: judged from the room there is.
        (_, Some(_)) if needs_mb > 0 => {
            if room_mb < needs_mb as u64 {
                "spilled"
            } else if room_mb < (needs_mb as f64 * 1.15) as u64 {
                "tight"
            } else {
                "ok"
            }
        }
        _ => "ok",
    };

    // Offered only when the current model does not fit: the most accurate one
    // already on disk that would. A model with no measured accuracy is skipped
    // rather than guessed at.
    let fits = (state_word == "spilled")
        .then(|| {
            let mut options: Vec<_> = crate::models::list_status()
                .into_iter()
                .filter(|m| m.installed && m.spec.id != id && (m.spec.vram_mb as u64) < room_mb)
                .filter(|m| m.spec.greek_errors_pct.is_some())
                .collect();
            options.sort_by_key(|m| m.spec.greek_errors_pct.unwrap_or(u32::MAX));
            options.into_iter().next()
        })
        .flatten();

    GpuGauge {
        card,
        model_name: spec.as_ref().map(|s| s.display_name.clone()).unwrap_or_else(|| id.clone()),
        model_id: id,
        needs_mb,
        greek_errors_pct: spec.as_ref().and_then(|s| s.greek_errors_pct),
        median_ms: spec.as_ref().and_then(|s| s.median_ms),
        on_card_mb,
        pushed_out_mb,
        state: state_word.into(),
        fits_instead_name: fits.as_ref().map(|m| m.spec.display_name.clone()),
        fits_instead: fits.map(|m| m.spec.id),
    }
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

// Runs off the window thread. A Tauri command declared without `async`
// is executed on the thread that pumps the window, so while this one
// worked the whole dashboard stopped repainting.
#[tauri::command(async)]
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

// Runs off the window thread. A Tauri command declared without `async`
// is executed on the thread that pumps the window, so while this one
// worked the whole dashboard stopped repainting.
#[tauri::command(async)]
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

// Runs off the window thread. A Tauri command declared without `async`
// is executed on the thread that pumps the window, so while this one
// worked the whole dashboard stopped repainting.
#[tauri::command(async)]
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

// Runs off the window thread. A Tauri command declared without `async`
// is executed on the thread that pumps the window, so while this one
// worked the whole dashboard stopped repainting.
#[tauri::command(async)]
pub fn export_all_data(state: State<'_, Arc<AppState>>) -> R<String> {
    let mut v = state.shared.db.export_all().map_err(e)?;
    v["settings"] = serde_json::to_value(state.shared.settings.read().clone()).map_err(e)?;
    serde_json::to_string_pretty(&v).map_err(e)
}

// Runs off the window thread. A Tauri command declared without `async`
// is executed on the thread that pumps the window, so while this one
// worked the whole dashboard stopped repainting.
#[tauri::command(async)]
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
        // The hook is put back once a second because Windows removes it
        // silently; this counter shows the loop is alive.
        "hook_rehooks": crate::hotkey::hook_rehooks(),
    })
}

/// Problem lines from the newest log file, newest first, so the Diagnostics
/// page shows what went wrong without opening the log folder.
// Runs off the window thread. A Tauri command declared without `async`
// is executed on the thread that pumps the window, so while this one
// worked the whole dashboard stopped repainting.
#[tauri::command(async)]
pub fn recent_problems() -> Vec<String> {
    // Only the dated log files. The folder also holds the event journal and,
    // once the app has crashed even once, `panic.log`, whose name sorts after
    // every `lalia.log.<date>`. Taking the last name in the folder therefore
    // meant that from the first crash onwards this list showed old crash lines
    // and hid every warning and error of the running app.
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(crate::paths::logs_dir())
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.is_file()
                        && p.file_name().and_then(|n| n.to_str()).map(|n| n.starts_with("lalia.log.")).unwrap_or(false)
                })
                .collect()
        })
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
    let best = crate::hotkey::drop_altgr_companion(best);
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

// ---------------------------------------------------------------------------
// Hidden debug mode
//
// Nothing in the normal interface points at it: the Diagnostics title turns it
// on after five clicks. It exists so a problem reported by a stranger can be
// read without guessing, now that the app is public.
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn debug_mode_get(state: State<'_, Arc<AppState>>) -> bool {
    state.shared.settings.read().general.debug_mode
}

#[tauri::command]
pub fn debug_mode_set(app: tauri::AppHandle, on: bool, state: State<'_, Arc<AppState>>) -> R<()> {
    let mut s = state.shared.settings.read().clone();
    s.general.debug_mode = on;
    s.save(&crate::paths::settings_file()).map_err(e)?;
    *state.shared.settings.write() = s;
    crate::journal::set_verbose(on);
    crate::journal::info("debug.mode", serde_json::json!({ "on": on }));
    tracing::info!("debug mode {}", if on { "on" } else { "off" });
    // Without this the Settings screen keeps an older copy in its hands, and the
    // next Save there quietly switches debug mode back off.
    let _ = app.emit_to("main", "lalia://settings-changed", ());
    Ok(())
}

/// The newest journal entries, newest first, as raw JSON lines.
#[tauri::command]
pub fn debug_events(limit: Option<usize>) -> Vec<String> {
    // `test.*` kinds come from the test suite writing into the real journal on
    // a developer machine. They are noise in front of a user.
    crate::journal::tail(limit.unwrap_or(200).min(2000))
        .into_iter()
        .filter(|l| !l.contains("\"kind\":\"test."))
        .collect()
}

/// The last modifier keys Windows delivered to the hook, newest first, for
/// the key check in Diagnostics.
#[tauri::command]
pub fn recent_keys() -> Vec<crate::hotkey::SeenKey> {
    crate::hotkey::recent_keys()
}

/// Everything needed to understand a problem, in one block of text: the
/// machine, the engine, the settings that matter and the recent events. Also
/// written to `logs\debug-bundle.txt` so it can be read without the app.
// Runs off the window thread. A Tauri command declared without `async`
// is executed on the thread that pumps the window, so while this one
// worked the whole dashboard stopped repainting.
#[tauri::command(async)]
pub fn debug_bundle(state: State<'_, Arc<AppState>>) -> R<String> {
    use std::fmt::Write as _;
    let mut out = String::new();
    let _ = writeln!(out, "Fuck You Flow {} debug bundle", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(out, "written {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
    let hw = crate::hw::detect();
    let _ = writeln!(out, "\n== machine ==\n{hw:#?}");
    {
        let s = state.shared.settings.read();
        let _ = writeln!(out, "\n== settings ==");
        let _ = writeln!(out, "hotkeys      {:?}", s.hotkeys);
        let _ = writeln!(out, "asr backend  {} model {}", s.asr.backend, s.asr.model_id);
        let _ = writeln!(out, "threads      {}  beam {}", s.asr.threads, s.asr.beam_size);
        let _ = writeln!(out, "segmenting   {}", s.asr.segment_while_speaking);
        let _ = writeln!(out, "paste settle {} ms", s.insertion.paste_settle_ms);
        let _ = writeln!(out, "debug mode   {}", s.general.debug_mode);
    }
    let _ = writeln!(out, "\n== problems recorded this run: {} ==", crate::journal::problem_count());
    let _ = writeln!(out, "\n== last 300 events (newest first) ==");
    for line in crate::journal::tail(300) {
        let _ = writeln!(out, "{line}");
    }
    let _ = writeln!(out, "\n== last 60 log lines flagged as problems ==");
    for line in recent_problems().into_iter().take(60) {
        let _ = writeln!(out, "{line}");
    }
    // Crashes leave nothing anywhere else: the buffered log dies with the
    // process and Windows Error Reporting can be switched off. This file is
    // written by the panic hook itself, so it is the only account of a sudden
    // death and belongs at the top of anything sent to us.
    if let Ok(panics) = std::fs::read_to_string(crate::paths::logs_dir().join("panic.log")) {
        let _ = writeln!(out, "\n== crashes recorded (panic.log) ==");
        for line in panics.lines().rev().take(20) {
            let _ = writeln!(out, "{line}");
        }
    }
    let path = crate::paths::logs_dir().join("debug-bundle.txt");
    if let Err(e) = std::fs::write(&path, &out) {
        tracing::warn!("could not write {}: {e}", path.display());
    }
    Ok(out)
}

// ----- a sound file the user already has -----

/// Turn a recording on disk into text. Decoding, cutting and transcription all
/// happen here rather than in the dictation pipeline, because there is no
/// window to insert into, no key held down and no hurry: a long recording is
/// allowed to take a minute and to report how far it has got.
///
/// The result lands in the notepad window, in the clipboard and in History,
/// so it is reachable three ways and cannot be lost.
#[tauri::command]
pub async fn transcribe_audio_file(app: tauri::AppHandle, state: State<'_, Arc<AppState>>, path: String) -> R<String> {
    let file = std::path::PathBuf::from(&path);
    let name = file.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.clone());
    let started = std::time::Instant::now();
    let progress = |app: &tauri::AppHandle, done: usize, total: usize, phase: &str| {
        let _ = app.emit_to("main", "lalia://file-progress", serde_json::json!({ "done": done, "total": total, "phase": phase, "file": name }));
    };

    progress(&app, 0, 1, "reading");
    let f2 = file.clone();
    let samples = tokio::task::spawn_blocking(move || crate::audiofile::decode_to_engine_rate(&f2)).await.map_err(e)??;
    let audio_ms = (samples.len() as u64 * 1000) / crate::audio::TARGET_RATE as u64;

    let (settings, engine) = {
        let s = state.shared.settings.read().clone();
        (s, state.shared.engine.clone())
    };
    let lang = settings.language.mode.whisper_code().to_string();
    // Two minutes a piece: long enough that the engine keeps its context, short
    // enough that a failure costs little and progress moves visibly.
    let cuts = crate::audiofile::cut_points(samples.len(), 120, &samples);
    let total = cuts.len();
    let mut raw = String::new();
    let mut engine_name = String::new();
    let mut model_name = String::new();
    let mut inference_ms = 0u64;
    let mut start = 0usize;
    for (i, end) in cuts.iter().copied().enumerate() {
        progress(&app, i, total, "transcribing");
        let piece = &samples[start..end];
        start = end;
        if crate::audio::is_silent(piece) {
            continue;
        }
        let wav = crate::audio::encode_wav(piece);
        // The tail of what came before is the recognition hint, exactly as it
        // is while dictating: names and endings carry across a cut that way.
        let tail: String = raw.chars().rev().take(200).collect::<Vec<_>>().into_iter().rev().collect();
        let req = crate::asr::TranscriptionRequest {
            wav,
            language: lang.clone(),
            prompt: if tail.trim().is_empty() { None } else { Some(tail.trim().to_string()) },
            beam_size: settings.asr.beam_size,
            vad: settings.asr.vad,
        };
        match engine.transcribe(req).await {
            Ok(t) => {
                if !t.text.trim().is_empty() {
                    if !raw.is_empty() {
                        raw.push(' ');
                    }
                    raw.push_str(t.text.trim());
                }
                engine_name = t.engine;
                model_name = t.model;
                inference_ms += t.inference_ms;
            }
            Err(err) => {
                tracing::error!("file transcription failed on piece {}/{total}: {err}", i + 1);
                return Err(format!("piece {} of {total} could not be transcribed: {err}", i + 1));
            }
        }
    }
    if raw.trim().is_empty() {
        return Err("no speech was found in this recording".into());
    }

    progress(&app, total, total, "cleaning");
    let opts = crate::cleanup::CleanupOptions {
        intensity: settings.cleanup.intensity.clone(),
        remove_fillers: settings.cleanup.remove_fillers,
        resolve_self_corrections: settings.cleanup.resolve_self_corrections,
        auto_punctuate: settings.cleanup.auto_punctuate,
        auto_capitalize: settings.cleanup.auto_capitalize,
        trailing_punctuation: true,
        capitalize_first: true,
        language: if lang == "auto" {
            if crate::cleanup::deterministic::looks_greek(&raw) { "el".into() } else { "en".into() }
        } else {
            lang.clone()
        },
    };
    let outcome = {
        let dict = state.shared.dict.read();
        let snips = state.shared.snippets.read();
        crate::cleanup::run_deterministic(&raw, &opts, &dict, &snips)
    };
    let text = outcome.cleaned.trim().to_string();

    // On the clipboard straight away: the user asked for text, and text they
    // can paste is the whole point.
    let t2 = text.clone();
    let _ = tokio::task::spawn_blocking(move || crate::insertion::copy_only(&t2)).await;

    if settings.privacy.keep_history {
        let word_count = crate::cleanup::deterministic::word_count(&text);
        let entry = HistoryEntry {
            id: crate::db::new_id(),
            created_at: crate::db::ts_now(),
            raw_text: raw.clone(),
            cleaned_text: outcome.cleaned.clone(),
            final_text: text.clone(),
            language: lang.clone(),
            detected_language: None,
            app_name: Some("Sound file".into()),
            app_process: Some(name.clone()),
            app_category: Some("file".into()),
            cleanup_mode: format!("{:?}", settings.cleanup.intensity).to_lowercase(),
            rules_applied: outcome.applied.clone(),
            audio_ms,
            latency_ms: started.elapsed().as_millis() as u64,
            inference_ms,
            word_count,
            engine: Some(engine_name),
            model: Some(model_name),
            insertion_method: Some("file".into()),
            status: "success".into(),
            audio_path: Some(path.clone()),
            context_used: false,
            retried: false,
            undone: false,
            edited_text: None,
        };
        if let Err(err) = state.shared.db.insert_history(&entry) {
            tracing::error!("history insert failed for {name}: {err}");
        }
        let _ = app.emit_to("main", "lalia://history-changed", ());
        let _ = state.shared.db.bump_daily(word_count, audio_ms, outcome.rule_ids.len() as u64);
    }

    tracing::info!("file transcribed: {name}, {} s audio, {} pieces, {} ms inference, {} chars", audio_ms / 1000, total, inference_ms, text.chars().count());
    crate::journal::info(
        "file.transcribed",
        serde_json::json!({ "audio_ms": audio_ms, "pieces": total, "inference_ms": inference_ms, "chars": text.chars().count(), "took_ms": started.elapsed().as_millis() as u64 }),
    );
    crate::scratch::show(&app, &text, "file", Some(name.clone()), "file");
    Ok(text)
}

/// What the notepad window should hold right now, for a webview that has just
/// come up and missed the event.
#[tauri::command]
pub fn scratch_last() -> Option<crate::scratch::ScratchPayload> {
    crate::scratch::last()
}

/// The file endings the picker offers.
#[tauri::command]
pub fn audio_file_extensions() -> Vec<String> {
    crate::audiofile::SUPPORTED.iter().map(|s| s.to_string()).collect()
}

/// The native "choose a file" box, filtered to sound the app can read. Done in
/// Rust so the interface needs no extra package for it.
#[tauri::command]
pub async fn pick_audio_file(app: tauri::AppHandle) -> R<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .add_filter("Sound", crate::audiofile::SUPPORTED)
        .pick_file(move |chosen| {
            let _ = tx.send(chosen);
        });
    let picked = rx.await.map_err(e)?;
    Ok(picked.map(|p| p.to_string()))
}

/// The "do not show this again" box at the bottom of the notepad window. It is
/// the same switch as the one in Settings, reachable at the moment the window
/// is in the way, which is the only moment a user actually wants it.
#[tauri::command]
pub async fn set_notepad_when_lost(app: tauri::AppHandle, state: State<'_, Arc<AppState>>, enabled: bool) -> R<()> {
    let mut settings = state.shared.settings.read().clone();
    settings.insertion.notepad_when_lost = enabled;
    settings.save(&crate::paths::settings_file()).map_err(e)?;
    *state.shared.settings.write() = settings.clone();
    let _ = state.shared.tx.send(PipelineMsg::SettingsChanged);
    let _ = app.emit_to("main", "lalia://settings-changed", ());
    tracing::info!("notepad when text is lost: {}", if enabled { "on" } else { "off" });
    Ok(())
}

// ----- updates -----

/// The version of the running program. Answered from the binary itself, so the
/// Settings page can show it without asking the server anything.
#[tauri::command]
pub fn app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

/// What the UI shows when it asks "there is a new version, shall I get it?".
#[derive(Debug, Clone, serde::Serialize)]
pub struct UpdateInfo {
    pub available: bool,
    /// The version waiting on the server, empty when there is none.
    pub version: String,
    pub current: String,
    /// The release notes, as written in the manifest.
    pub notes: Option<String>,
    pub date: Option<String>,
    /// False when the models still sit inside the install folder. The update
    /// that is published carries the program alone, so installing it would
    /// take the models with the old version and leave the app with no engine
    /// to speak of. In that state the user is sent to the site for the full
    /// installer, and `install_update` refuses.
    pub small_download: bool,
}

/// What the last check found, kept so that the install button gets exactly the
/// version the user was shown. Asking the server a second time could hand them
/// a different one, or nothing at all if the server hiccups in between.
static FOUND_UPDATE: std::sync::Mutex<Option<tauri_plugin_updater::Update>> = std::sync::Mutex::new(None);

/// True while a download is on its way. The check runs on its own fifteen
/// seconds after any window opens, so without this it could land in the middle
/// of an install and swap the answer under it.
static INSTALLING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

async fn look_for_update(app: &tauri::AppHandle) -> R<Option<tauri_plugin_updater::Update>> {
    use tauri_plugin_updater::UpdaterExt;
    app.updater().map_err(e)?.check().await.map_err(e)
}

/// Asks the update server whether a newer version exists. Downloads nothing.
#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle) -> R<UpdateInfo> {
    let current = app.package_info().version.to_string();
    let found = look_for_update(&app).await?;
    let info = match &found {
        Some(u) => UpdateInfo {
            available: true,
            version: u.version.clone(),
            current,
            notes: u.body.clone(),
            date: u.date.map(|d| d.to_string()),
            small_download: crate::models::models_are_external(),
        },
        None => UpdateInfo { available: false, version: String::new(), current, notes: None, date: None, small_download: crate::models::models_are_external() },
    };
    if INSTALLING.load(std::sync::atomic::Ordering::Relaxed) {
        tracing::info!("an install is under way; the answer already in hand stays");
    } else {
        *FOUND_UPDATE.lock().map_err(|_| "update lock")? = found;
    }
    tracing::info!("update check: {} (running {})", if info.available { info.version.as_str() } else { "nothing newer" }, info.current);
    crate::journal::info("update.checked", serde_json::json!({ "available": info.available, "version": info.version }));
    Ok(info)
}

/// Downloads and installs the update the user just said yes to. The installer
/// closes the app and starts the new one, so this call normally never returns.
#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> R<()> {
    // The published update has no models in it. Installing it over a copy
    // whose models still live in the install folder would delete them, and
    // the app would come back up with nothing to transcribe with.
    if !crate::models::models_are_external() {
        tracing::warn!("update refused: the models are still inside the install folder");
        return Err("this copy has to be updated from the site: fuckyouflow.app".into());
    }
    // Whatever the last check found, and nothing else.
    INSTALLING.store(true, std::sync::atomic::Ordering::Relaxed);
    let update = FOUND_UPDATE.lock().map_err(|_| "update lock")?.take();
    let Some(update) = update else {
        INSTALLING.store(false, std::sync::atomic::Ordering::Relaxed);
        return Err("ask the server first".into());
    };
    let version = update.version.clone();
    tracing::info!("downloading update {version}");
    let got = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let app2 = app.clone();
    let got2 = got.clone();
    let got_it = update
        .download(
            move |chunk, total| {
                let sofar = got2.fetch_add(chunk as u64, std::sync::atomic::Ordering::Relaxed) + chunk as u64;
                let _ = app2.emit("lalia://update-progress", serde_json::json!({ "received": sofar, "total": total }));
            },
            || {
                tracing::info!("update downloaded, handing over to the installer");
            },
        )
        .await;
    if let Err(err) = got_it {
        // Put it back, so the button works on a second try without another
        // trip to the server.
        if let Ok(mut slot) = FOUND_UPDATE.lock() {
            *slot = Some(update);
        }
        INSTALLING.store(false, std::sync::atomic::Ordering::Relaxed);
        return Err(e(err));
    }
    crate::journal::info("update.installing", serde_json::json!({ "version": version }));
    let _ = app.emit("lalia://update-progress", serde_json::json!({ "received": 0, "total": 0, "installing": true }));
    // The installer ends this process itself, so the normal way out never
    // runs. Take the tray icon down here, or it stays painted on the taskbar
    // until the user waves the mouse over it.
    crate::app::drop_tray(&app);
    if let Err(err) = update.install(got_it.expect("checked just above")) {
        if let Ok(mut slot) = FOUND_UPDATE.lock() {
            *slot = Some(update);
        }
        INSTALLING.store(false, std::sync::atomic::Ordering::Relaxed);
        return Err(e(err));
    }
    Ok(())
}
