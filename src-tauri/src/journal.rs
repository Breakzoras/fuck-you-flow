//! The event journal: one JSON line per thing that happened, next to the logs.
//!
//! The rolling text log is written for a human reading a stack of lines. This
//! file is written for a machine (or for me, reading it from outside the app)
//! and answers one question quickly: what happened, in what order, and what
//! went wrong. It never holds transcript text or audio, only kinds, counts,
//! durations and window classes.
//!
//! `%LOCALAPPDATA%\Lalia\logs\events-YYYY-MM-DD.jsonl`, seven days kept.

use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use serde_json::{json, Value};

/// Verbose mode: records the low-level events too (every hotkey edge, every
/// segment). Off by default; the Diagnostics page turns it on.
static VERBOSE: AtomicBool = AtomicBool::new(false);
/// Counts every problem recorded since start, so the UI can show a number
/// without reading the file.
static PROBLEMS: AtomicU64 = AtomicU64::new(0);
static FILE: Mutex<Option<(String, std::fs::File)>> = Mutex::new(None);

pub fn set_verbose(on: bool) {
    VERBOSE.store(on, Ordering::Relaxed);
}

pub fn verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

pub fn problem_count() -> u64 {
    PROBLEMS.load(Ordering::Relaxed)
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn stamp() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string()
}

/// Records one event. `level` is "info", "warn" or "error"; anything but "info"
/// also counts as a problem and is what the Diagnostics page shows first.
pub fn record(level: &str, kind: &str, fields: Value) {
    if level != "info" {
        PROBLEMS.fetch_add(1, Ordering::Relaxed);
    }
    let line = json!({ "at": stamp(), "level": level, "kind": kind, "d": fields });
    if let Err(e) = append(&line.to_string()) {
        // The journal must never take the app down or spam the text log.
        static WARNED: AtomicBool = AtomicBool::new(false);
        if !WARNED.swap(true, Ordering::Relaxed) {
            tracing::warn!("event journal unavailable: {e}");
        }
    }
}

/// Records an event only when verbose mode is on. Used for the high-frequency
/// things: key edges, segment boundaries, clipboard reads.
pub fn trace(kind: &str, fields: Value) {
    if verbose() {
        record("info", kind, fields);
    }
}

pub fn info(kind: &str, fields: Value) {
    record("info", kind, fields);
}

pub fn warn(kind: &str, fields: Value) {
    record("warn", kind, fields);
}

pub fn error(kind: &str, fields: Value) {
    record("error", kind, fields);
}

fn append(line: &str) -> std::io::Result<()> {
    let day = today();
    let mut guard = FILE.lock().unwrap_or_else(|p| p.into_inner());
    let need_open = match guard.as_ref() {
        Some((d, _)) => d != &day,
        None => true,
    };
    if need_open {
        let dir = crate::paths::logs_dir();
        std::fs::create_dir_all(&dir)?;
        let f = std::fs::OpenOptions::new().create(true).append(true).open(dir.join(format!("events-{day}.jsonl")))?;
        *guard = Some((day, f));
        drop(guard);
        prune();
        guard = FILE.lock().unwrap_or_else(|p| p.into_inner());
    }
    if let Some((_, f)) = guard.as_mut() {
        writeln!(f, "{line}")?;
    }
    Ok(())
}

/// Keeps a week. Called once whenever a new day's file is opened.
fn prune() {
    let dir = crate::paths::logs_dir();
    let Ok(rd) = std::fs::read_dir(&dir) else { return };
    let mut files: Vec<std::path::PathBuf> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("events-") && n.ends_with(".jsonl")))
        .collect();
    if files.len() <= 7 {
        return;
    }
    files.sort();
    let cut = files.len() - 7;
    for p in files.into_iter().take(cut) {
        let _ = std::fs::remove_file(p);
    }
}

/// The last `limit` events, newest first, as raw JSON lines. Reads today's file
/// and, if it is short, yesterday's too.
pub fn tail(limit: usize) -> Vec<String> {
    let dir = crate::paths::logs_dir();
    let Ok(rd) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut files: Vec<std::path::PathBuf> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("events-") && n.ends_with(".jsonl")))
        .collect();
    files.sort();
    let mut out: Vec<String> = Vec::new();
    for path in files.iter().rev().take(2) {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let mut lines: Vec<String> = text.lines().rev().map(|s| s.to_string()).collect();
        out.append(&mut lines);
        if out.len() >= limit {
            break;
        }
    }
    out.truncate(limit);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The journal is only useful if what went in comes back out, newest first,
    /// and if a warning is counted as a problem. Everything the Diagnostics
    /// panel shows rests on those two.
    #[test]
    fn writes_and_reads_back_newest_first() {
        let before = problem_count();
        let tag = format!("test.roundtrip.{}", std::process::id());
        info(&tag, json!({ "n": 1 }));
        warn(&tag, json!({ "n": 2 }));
        assert_eq!(problem_count(), before + 1, "a warn counts once as a problem, an info never");

        let mine: Vec<Value> = tail(200)
            .iter()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .filter(|v| v["kind"] == tag.as_str())
            .collect();
        assert!(mine.len() >= 2, "both lines must come back, got {}", mine.len());
        assert_eq!(mine[0]["d"]["n"], 2, "newest first");
        assert_eq!(mine[0]["level"], "warn");
        assert_eq!(mine[1]["d"]["n"], 1);
        assert!(mine[0]["at"].as_str().is_some_and(|s| s.len() > 10), "every line carries a timestamp");
    }

    /// `trace` is the high-frequency path: silent unless debug mode asked for it.
    #[test]
    fn trace_is_silent_until_verbose() {
        let tag = format!("test.verbose.{}", std::process::id());
        set_verbose(false);
        trace(&tag, json!({ "seen": "no" }));
        let count = |t: &str| tail(200).iter().filter(|l| l.contains(t)).count();
        assert_eq!(count(&tag), 0, "nothing may be written while verbose is off");
        set_verbose(true);
        trace(&tag, json!({ "seen": "yes" }));
        assert_eq!(count(&tag), 1, "verbose mode records it once");
        set_verbose(false);
    }
}
