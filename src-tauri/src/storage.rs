//! Disk space the app no longer needs, measured and cleaned.
//!
//! Added for 0.9.23. On one machine on 25 November 2026 the app's folders held
//! about 4.5 GB. Most of it was in use (the speech model, the recordings the
//! voice learning reads), but some of it was left over: program files from the
//! Lalia days, safety copies of the history taken before old upgrades, and a
//! second speech model nobody had picked.
//!
//! Two rules decide what counts as left over, and both lean towards keeping:
//! - Only files this app wrote into its own folders. A model shipped inside the
//!   installer or found in a developer checkout is never touched.
//! - Nothing anyone might still want. The model in use stays, and so does one
//!   reserve: the model the graphics card gauge would offer when the card fills
//!   up (the most accurate measured model that is smaller than the one in use).
//!   The recordings stay too: they are what the voice learning reads, they are
//!   already deleted after 30 days, and removing them sooner is a separate,
//!   explicit choice in Settings.
//!
//! What happens on its own at every start (`auto_tidy`) is limited to files
//! nobody can miss: old program files next to the installed program, and
//! pre-upgrade copies of the history older than 30 days (the newest one stays).
//! Models are removed only when the user presses a button.

use serde::Serialize;
use std::path::{Path, PathBuf};

/// Pre-upgrade copies of the history are kept this long, and the newest one
/// is kept for good.
pub const UPGRADE_COPY_DAYS: u64 = 30;

#[derive(Debug, Clone, Serialize, Default)]
pub struct StorageItem {
    /// "kind:name". The confirm box lists items by key, and `clean` deletes
    /// only the keys the user confirmed.
    pub key: String,
    /// "model", "old_program", "upgrade_copy".
    pub kind: String,
    /// What the user sees: a model's display name, or a file name.
    pub name: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct StorageReport {
    /// Everything below that can go, added up.
    pub cleanable_bytes: u64,
    pub items: Vec<StorageItem>,
    /// Kept on purpose, shown so the user knows why it is not counted.
    pub reserve_model: Option<String>,
    /// The kept recordings, which the gauge does not count.
    pub recordings_bytes: u64,
    pub recordings_files: u64,
    /// All of the app's own folders together.
    pub total_bytes: u64,
    /// A model is downloading; models are left alone until it ends.
    pub models_busy: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CleanResult {
    pub freed_bytes: u64,
    pub removed: u64,
    /// Files that could not be deleted (open in another program, permissions).
    pub failed: u64,
}

// ---------------------------------------------------------------- models

/// The model the gauge would offer instead of `current` when the card fills
/// up: the most accurate measured one that needs less video memory. Mirrors
/// the choice in `commands::gpu_gauge`, but without the live free-memory
/// figure, since a reserve is for the day the card is full.
fn reserve_model(current: &str, installed: &[crate::models::ModelSpec]) -> Option<String> {
    let catalog = crate::models::catalog();
    let needs = catalog.iter().find(|m| m.id == current).map(|m| m.vram_mb).unwrap_or(u32::MAX);
    let mut options: Vec<_> = installed
        .iter()
        .filter(|m| m.id != current && m.vram_mb < needs && m.greek_errors_pct.is_some())
        .collect();
    options.sort_by_key(|m| m.greek_errors_pct.unwrap_or(u32::MAX));
    options.first().map(|m| m.id.clone())
}

/// Installed models in the app's own models folder that are neither in use nor
/// the reserve, and never one larger than the model in use: a user moved down
/// to a lighter model (by the light-model offer, or after the graphics card
/// failed) will want the big one back once the card works again.
fn unused_models(models_dir: &Path, current: &str) -> (Vec<(crate::models::ModelSpec, PathBuf, u64)>, Option<String>) {
    let mut installed = vec![];
    for spec in crate::models::catalog() {
        let p = models_dir.join(&spec.file_name);
        if let Ok(m) = std::fs::metadata(&p) {
            if m.is_file() {
                installed.push((spec, p, m.len()));
            }
        }
    }
    let specs: Vec<_> = installed.iter().map(|(s, _, _)| s.clone()).collect();
    let needs = crate::models::catalog().iter().find(|m| m.id == current).map(|m| m.vram_mb).unwrap_or(u32::MAX);
    let reserve = reserve_model(current, &specs);
    let unused = installed
        .into_iter()
        .filter(|(s, _, _)| s.id != current && Some(&s.id) != reserve.as_ref() && s.vram_mb <= needs)
        .collect();
    (unused, reserve)
}

// ---------------------------------------------------------- old program

/// Files next to the installed program that an older version left behind:
/// `lalia.exe` (the program's name before 10 September 2026) and `*.exe.bak`
/// copies. Only inside a real installation (the uninstaller is there) and
/// never the running program itself.
fn old_program_files(install_dir: &Path, running: &Path) -> Vec<(PathBuf, u64)> {
    if !install_dir.join("uninstall.exe").is_file() {
        return vec![];
    }
    let running_name = running.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
    let Ok(entries) = std::fs::read_dir(install_dir) else { return vec![] };
    let mut out = vec![];
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_lowercase();
        if name == running_name {
            continue;
        }
        let old = name == "lalia.exe" || name.ends_with(".exe.bak");
        if !old {
            continue;
        }
        if let Ok(m) = e.metadata() {
            if m.is_file() {
                out.push((e.path(), m.len()));
            }
        }
    }
    out
}

fn install_dir_and_exe() -> Option<(PathBuf, PathBuf)> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?.to_path_buf();
    Some((dir, exe))
}

