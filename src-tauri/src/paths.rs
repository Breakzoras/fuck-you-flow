//! Where the app keeps its files on disk.
//!
//! Roaming (%APPDATA%\FuckYouFlow): settings.json, fuckyouflow.db (small, user data).
//! Local (%LOCALAPPDATA%\FuckYouFlow): models, whisper runtime, logs, temp audio (large, machine bound).
//!
//! The folders were called Lalia until 10 September 2026. The old name is not
//! the product's name any more and had no business being on anybody's disk, so
//! the folders are renamed on the first start after the update. A rename inside
//! the same drive moves nothing, whatever the folder holds, so the 1.6 GB of
//! models cost nothing here. If the rename cannot happen, because a file is
//! open or the disk says no, the old folder keeps being used and the app runs
//! exactly as before rather than starting empty.

use std::path::PathBuf;
use std::sync::OnceLock;

/// One word, no spaces, on purpose.
///
/// The program itself installs into `%LOCALAPPDATA%\Fuck You Flow`. A data
/// folder of the same name would land inside the installation, mixing a user's
/// history and models with the files an uninstall deletes, so the two names
/// must differ.
pub const APP_DIR_NAME: &str = "FuckYouFlow";

/// Folder names used before, newest first. Each one is adopted in turn, so a
/// user who skipped a version still finds their own files.
const OLD_APP_DIR_NAMES: [&str; 2] = ["Fuck You Flow", "Lalia"];

/// True when this folder is the installed program rather than somebody's data.
fn is_installation(dir: &std::path::Path) -> bool {
    dir.join("uninstall.exe").exists() || dir.join("fuckyouflow.exe").exists() || dir.join("lalia.exe").exists()
}

/// The new path, unless an older one is still there and cannot be moved.
fn adopt(base: PathBuf) -> PathBuf {
    let now = base.join(APP_DIR_NAME);
    if now.exists() {
        return now;
    }
    for old in OLD_APP_DIR_NAMES {
        let before = base.join(old);
        if !before.exists() || is_installation(&before) {
            continue;
        }
        match std::fs::rename(&before, &now) {
            Ok(()) => {
                tracing::info!("moved {} to {}", before.display(), now.display());
                return now;
            }
            Err(e) => {
                tracing::warn!("could not rename {} to {}: {e}", before.display(), now.display());
                return before;
            }
        }
    }
    now
}

pub fn config_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| adopt(dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")))).clone()
}

pub fn local_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| adopt(dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")))).clone()
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
    static FILE: OnceLock<PathBuf> = OnceLock::new();
    FILE.get_or_init(|| {
        let now = config_dir().join("fuckyouflow.db");
        let before = config_dir().join("lalia.db");
        if now.exists() || !before.exists() {
            return now;
        }
        // The history of everything ever dictated lives in this one file, so a
        // failed rename keeps the old file rather than quietly starting a new
        // and empty one.
        match std::fs::rename(&before, &now) {
            Ok(()) => {
                for suffix in ["-journal", "-wal", "-shm"] {
                    let a = config_dir().join(format!("lalia.db{suffix}"));
                    if a.exists() {
                        let _ = std::fs::rename(&a, config_dir().join(format!("fuckyouflow.db{suffix}")));
                    }
                }
                now
            }
            Err(e) => {
                tracing::warn!("could not rename the history file: {e}");
                before
            }
        }
    })
    .clone()
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
