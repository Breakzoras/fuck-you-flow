pub mod app;
pub mod asr;
pub mod audio;
pub mod audiofile;
pub mod cleanup;
pub mod commands;
pub mod context;
pub mod db;
pub mod engine;
pub mod hotkey;
pub mod hw;
pub mod insertion;
pub mod jobobject;
pub mod journal;
pub mod languages;
pub mod learning;
pub mod local_api;
pub mod logging;
pub mod milestones;
pub mod models;
pub mod overlay;
pub mod packs;
pub mod paths;
pub mod pipeline;
pub mod scratch;
pub mod settings;
pub mod sharing;

use tauri::Manager;

/// The last thing the app does when it cannot start.
///
/// A program that fails before its window exists says nothing at all: the
/// error goes to a console that a windowed program does not have, and the icon
/// in the taskbar simply never appears. Somebody double clicks and nothing
/// happens, twice, and then they give up. This puts the reason on the screen
/// and writes it next to the crash log so it can be read again later.
#[cfg(windows)]
fn say_it_cannot_start(reason: &str) {
    use std::io::Write;
    let path = paths::logs_dir().join("panic.log");
    let line = format!(
        "{} CANNOT START: {reason}\n",
        chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%:z")
    );
    let _ = std::fs::create_dir_all(paths::logs_dir());
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
        let _ = f.flush();
    }
    tracing::error!("cannot start: {reason}");

    let text = format!(
        "Fuck You Flow could not start.\n\n{reason}\n\nThe details are in:\n{}",
        path.display()
    );
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let title: Vec<u16> = "Fuck You Flow".encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        use windows::core::PCWSTR;
        use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK, MB_SETFOREGROUND};
        MessageBoxW(
            None,
            PCWSTR(wide.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK | MB_ICONERROR | MB_SETFOREGROUND,
        );
    }
}

#[cfg(not(windows))]
fn say_it_cannot_start(reason: &str) {
    tracing::error!("cannot start: {reason}");
    eprintln!("Fuck You Flow could not start: {reason}");
}

/// The tag of a panic.log line written while Windows shut down. Problem
/// reports leave these out of their list of crashes.
pub(crate) const SHUTDOWN_MARK: &str = "WINDOWS SHUTDOWN, not a crash";

fn panic_text(info: &std::panic::PanicHookInfo<'_>) -> String {
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        String::new()
    }
}