// --------------------------------------------------------- upgrade copies

/// `<name>.pre-v<N>.bak` copies of the history, taken before an upgrade of
/// its layout (db.rs). The last batch is kept for good: every copy written
/// within an hour of the newest one, since an upgrade over several versions
/// writes pre-v4, pre-v5 and pre-v6 in one start and only the lowest of them
/// is the untouched original. Older copies go once they are older than `days`.
fn old_upgrade_copies(config_dir: &Path, days: u64) -> Vec<(PathBuf, u64)> {
    let Ok(entries) = std::fs::read_dir(config_dir) else { return vec![] };
    let mut copies: Vec<(u32, PathBuf, u64, std::time::SystemTime)> = vec![];
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        let Some(v) = upgrade_copy_version(&name) else { continue };
        let Ok(m) = e.metadata() else { continue };
        if !m.is_file() {
            continue;
        }
        copies.push((v, e.path(), m.len(), m.modified().unwrap_or(std::time::SystemTime::now())));
    }
    let Some(newest) = copies.iter().map(|c| c.3).max() else { return vec![] };
    let batch = std::time::Duration::from_secs(3600);
    let now = std::time::SystemTime::now();
    let age = std::time::Duration::from_secs(days * 86_400);
    copies
        .into_iter()
        .filter(|(_, _, _, t)| newest.duration_since(*t).map(|d| d > batch).unwrap_or(false))
        .filter(|(_, _, _, t)| now.duration_since(*t).map(|d| d > age).unwrap_or(false))
        .map(|(_, p, len, _)| (p, len))
        .collect()
}

/// "fuckyouflow.pre-v3.bak" or "lalia.pre-v3.bak" -> 3.
fn upgrade_copy_version(name: &str) -> Option<u32> {
    let rest = name.strip_prefix("fuckyouflow.pre-v").or_else(|| name.strip_prefix("lalia.pre-v"))?;
    rest.strip_suffix(".bak")?.parse().ok()
}

// ------------------------------------------------------------------ sizes

