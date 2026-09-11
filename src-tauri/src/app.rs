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

/// Windows starts whatever path the entry names, so only the installed copy may
/// ever write it. A run from the build folder used to claim the entry, and then
/// every boot started that build while the installed copy sat unused and could
/// never update itself (measured 10 September 2026). Switching autostart off is
/// always allowed, from any copy.
/// Returns whether the Windows entry now matches what was asked. A caller that
/// gets `false` must not leave the switch showing on: it did nothing, and a
/// switch that lies is worse than one that refuses.
pub fn set_autostart(app: &tauri::AppHandle, enabled: bool) -> bool {
    if enabled && !is_installed_copy() {
        tracing::info!(
            "startup entry left alone: this copy is not the installed one ({})",
            std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default()
        );
        return false;
    }
    let manager = app.autolaunch();
    let r = if enabled { manager.enable() } else { manager.disable() };
    if let Err(e) = r {
        tracing::warn!("autostart change failed: {e}");
        return false;
    }
    true
}

/// Whether `exe` is the copy the installer put down. The installer leaves its
/// uninstaller next to the program and nothing else does, so that file is the
/// marker. A run from the build folder or a loose copy has no such neighbour.
pub fn installed_marker(exe: &std::path::Path) -> bool {
    exe.parent().map(|dir| dir.join("uninstall.exe").is_file()).unwrap_or(false)
}

fn is_installed_copy() -> bool {
    std::env::current_exe().map(|p| installed_marker(&p)).unwrap_or(false)
}

#[cfg(test)]
mod startup_tests {
    use super::installed_marker;

