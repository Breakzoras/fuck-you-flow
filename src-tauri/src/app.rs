//! Application wiring: state, tray icon, hotkeys, autostart, startup sequence.

use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::audio::AudioCapture;
use crate::cleanup::dictionary::DictionaryEngine;
use crate::cleanup::snippets::SnippetEngine;
use crate::db::Db;
use crate::engine::EngineManager;
use crate::hotkey::{Chord, ChordId};
use crate::pipeline::{Phase, PipelineMsg, PipelineSnapshot, Shared};
use crate::settings::Settings;

pub struct AppState {
    pub shared: Arc<Shared>,
}

pub fn apply_hotkeys(settings: &Settings) {
    let mut list = Vec::new();
    if let Ok(c) = Chord::parse(&settings.hotkeys.push_to_talk) {
        list.push((ChordId::PushToTalk, c));
    }
    if let Ok(c) = Chord::parse(&settings.hotkeys.hands_free) {
        list.push((ChordId::HandsFree, c));
    }
    if let Ok(c) = Chord::parse(&settings.hotkeys.paste_last) {
        list.push((ChordId::PasteLast, c));
    }
    crate::hotkey::set_bindings(list);
}

pub fn reload_engines(state: &AppState) {
    let rules = state.shared.db.list_rules().unwrap_or_default();
    let exceptions = state.shared.db.list_rule_exceptions().unwrap_or_default();
    *state.shared.dict.write() = DictionaryEngine::new(rules, exceptions);
    let snippets = state.shared.db.list_snippets().unwrap_or_default();
    *state.shared.snippets.write() = SnippetEngine::new(snippets);
}

pub fn set_autostart(app: &tauri::AppHandle, enabled: bool) {
    let manager = app.autolaunch();
    let r = if enabled { manager.enable() } else { manager.disable() };
    if let Err(e) = r {
        tracing::warn!("autostart change failed: {e}");
    }
}

pub fn build(app: &tauri::App) -> anyhow::Result<()> {
    crate::paths::ensure_all()?;
    // The installer ships the engine and the models next to the executable; the
    // app must know that folder before anything looks for either of them.
    if let Ok(res) = app.path().resource_dir() {
        let bundled = res.join("bundled");
        if bundled.exists() {
            tracing::info!("bundled engine and models found at {}", bundled.display());
            crate::paths::set_bundled_dir(bundled);
        }
    }
    let mut settings = Settings::load(&crate::paths::settings_file());
    crate::journal::set_verbose(settings.general.debug_mode);
    let hw = crate::hw::detect();
    tracing::info!("machine: {}", hw.summary());
    crate::journal::info(
        "app.start",
        serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "machine": hw.summary(),
            "debug_mode": settings.general.debug_mode,
        }),
    );
    let mut changed = false;
    // First start on this machine: threads and model from the hardware.
    if !settings.general.machine_profiled {
        settings.asr.threads = hw.engine_threads();
        if let Some(g) = hw.best_gpu() {
            if g.vram_mb < 3000 && settings.asr.model_id == "large-v3-q5_0" {
                settings.asr.model_id = "large-v3-turbo-q5_0".into();
            }
        } else {
            settings.asr.model_id = "large-v3-turbo-q5_0".into();
        }
        settings.general.machine_profiled = true;
        changed = true;
        tracing::info!("machine profiled: {} threads, model {}", settings.asr.threads, settings.asr.model_id);
    }
    // The configured model is not on disk (fresh install): take a shipped one.
    if let Some(id) = crate::models::preferred_installed_model(&settings.asr.model_id) {
        tracing::info!("model {} is not present; switching to the bundled {}", settings.asr.model_id, id);
        settings.asr.model_id = id;
        changed = true;
    }
    if changed {
        let _ = settings.save(&crate::paths::settings_file());
    }
    let settings = settings;
    crate::logging::set_redaction(settings.privacy.redact_logs);
    let db = Arc::new(Db::open(&crate::paths::db_file())?);

    // retention
    if let Some(days) = settings.privacy.retention_days {
        if let Ok(paths) = db.delete_history_older_than(days) {
            for p in paths {
                let _ = std::fs::remove_file(p);
            }
        }
    }

    let audio = AudioCapture::new(settings.audio.preroll_ms, settings.audio.max_recording_seconds);
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<PipelineMsg>();
    let shared = Arc::new(Shared {
        settings: Arc::new(RwLock::new(settings.clone())),
        db: db.clone(),
        audio: audio.clone(),
        engine: Arc::new(EngineManager::new()),
        dict: Arc::new(RwLock::new(DictionaryEngine::empty())),
        snippets: Arc::new(RwLock::new(SnippetEngine::empty())),
        snapshot: Arc::new(Mutex::new(PipelineSnapshot { phase: Phase::Idle, hands_free: false, last_error: None, last_transcript: None, mic_open: false })),
        tx: tx.clone(),
    });
    let state = Arc::new(AppState { shared: shared.clone() });
    reload_engines(&state);
    app.manage(state.clone());

    // hotkeys: forward to the pipeline
    let (hk_tx, mut hk_rx) = tokio::sync::mpsc::unbounded_channel::<crate::hotkey::HotkeyEvent>();
    apply_hotkeys(&settings);
    crate::hotkey::install(hk_tx);
    let tx2 = tx.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(ev) = hk_rx.recv().await {
            tracing::debug!("hotkey event {ev:?}");
            let _ = tx2.send(PipelineMsg::Hotkey(ev));
        }
    });

    // clipboard owner thread, warm early
    crate::insertion::ensure_started();

    // pipeline
    crate::pipeline::spawn(app.handle().clone(), shared.clone(), rx);
    let _ = tx.send(PipelineMsg::SettingsChanged);

    // engine start + watchdog
    {
        let handle = app.handle().clone();
        let shared2 = shared.clone();
        tauri::async_runtime::spawn(async move {
            let s = shared2.settings.read().clone();
            shared2.engine.apply(&handle, &s).await;
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(5));
            loop {
                tick.tick().await;
                let s = shared2.settings.read().clone();
                shared2.engine.ensure_alive(&handle, &s).await;
                // microphone watchdog: re-open if the warm stream stopped delivering
                let idle = shared2.snapshot.lock().phase == Phase::Idle;
                if s.audio.keep_stream_warm && idle && (!shared2.audio.is_open() || !shared2.audio.is_alive()) {
                    tracing::warn!("microphone stream went silent, re-opening");
                    let audio = shared2.audio.clone();
                    let dev = s.audio.device_name.clone();
                    let _ = tokio::task::spawn_blocking(move || audio.open(dev)).await;
                }
            }
        });
    }

    build_tray(app)?;

    // First run: show the dashboard so the user can finish setup.
    if !settings.general.first_run_done {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.show();
        }
    }
    Ok(())
}