fn dir_size(dir: &Path) -> (u64, u64) {
    let Ok(entries) = std::fs::read_dir(dir) else { return (0, 0) };
    let (mut bytes, mut files) = (0, 0);
    for e in entries.flatten() {
        match e.file_type() {
            // Links are not followed: a linked models folder belongs elsewhere.
            Ok(t) if t.is_symlink() => {}
            Ok(t) if t.is_dir() => {
                let (b, f) = dir_size(&e.path());
                bytes += b;
                files += f;
            }
            Ok(_) => {
                bytes += e.metadata().map(|m| m.len()).unwrap_or(0);
                files += 1;
            }
            Err(_) => {}
        }
    }
    (bytes, files)
}

// ----------------------------------------------------------------- public

/// What can be cleaned right now, for the gauge, Settings and the "What's
/// new" card.
pub fn report(current_model: &str) -> StorageReport {
    let mut r = StorageReport { models_busy: crate::models::downloading_any(), ..Default::default() };
    let (unused, reserve) = unused_models(&crate::paths::models_dir(), current_model);
    r.reserve_model = reserve.and_then(|id| crate::models::catalog().into_iter().find(|m| m.id == id)).map(|m| m.display_name);
    if !r.models_busy {
        for (spec, _, len) in unused {
            r.items.push(item("model", spec.display_name, len));
        }
    }
    if let Some((dir, exe)) = install_dir_and_exe() {
        for (p, len) in old_program_files(&dir, &exe) {
            r.items.push(item("old_program", file_name(&p), len));
        }
    }
    for (p, len) in old_upgrade_copies(&crate::paths::config_dir(), UPGRADE_COPY_DAYS) {
        r.items.push(item("upgrade_copy", file_name(&p), len));
    }
    r.cleanable_bytes = r.items.iter().map(|i| i.bytes).sum();
    let (rb, rf) = dir_size(&crate::paths::audio_dir());
    r.recordings_bytes = rb;
    r.recordings_files = rf;
    let mut roots = vec![crate::paths::config_dir(), crate::paths::local_dir()];
    roots.dedup();
    r.total_bytes = roots.iter().map(|d| dir_size(d).0).sum();
    r
}

/// Deletes what the user confirmed, and only what is still left over at this
/// moment: the list is worked out again from the settings in force and kept
/// to the keys in `confirmed` (the ones the confirm box named). A model picked
/// a second ago, or one the box never showed, stays.
pub fn clean(current_model: &str, confirmed: &[String]) -> CleanResult {
    let ok = |key: String| confirmed.iter().any(|c| *c == key);
    let mut out = CleanResult::default();
    let mut remove = |p: &Path, len: u64| match std::fs::remove_file(p) {
        Ok(()) => {
            out.freed_bytes += len;
            out.removed += 1;
        }
        Err(err) => {
            tracing::warn!("could not delete {}: {err}", p.display());
            out.failed += 1;
        }
    };
    if !crate::models::downloading_any() {
        for (spec, p, len) in unused_models(&crate::paths::models_dir(), current_model).0 {
            // Checked again per file: a download may have started meanwhile.
            if !ok(format!("model:{}", spec.display_name)) || crate::models::downloading(&p) {
                continue;
            }
            remove(&p, len);
            // The checksum note beside the model goes with it.
            let _ = std::fs::remove_file(p.with_extension("sha256"));
        }
    }
    if let Some((dir, exe)) = install_dir_and_exe() {
        for (p, len) in old_program_files(&dir, &exe) {
            if ok(format!("old_program:{}", file_name(&p))) {
                remove(&p, len);
            }
        }
    }
    for (p, len) in old_upgrade_copies(&crate::paths::config_dir(), UPGRADE_COPY_DAYS) {
        if ok(format!("upgrade_copy:{}", file_name(&p))) {
            remove(&p, len);
        }
    }
    crate::journal::info("storage.cleaned", serde_json::json!({ "freed": out.freed_bytes, "files": out.removed, "failed": out.failed }));
    out
}