/// tao 0.35 on Windows: after WM_ENDSESSION the event loop is "Destroyed", and
/// any message after that panics with this text (panic.log, 28 September,
/// 16 and 21 October 2026, each a second after the shutdown event 1074).
fn is_windows_shutdown_panic(text: &str) -> bool {
    text.contains("cannot move state from Destroyed")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::init();
    tracing::info!("Fuck You Flow {} starting", env!("CARGO_PKG_VERSION"));
    std::panic::set_hook(Box::new(|info| {
        // Write it to disk here and now, with our own hands. The normal log goes
        // through a buffered writer on another thread (tracing_appender's
        // non_blocking), and `panic = "abort"` kills the process before that
        // thread ever gets to flush, so the line below is the only one that
        // survives. Windows Error Reporting cannot be relied on either: it is
        // switched off on the development machine, which is why the crash of
        // 7 September 2026 at 19:15:37, in the middle of a dictation, left
        // nothing behind at all.
        use std::io::Write;
        let shutdown = is_windows_shutdown_panic(&panic_text(info));
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%:z");
        let line = if shutdown {
            format!("{now} {SHUTDOWN_MARK}: {info}\n")
        } else {
            format!("{now} PANIC: {info}\n")
        };
        let path = paths::logs_dir().join("panic.log");
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
            let _ = f.write_all(line.as_bytes());
            let _ = f.flush();
        }
        if shutdown {
            // Windows is ending the session and the window library has already
            // torn its event loop down; one more message reaches it and it
            // panics. Nothing is lost at that point, so leave quietly, which is
            // what tao 0.37 does itself on WM_ENDSESSION (a version this build
            // cannot take without a tauri upgrade).
            std::process::exit(0);
        }
        tracing::error!("PANIC: {info}");
    }));

    let built = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--autostart"])))
        .setup(|app| {
            if let Err(e) = app::build(app) {
                say_it_cannot_start(&format!("{e:#}"));
                return Err(e.into());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the dashboard hides it; the app lives in the tray.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::overlay_dock,
            commands::overlay_preview,
            commands::save_settings,
            commands::get_pipeline_snapshot,
            commands::pipeline_toggle,
            commands::pipeline_cancel,
            commands::pipeline_retry,
            commands::paste_last,
            commands::paste_history,
            commands::list_microphones,
            commands::mic_level,
            commands::mic_test_open,
            commands::engine_info,
            commands::engine_restart,
            commands::local_api_status,
            commands::list_models,
            commands::runtime_status,
            commands::gpu_gauge,
            commands::download_model,
            commands::install_runtime,
            commands::remove_model,
            commands::verify_model,
            commands::set_cloud_api_key,
            commands::has_cloud_api_key,
            commands::list_history,
            commands::delete_history,
            commands::delete_all_history,
            commands::reclean_history,
            commands::record_edit,
            commands::list_rules,
            commands::save_rule,
            commands::delete_rule,
            commands::add_rule_exception,
            commands::test_rules,
            commands::import_rules,
            commands::list_snippets,
            commands::save_snippet,
            commands::delete_snippet,
            commands::list_suggestions,
            commands::resolve_suggestion,
            commands::accept_suggestion_as,
            commands::delete_learning_data,
            commands::list_app_styles,
            commands::save_app_style,
            commands::delete_app_style,
            commands::get_stats,
            commands::get_level,
            commands::get_milestones,
            commands::export_all_data,
            commands::delete_all_data,
            commands::open_data_folder,
            commands::open_logs_folder,
            commands::diagnostics,
            commands::recent_problems,
            commands::debug_mode_get,
            commands::debug_mode_set,
            commands::debug_events,
            commands::debug_bundle,
            commands::report_preview,
            commands::send_report,
            commands::share_dictionary_preview,
            commands::share_dictionary_set,
            commands::mark_rated_on_github,
            commands::share_dictionary_forget,
            commands::pack_status,
            commands::pack_install,
            commands::pack_decline,
            commands::pack_notice_seen,
            commands::light_model_offer,
            commands::light_model_accept,
            commands::light_model_decline,
            commands::pack_remove,
            commands::recent_keys,
            commands::current_foreground_app,
            commands::record_shortcut,
            commands::show_main_window,
            commands::quit_app,
            commands::transcribe_audio_file,
            commands::audio_file_extensions,
            commands::scratch_last,
            commands::pick_audio_file,
            commands::set_notepad_when_lost,
            commands::app_version,
            commands::linux_input_status,
            commands::check_for_update,
            commands::install_update,
        ])
        .build(tauri::generate_context!());
    let app = match built {
        Ok(a) => a,
        Err(e) => {
            say_it_cannot_start(&format!("{e}"));
            std::process::exit(1);
        }
    };
    app
        .run(|app, event| {
            // Always take the tray icon down on the way out. A tray icon whose
            // owner disappears without removing it stays painted on the taskbar
            // until the user waves the mouse over it ("ghost icon").
            if matches!(event, tauri::RunEvent::Exit) {
                app::drop_tray(app);
                // The last line of a normal run. Its absence at the end of a
                // day's log means the process was killed rather than closed,
                // which is the only way to tell a crash from a quit here:
                // panics write panic.log, and Windows Error Reporting is off.
                tracing::info!("Fuck You Flow exiting cleanly");
            }
        });
}

#[cfg(test)]
mod shutdown_panic_tests {
    #[test]
    fn only_the_destroyed_event_loop_panic_counts_as_a_shutdown() {
        assert!(super::is_windows_shutdown_panic("cannot move state from Destroyed"));
        assert!(!super::is_windows_shutdown_panic("overflow when subtracting duration from instant"));
        assert!(!super::is_windows_shutdown_panic(""));
    }
}
