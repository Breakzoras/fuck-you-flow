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

/// True when this folder is somebody's own work rather than an empty shell left
/// behind by a half finished update.
///
/// Folders are looked inside rather than merely counted. A run that was cut
/// short leaves a tree of empty folders under the new name, and taking that as
/// data left 1.6 GB of models sitting under the old name while the app
/// reported the engine missing. Measured 10 September 2026.
fn holds_user_files(dir: &std::path::Path) -> bool {
    for f in ["settings.json", "fuckyouflow.db", "lalia.db"] {
        if dir.join(f).is_file() {
            return true;
        }
    }
    ["models", "runtime"]
        .iter()
        .any(|n| std::fs::read_dir(dir.join(n)).map(|mut d| d.next().is_some()).unwrap_or(false))
}

/// True when any file sits anywhere under this folder.
fn tree_has_files(dir: &std::path::Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else { return false };
    entries.flatten().any(|e| match e.file_type() {
        Ok(t) if t.is_dir() => tree_has_files(&e.path()),
        Ok(_) => true,
        Err(_) => true,
    })
}

fn unix_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Old settings folders that still hold a history of their own after the move.
///
/// An older build that runs after the update cannot see the new folder, so it
/// starts an empty one under its old name and writes there. That happened on
/// 10 September 2026 when a taskbar pin still pointed at an old build: 24
/// dictations went into a history nobody could see. Each folder found is
/// renamed aside first (which fails while that old build still has it open,
/// and then it is left for the next start) and its history file is returned
/// for merging.
pub fn take_orphan_histories() -> Vec<PathBuf> {
    let Some(base) = dirs::config_dir() else { return vec![] };
    let current = config_dir();
    let mut found = vec![];
    for name in OLD_APP_DIR_NAMES {
        let old = base.join(name);
        if old == current || !old.is_dir() || is_installation(&old) {
            continue;
        }
        let Some(file) = ["fuckyouflow.db", "lalia.db"].into_iter().find(|f| old.join(f).is_file()) else {
            continue;
        };
        let aside = base.join(format!("{}-merged-{}", name.replace(' ', ""), unix_secs()));
        match std::fs::rename(&old, &aside) {
            Ok(()) => found.push(aside.join(file)),
            Err(e) => tracing::warn!("a second history sits in {} and cannot be moved yet: {e}", old.display()),
        }
    }
    found
}

/// Folders this app moved out of the way: an old profile once its history was
/// merged, and a new-name folder that stood in the way of the real one. Both
/// can hold a full copy of the history, so "delete all data" takes them too
/// (pre-release review, 11 September 2026). Only names of exactly those two
/// shapes, ending in the seconds stamp, are ever returned.
pub fn set_aside_folders() -> Vec<PathBuf> {
    let mut prefixes = vec![format!("{APP_DIR_NAME}-aside-")];
    prefixes.extend(OLD_APP_DIR_NAMES.iter().map(|n| format!("{}-merged-", n.replace(' ', ""))));
    let mut bases: Vec<PathBuf> = [config_dir(), local_dir()].iter().filter_map(|d| d.parent().map(PathBuf::from)).collect();
    bases.dedup();
    let mut out = vec![];
    for base in bases {
        let Ok(entries) = std::fs::read_dir(&base) else { continue };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let ours = prefixes.iter().any(|p| {
                name.strip_prefix(p.as_str()).is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()))
            });
            if ours && entry.path().is_dir() {
                out.push(entry.path());
            }
        }
    }
    out
}

#[cfg(test)]
mod set_aside_tests {
    use super::*;

    #[test]
    fn only_the_folders_this_app_set_aside_are_listed() {
        let base = config_dir().parent().unwrap().to_path_buf();
        for d in ["FuckYouFlow-aside-1757590000", "Lalia-merged-1757590001", "FuckYouFlow-merged-1757590002",
                  "Lalia-merged-copy", "Lalia-orphan-2026-09-11", "SomeoneElse-aside-1757590003"] {
            std::fs::create_dir_all(base.join(d)).unwrap();
        }
        let mut got: Vec<String> = set_aside_folders().iter()
            .filter(|p| p.parent() == Some(base.as_path()))
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned()).collect();
        got.sort();
        assert_eq!(got, ["FuckYouFlow-aside-1757590000", "FuckYouFlow-merged-1757590002", "Lalia-merged-1757590001"]);
    }
}