    #[test]
    fn only_a_folder_with_the_uninstaller_counts_as_installed() {
        let dir = std::env::temp_dir().join(format!("fuckyouflow-marker-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("fuckyouflow.exe");
        assert!(!installed_marker(&exe), "a bare folder is a developer run");
        std::fs::write(dir.join("uninstall.exe"), b"").unwrap();
        assert!(installed_marker(&exe), "the uninstaller next to it marks the installed copy");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

pub fn build(app: &tauri::App) -> anyhow::Result<()> {
    crate::paths::ensure_all()?;
    // The installer ships the engine and the models next to the executable; the
    // app must know that folder before anything looks for either of them.
    let resource_dir = match app.path().resource_dir() {
        Ok(res) => {
            let bundled = res.join("bundled");
            if bundled.exists() {
                tracing::info!("bundled engine and models found at {}", bundled.display());
                crate::paths::set_bundled_dir(bundled);
            }
            Some(res)
        }
        Err(err) => {
            // Without this we cannot tell an installed copy from a developer
            // run, and the update flow has to assume the worst.
            tracing::warn!("cannot find where the program lives: {err}");
            None
        }
    };
    // Get the models out of the install folder before anything looks for them.
    // On the same disk this is a rename and costs nothing; see the function for
    // why it has to happen at all.
    if !crate::models::migrate_bundled_models(resource_dir.as_deref()) {
        tracing::warn!("a model is still inside the install folder; this copy updates from the site");
    }
    // A fresh install has no settings file yet. That is the only moment the
    // machine's own language may choose the defaults; after it, the user's
    // choice stands and is never overwritten.
    let fresh_install = !crate::paths::settings_file().exists();
    let mut settings = Settings::load(&crate::paths::settings_file());
    crate::journal::set_verbose(settings.general.debug_mode);
    // Point the Windows startup entry at this copy. It used to be written only
    // when the user saved the settings, so after an install it still named the
    // old path and Windows started a program that was no longer there
    // (8 September 2026). `set_autostart` decides whether this copy is allowed
    // to claim the entry.
    if settings.general.autostart {
        set_autostart(&app.handle().clone(), true);
    }
    if fresh_install {
        let locale = crate::hw::user_locale();
        let greek = locale.starts_with("el");
        settings.general.ui_language = if greek { "el" } else { "en" }.into();
        // A Greek speaker mixes English words into Greek sentences all day, so
        // the bilingual mode earns its cost there. Everyone else gets their own
        // language alone, which is both faster and more accurate.
        settings.language.mode = if greek {
            crate::settings::LanguageMode::Multi
        } else if locale.starts_with("en") {
            crate::settings::LanguageMode::English
        } else {
            crate::settings::LanguageMode::Auto
        };
        tracing::info!("fresh install: locale {locale} -> interface {}, dictation {:?}", settings.general.ui_language, settings.language.mode);
        crate::journal::info("app.first_run", serde_json::json!({ "locale": locale, "ui": settings.general.ui_language, "dictation": format!("{:?}", settings.language.mode) }));
        let _ = settings.save(&crate::paths::settings_file());
    }
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
        // The one record that tells us, from a stranger's machine, what the app
        // decided on its own and whether that decision was right.
        crate::journal::info(
            "machine.profiled",
            serde_json::json!({
                "gpu": hw.best_gpu().map(|g| g.name.clone()),
                "vendor": hw.best_gpu().map(|g| g.vendor.clone()),
                "vram_mb": hw.best_gpu().map(|g| g.vram_mb),
                "logical_cores": hw.logical_cores,
                "ram_mb": hw.ram_mb,
                "vulkan": hw.vulkan_runtime,
                "cuda": hw.cuda_driver,
                "chose_threads": settings.asr.threads,
                "chose_model": settings.asr.model_id,
            }),
        );
    }
    // The configured model is not on disk (fresh install): take a shipped one.
    if let Some(id) = crate::models::preferred_installed_model(&settings.asr.model_id) {
        tracing::info!("model {} is not present; switching to the bundled {}", settings.asr.model_id, id);
        settings.asr.model_id = id;
        changed = true;
    }
    // A graphics card that is sitting right there should be doing the work.
    // Any modern card can, through Vulkan, AMD and Intel included; the engine
    // runs roughly six times faster on one than on the processor. The first
    // AMD tester, on 7 September 2026, found the card switched off with the
    // engine reported as missing and had to turn it on by hand before the app
    // did anything useful. So: correct it at startup, once, and say so in the
    // journal. A user who deliberately turned the card off is never overruled
    // (`gpu_choice_by_user`).
    {
        let vulkan_build = crate::asr::whisper_server::find_runtime_exe_for("vulkan").is_some();
        if crate::hw::should_switch_to_gpu(
            settings.asr.use_gpu,
            &settings.asr.backend,
            settings.general.gpu_choice_by_user,
            hw.best_gpu().is_some(),
            hw.vulkan_runtime,
            vulkan_build,
        ) {
            let card = hw.best_gpu().map(|g| g.name.clone()).unwrap_or_default();
            settings.asr.use_gpu = true;
            settings.asr.backend = "auto".into();
            changed = true;
            tracing::info!("graphics card found ({card}) while the engine was set to the processor; switching it on");
            crate::journal::info(
                "gpu.auto_enabled",
                serde_json::json!({ "card": card, "vendor": hw.best_gpu().map(|g| g.vendor.clone()), "vulkan": hw.vulkan_runtime, "cuda": hw.cuda_driver }),
            );
        }
    }
    if changed {
        let _ = settings.save(&crate::paths::settings_file());
    }
    let settings = settings;
    crate::logging::set_redaction(settings.privacy.redact_logs);
    let db = Arc::new(Db::open(&crate::paths::db_file())?);
    // A history an older build wrote under the old folder name, after the move.
    for orphan in crate::paths::take_orphan_histories() {
        match db.merge_from(&orphan) {
            Ok(n) => {
                tracing::info!("merged {n} dictations from a second history at {}", orphan.display());
                crate::journal::info("profile.merged", serde_json::json!({ "rows": n }));
            }
            Err(err) => {
                tracing::warn!("could not merge the history at {}: {err}", orphan.display());
                crate::journal::warn("profile.merge_failed", serde_json::json!({ "error": err.to_string() }));
            }
        }
    }

    // retention
    if let Some(days) = settings.privacy.retention_days {
        if let Ok(paths) = db.delete_history_older_than(days) {
            for p in paths {
                crate::paths::remove_kept_audio(&p);
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
/// closing the app never leaves an icon painted on the taskbar.
pub fn drop_tray(app: &tauri::AppHandle) {
    let _ = app.remove_tray_by_id("main");
    let icon = TRAY.lock().take();
    drop(icon);
}

/// The single tray icon. It is built here and nowhere else: declaring
/// `app.trayIcon` in tauri.conf.json as well makes Tauri create a second,
/// menu-less icon, and the user sees two of our icons in the notification area.
fn build_tray(app: &tauri::App) -> anyhow::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Fuck You Flow", true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, "toggle", "Start / stop dictation", true, None::<&str>)?;
    let paste = MenuItem::with_id(app, "paste_last", "Paste last transcript", true, None::<&str>)?;
    let check = MenuItem::with_id(app, "check_update", "Check for updates", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &toggle, &paste, &check, &sep, &quit])?;
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
                "check_update" => {
                    // The dashboard asks the server and says what it found; it
                    // comes forward on its own when there is a new version.
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.unminimize();
                        let _ = w.set_focus();
                        let _ = w.emit("lalia://check-update", ());
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