/// Holds the one tray icon for the lifetime of the process. Tauri drops a tray
/// icon when its last handle goes away, so the handle is parked here.
static TRAY: Mutex<Option<tauri::tray::TrayIcon>> = Mutex::new(None);

/// Takes the icon out of the notification area. Called on every exit path, so a
/// closing Lalia never leaves an icon painted on the taskbar.
pub fn drop_tray(app: &tauri::AppHandle) {
    let _ = app.remove_tray_by_id("main");
    let icon = TRAY.lock().take();
    drop(icon);
}

/// The single tray icon. It is built here and nowhere else: declaring
/// `app.trayIcon` in tauri.conf.json as well makes Tauri create a second,
/// menu-less icon, and the user sees two Lalia icons in the notification area.
fn build_tray(app: &tauri::App) -> anyhow::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Fuck You Flow", true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, "toggle", "Start / stop dictation", true, None::<&str>)?;
    let paste = MenuItem::with_id(app, "paste_last", "Paste last transcript", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &toggle, &paste, &sep, &quit])?;
    let icon = match app.default_window_icon().cloned() {
        Some(i) => i,
        None => {
            tracing::error!("no default window icon; the tray icon cannot be created");
            anyhow::bail!("no icon");
        }
    };
    let tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("Fuck You Flow")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let state = app.state::<Arc<AppState>>();
            match event.id().as_ref() {
                "open" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.unminimize();
                        let _ = w.set_focus();
                    }
                }
                "toggle" => {
                    let _ = state.shared.tx.send(PipelineMsg::Toggle);
                }
                "paste_last" => {
                    let _ = state.shared.tx.send(PipelineMsg::PasteLast);
                }
                "quit" => {
                    let _ = state.shared.tx.send(PipelineMsg::Shutdown);
                    // Remove the icon before anything can fail, so quitting never
                    // leaves a ghost behind.
                    drop_tray(app);
                    let engine = state.shared.engine.clone();
                    let app2 = app.clone();
                    tauri::async_runtime::spawn(async move {
                        engine.stop().await;
                        app2.exit(0);
                    });
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.unminimize();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;
    tracing::info!("tray icon created (id {:?})", tray.id());
    if TRAY.lock().replace(tray).is_some() {
        tracing::warn!("tray icon was built twice; the older one is dropped");
    }
    Ok(())
}

pub fn notify_main(app: &tauri::AppHandle, event: &str) {
    let _ = app.emit_to("main", event, ());
}