/// The folder to use, moving an older one into place when there is one.
///
/// Every old name is looked at, and the one that actually holds files wins.
/// Picking the first name that merely exists was not enough: an interrupted
/// update can leave an empty folder under a newer name, and adopting that one
/// would open the app with an empty history while the real one sat untouched
/// beside it.
fn adopt(base: PathBuf) -> PathBuf {
    let now = base.join(APP_DIR_NAME);
    if now.exists() && holds_user_files(&now) {
        return now;
    }
    let candidates: Vec<PathBuf> = OLD_APP_DIR_NAMES
        .iter()
        .map(|n| base.join(n))
        .filter(|p| p.exists() && !is_installation(p))
        .collect();
    let Some(before) = candidates.iter().find(|p| holds_user_files(p)).or_else(|| candidates.first()) else {
        return now;
    };
    if now.exists() {
        // Something under the new name is in the way of the real folder. It
        // holds no settings, history or models, but it can hold logs, a
        // recovery recording or kept audio. A tree of empty folders goes; any
        // file at all means the folder is moved aside, never deleted (audit and
        // Greptile, 11 September 2026).
        if !tree_has_files(&now) {
            let _ = std::fs::remove_dir_all(&now);
        } else {
            let aside = base.join(format!("{APP_DIR_NAME}-aside-{}", unix_secs()));
            if let Err(e) = std::fs::rename(&now, &aside) {
                tracing::warn!("could not move {} aside: {e}; staying with {}", now.display(), before.display());
                return before.clone();
            }
            tracing::info!("moved {} aside to {}", now.display(), aside.display());
        }
    }
    match std::fs::rename(before, &now) {
        Ok(()) => {
            tracing::info!("moved {} to {}", before.display(), now.display());
            now
        }
        Err(e) => {
            tracing::warn!("could not rename {} to {}: {e}", before.display(), now.display());
            before.clone()
        }
    }
}

#[cfg(test)]
mod adopt_tests {
    use super::*;

    fn seed(base: &std::path::Path, name: &str, file: Option<&str>) {
        let d = base.join(name);
        std::fs::create_dir_all(&d).unwrap();
        if let Some(f) = file {
            std::fs::write(d.join(f), b"x").unwrap();
        }
    }