/// Runs at every start: removes only what nobody can miss (old program files
/// and old pre-upgrade copies). Models wait for the user's button.
pub fn auto_tidy() {
    let mut freed = 0u64;
    let mut files = 0u64;
    let mut gone = |p: &Path, len: u64| {
        if std::fs::remove_file(p).is_ok() {
            freed += len;
            files += 1;
        }
    };
    if let Some((dir, exe)) = install_dir_and_exe() {
        for (p, len) in old_program_files(&dir, &exe) {
            gone(&p, len);
        }
    }
    for (p, len) in old_upgrade_copies(&crate::paths::config_dir(), UPGRADE_COPY_DAYS) {
        gone(&p, len);
    }
    if files > 0 {
        tracing::info!("tidied {files} leftover files ({freed} bytes)");
        crate::journal::info("storage.tidied", serde_json::json!({ "freed": freed, "files": files }));
    }
}

/// Deletes the kept recordings. The history text stays; only the sound behind
/// it goes, the same thing the 30-day rule does one day at a time. The last
/// ten minutes stay: a dictation that just failed may still be retried from
/// its recording.
pub fn delete_recordings() -> CleanResult {
    let mut out = CleanResult::default();
    let fresh = std::time::Duration::from_secs(10 * 60);
    let now = std::time::SystemTime::now();
    let Ok(entries) = std::fs::read_dir(crate::paths::audio_dir()) else { return out };
    for e in entries.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("wav") {
            continue;
        }
        let Ok(meta) = e.metadata() else { continue };
        let young = meta.modified().ok().and_then(|t| now.duration_since(t).ok()).is_none_or(|d| d < fresh);
        if young {
            continue;
        }
        let len = meta.len();
        match std::fs::remove_file(&p) {
            Ok(()) => {
                out.freed_bytes += len;
                out.removed += 1;
            }
            Err(_) => out.failed += 1,
        }
    }
    crate::journal::info("storage.recordings_deleted", serde_json::json!({ "freed": out.freed_bytes, "files": out.removed }));
    out
}

fn item(kind: &str, name: String, bytes: u64) -> StorageItem {
    StorageItem { key: format!("{kind}:{name}"), kind: kind.into(), name, bytes }
}

