pub mod app;
pub mod asr;
pub mod audio;
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
pub mod learning;
pub mod logging;
pub mod models;
pub mod overlay;
pub mod paths;
pub mod pipeline;
pub mod settings;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::init();
    tracing::info!("Lalia {} starting", env!("CARGO_PKG_VERSION"));
    std::panic::set_hook(Box::new(|info| {
        tracing::error!("PANIC: {info}");
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--autostart"])))
        .setup(|app| {
            app::build(app)?;
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
            commands::list_models,
            commands::runtime_status,
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
            commands::delete_learning_data,
            commands::list_app_styles,
            commands::save_app_style,
            commands::delete_app_style,
            commands::get_stats,
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
            commands::current_foreground_app,
            commands::record_shortcut,
            commands::show_main_window,
            commands::quit_app,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // Always take the tray icon down on the way out. A tray icon whose
            // owner disappears without removing it stays painted on the taskbar
            // until the user waves the mouse over it ("ghost icon").
            if matches!(event, tauri::RunEvent::Exit) {
                app::drop_tray(app);
            }
        });
}