    /// The folder holding the user's own files wins, whatever it is called and
    /// whatever empty folders sit beside it.
    #[test]
    fn the_folder_with_the_files_in_it_is_the_one_that_moves() {
        let base = std::env::temp_dir().join(format!("fyf-adopt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        // An empty tree under the new name, and the real history under the
        // oldest name. This is exactly what an interrupted update leaves.
        seed(&base, APP_DIR_NAME, None);
        std::fs::create_dir_all(base.join(APP_DIR_NAME).join("models")).unwrap();
        std::fs::create_dir_all(base.join(APP_DIR_NAME).join("runtime")).unwrap();
        seed(&base, "Lalia", Some("lalia.db"));
        let got = adopt(base.clone());
        assert_eq!(got, base.join(APP_DIR_NAME));
        assert!(got.join("lalia.db").exists(), "the history must travel with the folder");
        assert!(!base.join("Lalia").exists(), "the old folder is gone once it has moved");

        let _ = std::fs::remove_dir_all(&base);
    }

    /// A new-name folder that holds only logs or a recovery recording is in the
    /// way of the real one. It moves aside with its files; nothing is deleted.
    #[test]
    fn a_folder_in_the_way_is_moved_aside_never_deleted() {
        let base = std::env::temp_dir().join(format!("fyf-adopt-aside-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        std::fs::create_dir_all(base.join(APP_DIR_NAME).join("logs")).unwrap();
        std::fs::write(base.join(APP_DIR_NAME).join("logs").join("fuckyouflow.log.2026-09-11"), b"x").unwrap();
        seed(&base, "Lalia", Some("lalia.db"));
        let got = adopt(base.clone());
        assert_eq!(got, base.join(APP_DIR_NAME));
        assert!(got.join("lalia.db").exists(), "the history moved into place");
        let aside: Vec<_> = std::fs::read_dir(&base).unwrap().flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with("FuckYouFlow-aside-")).collect();
        assert_eq!(aside.len(), 1, "the folder that was in the way still exists");
        assert!(aside[0].path().join("logs").join("fuckyouflow.log.2026-09-11").exists(), "its log survived");

        let _ = std::fs::remove_dir_all(&base);
    }

    /// The installed program is never mistaken for a data folder, which is what
    /// would happen on Windows where the program lives one name away.
    #[test]
    fn the_installation_folder_is_never_adopted() {
        let base = std::env::temp_dir().join(format!("fyf-adopt-install-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        seed(&base, "Fuck You Flow", Some("uninstall.exe"));
        let got = adopt(base.clone());
        assert_eq!(got, base.join(APP_DIR_NAME));
        assert!(base.join("Fuck You Flow").join("uninstall.exe").exists(), "the installation is left alone");

        let _ = std::fs::remove_dir_all(&base);
    }
}

/// Tests get folders of their own. Before 11 September 2026 they wrote their
/// events into the real journal of whoever ran `cargo test`, 14 runs on the
/// owner's machine, and a test prune could have deleted real journal files.
#[cfg(test)]
fn test_base(kind: &str) -> PathBuf {
    std::env::temp_dir().join(format!("fyf-test-{kind}-{}", std::process::id()))
}

pub fn config_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    #[cfg(test)]
    return DIR.get_or_init(|| test_base("config").join(APP_DIR_NAME)).clone();
    #[cfg(not(test))]
    DIR.get_or_init(|| adopt(dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")))).clone()
}

pub fn local_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    #[cfg(test)]
    return DIR.get_or_init(|| test_base("local").join(APP_DIR_NAME)).clone();
    #[cfg(not(test))]
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

/// Where recordings are kept when "keep audio" is on.
pub fn audio_dir() -> PathBuf {
    local_dir().join("audio")
}

/// Deletes a recording this app kept, and nothing else.
///
/// A history row's `audio_path` also holds the user's own file when a sound
/// file was transcribed. That file belongs to the user: deleting the row, all
/// history, or old history must never delete it. Found in the audit of
/// 11 September 2026, before anyone lost a recording to it.
pub fn remove_kept_audio(path: &str) {
    if !remove_if_inside(std::path::Path::new(path), &audio_dir()) {
        tracing::info!("history row pointed at a file outside the recordings folder; the file was left in place");
    }
}

fn remove_if_inside(path: &std::path::Path, dir: &std::path::Path) -> bool {
    match (path.canonicalize(), dir.canonicalize()) {
        (Ok(f), Ok(d)) if f.starts_with(&d) => std::fs::remove_file(f).is_ok(),
        _ => false,
    }
}

#[cfg(test)]
mod kept_audio_tests {
    use super::*;

    #[test]
    fn only_files_inside_the_recordings_folder_are_deleted() {
        let base = std::env::temp_dir().join(format!("fyf-kept-audio-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let audio = base.join("audio");
        std::fs::create_dir_all(&audio).unwrap();
        let ours = audio.join("a.wav");
        let theirs = base.join("meeting.m4a");
        std::fs::write(&ours, b"x").unwrap();
        std::fs::write(&theirs, b"x").unwrap();

        assert!(remove_if_inside(&ours, &audio));
        assert!(!ours.exists());
        assert!(!remove_if_inside(&theirs, &audio), "the user's own file is not ours to delete");
        assert!(theirs.exists());
        // A path that climbs out of the folder is still outside it.
        let sneaky = audio.join("..").join("meeting.m4a");
        assert!(!remove_if_inside(&sneaky, &audio));
        assert!(theirs.exists());

        let _ = std::fs::remove_dir_all(&base);
    }
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
