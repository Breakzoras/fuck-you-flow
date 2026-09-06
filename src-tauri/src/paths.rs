//! Where Lalia keeps its files on disk.
//!
//! Roaming (%APPDATA%\Lalia): settings.json, lalia.db (small, user data).
//! Local (%LOCALAPPDATA%\Lalia): models, whisper runtime, logs, temp audio (large, machine bound).

use std::path::PathBuf;

pub const APP_DIR_NAME: &str = "Lalia";

pub fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join(APP_DIR_NAME)
}

pub fn local_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join(APP_DIR_NAME)
}

/// Where the installer put the engine and the models, set once at startup.
/// Empty in a dev run, where the checkout's own folders are used instead.
static BUNDLED: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

pub fn set_bundled_dir(dir: PathBuf) {
    let _ = BUNDLED.set(dir);
}

/// Engine and models shipped inside the installer (`<install>/resources/bundled`).
pub fn bundled_dir() -> Option<&'static PathBuf> {
    BUNDLED.get().filter(|p| p.exists())
}

pub fn settings_file() -> PathBuf {
    config_dir().join("settings.json")
}

pub fn db_file() -> PathBuf {
    config_dir().join("lalia.db")
}

pub fn models_dir() -> PathBuf {
    local_dir().join("models")
}

pub fn runtime_dir() -> PathBuf {
    local_dir().join("runtime")
}

pub fn whisper_runtime_dir() -> PathBuf {
    runtime_dir().join("whisper")
}

pub fn logs_dir() -> PathBuf {
    local_dir().join("logs")
}

pub fn temp_dir() -> PathBuf {
    local_dir().join("temp")
}

pub fn recovery_dir() -> PathBuf {
    local_dir().join("recovery")
}

pub fn ensure_all() -> std::io::Result<()> {
    for d in [
        config_dir(),
        local_dir(),
        models_dir(),
        runtime_dir(),
        logs_dir(),
        temp_dir(),
        recovery_dir(),
    ] {
        std::fs::create_dir_all(&d)?;
    }
    Ok(())
}