fn file_name(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("fyf-storage-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn spec(id: &str) -> crate::models::ModelSpec {
        crate::models::catalog().into_iter().find(|m| m.id == id).unwrap()
    }

    #[test]
    fn the_reserve_is_the_most_accurate_smaller_measured_model() {
        let installed = vec![spec("large-v3-q5_0"), spec("large-v3-turbo-q5_0"), spec("medium-q5_0")];
        // Turbo q5 has fewer measured Greek errors than medium among the smaller ones.
        let want = {
            let mut o: Vec<_> = installed.iter().filter(|m| m.id != "large-v3-q5_0" && m.greek_errors_pct.is_some()).collect();
            o.sort_by_key(|m| m.greek_errors_pct.unwrap());
            o[0].id.clone()
        };
        assert_eq!(reserve_model("large-v3-q5_0", &installed), Some(want));
        // Nothing smaller and measured: no reserve.
        assert_eq!(reserve_model("large-v3-q5_0", &[spec("large-v3-q5_0")]), None);
    }

    #[test]
    fn the_model_in_use_and_the_reserve_are_never_unused() {
        let d = tmp("models");
        for id in ["large-v3-q5_0", "large-v3-turbo-q5_0", "medium-q5_0"] {
            std::fs::write(d.join(spec(id).file_name), b"x").unwrap();
        }
        let (unused, reserve) = unused_models(&d, "large-v3-q5_0");
        let ids: Vec<_> = unused.iter().map(|(s, _, _)| s.id.clone()).collect();
        assert!(!ids.contains(&"large-v3-q5_0".to_string()));
        assert!(reserve.is_some());
        assert!(!ids.contains(reserve.as_ref().unwrap()));
        assert_eq!(ids.len(), 1);
        // Only the in-use model on disk: nothing is unused.
        let only = tmp("models-one");
        std::fs::write(only.join(spec("large-v3-q5_0").file_name), b"x").unwrap();
        assert!(unused_models(&only, "large-v3-q5_0").0.is_empty());
        let _ = std::fs::remove_dir_all(d);
        let _ = std::fs::remove_dir_all(only);
    }

    #[test]
    fn old_program_files_only_inside_an_installation_and_never_the_running_one() {
        let d = tmp("install");
        for f in ["fuckyouflow.exe", "lalia.exe", "lalia-0.9.2.exe.bak", "fuckyouflow-0.9.9-public.exe.bak", "notes.txt"] {
            std::fs::write(d.join(f), b"xy").unwrap();
        }
        // No uninstaller: not an installation, nothing is touched.
        assert!(old_program_files(&d, &d.join("fuckyouflow.exe")).is_empty());
        std::fs::write(d.join("uninstall.exe"), b"u").unwrap();
        let mut got: Vec<_> = old_program_files(&d, &d.join("fuckyouflow.exe")).iter().map(|(p, _)| file_name(p)).collect();
        got.sort();
        assert_eq!(got, ["fuckyouflow-0.9.9-public.exe.bak", "lalia-0.9.2.exe.bak", "lalia.exe"]);
        // Running as lalia.exe (a very old install): that file stays.
        let got: Vec<_> = old_program_files(&d, &d.join("lalia.exe")).iter().map(|(p, _)| file_name(p)).collect();
        assert!(!got.contains(&"lalia.exe".to_string()));
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn a_model_larger_than_the_one_in_use_is_never_unused() {
        // Moved down to medium (say, after the graphics card failed): the big
        // model waits for the card to work again, turbo is the reserve.
        let d = tmp("models-down");
        for id in ["large-v3-q5_0", "large-v3-turbo-q5_0", "medium-q5_0"] {
            std::fs::write(d.join(spec(id).file_name), b"x").unwrap();
        }
        assert!(spec("large-v3-q5_0").vram_mb > spec("medium-q5_0").vram_mb);
        let (unused, _) = unused_models(&d, "medium-q5_0");
        assert!(unused.iter().all(|(s, _, _)| s.id != "large-v3-q5_0"));
        let _ = std::fs::remove_dir_all(d);
    }

    fn age(p: &Path, by: std::time::Duration) {
        let f = std::fs::File::options().write(true).open(p).unwrap();
        f.set_modified(std::time::SystemTime::now() - by).unwrap();
    }

    #[test]
    fn the_last_batch_of_upgrade_copies_stays_and_young_ones_stay() {
        assert_eq!(upgrade_copy_version("fuckyouflow.pre-v3.bak"), Some(3));
        assert_eq!(upgrade_copy_version("lalia.pre-v12.bak"), Some(12));
        assert_eq!(upgrade_copy_version("settings.json.bak"), None);
        assert_eq!(upgrade_copy_version("fuckyouflow.db.before-rules-change-20261120-144251"), None);
        let d = tmp("copies");
        let day = std::time::Duration::from_secs(86_400);
        // pre-v2 from an upgrade 40 days ago; pre-v3, pre-v4 and pre-v5 written
        // together in one start 35 days ago (an upgrade over several versions).
        std::fs::write(d.join("fuckyouflow.pre-v2.bak"), b"db").unwrap();
        age(&d.join("fuckyouflow.pre-v2.bak"), day * 40);
        for f in ["fuckyouflow.pre-v3.bak", "fuckyouflow.pre-v4.bak", "fuckyouflow.pre-v5.bak"] {
            std::fs::write(d.join(f), b"db").unwrap();
            age(&d.join(f), day * 35);
        }
        let got: Vec<_> = old_upgrade_copies(&d, 30).iter().map(|(p, _)| file_name(p)).collect();
        assert_eq!(got, ["fuckyouflow.pre-v2.bak"]);
        // Within 30 days nothing goes, whatever the batch.
        assert!(old_upgrade_copies(&d, 45).is_empty());
        let _ = std::fs::remove_dir_all(d);
    }
}
