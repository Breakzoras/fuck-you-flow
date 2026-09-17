//! Structured local logs. Transcript text and audio are never logged unless the
//! user disables redaction in Privacy settings (and even then only text, never audio).
//!
//! `%LOCALAPPDATA%\FuckYouFlow\logs\fuckyouflow.log.YYYY-MM-DD`, seven days kept. Both the
//! file name and the stamps inside follow the user's own clock, so a line here
//! and a line in `journal.rs` describing the same moment carry the same time.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use tracing_subscriber::fmt::format::Writer as FmtWriter;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static REDACT: AtomicBool = AtomicBool::new(true);
static GUARD: once_cell::sync::OnceCell<tracing_appender::non_blocking::WorkerGuard> =
    once_cell::sync::OnceCell::new();

const PREFIX: &str = "fuckyouflow.log.";
/// What the files were called until 10 September 2026. Kept only so the old
/// ones still get swept up by the weekly prune instead of sitting on the
/// disk forever after the rename.
const OLD_PREFIX: &str = "lalia.log.";
/// Daily files to keep, matching the event journal's week.
const KEEP_DAYS: usize = 7;

/// Wall-clock time as the user reads it, offset included. The default timer
/// writes UTC, which put these lines three hours behind the event journal and
/// made the two stores look like they described different sessions.
struct LocalTime;

impl FormatTime for LocalTime {
    fn format_time(&self, w: &mut FmtWriter<'_>) -> std::fmt::Result {
        write!(w, "{}", chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.6f%:z"))
    }
}

/// A log file that rolls at local midnight. `tracing_appender`'s daily roller
/// rolls on UTC midnight, so for a user three hours ahead each file held 03:00
/// to 03:00 of the day it was named after.
struct LocalDaily {
    dir: PathBuf,
    open: Option<(String, std::fs::File)>,
}

impl LocalDaily {
    fn new(dir: PathBuf) -> Self {
        Self { dir, open: None }
    }

    fn today() -> String {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    }

    fn file_for_today(&mut self) -> std::io::Result<&mut std::fs::File> {
        let day = Self::today();
        let stale = match &self.open {
            Some((d, _)) => d != &day,
            None => true,
        };
        if stale {
            std::fs::create_dir_all(&self.dir)?;
            let f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.dir.join(format!("{PREFIX}{day}")))?;
            self.open = Some((day, f));
            prune(&self.dir);
        }
        Ok(&mut self.open.as_mut().expect("opened just above").1)
    }
}

impl Write for LocalDaily {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file_for_today()?.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.open.as_mut() {
            Some((_, f)) => f.flush(),
            None => Ok(()),
        }
    }
}

/// The dated log files in `dir`, oldest first, ordered by the date in the
/// name. Sorting whole names put every
/// `fuckyouflow.log.*` before every `lalia.log.*` ('f' before 'l'), so on a
/// machine that ran the old builds the prune deleted the newest logs first and
/// kept the old ones forever. On the same day the old name counts as older.
/// Each entry says whether it carries the current name.
fn dated_logs(dir: &Path) -> Vec<(PathBuf, bool)> {
    let Ok(rd) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut files: Vec<(String, bool, PathBuf)> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter_map(|p| {
            let (day, current) = {
                let name = p.file_name()?.to_str()?;
                match name.strip_prefix(PREFIX) {
                    Some(day) => (day.to_string(), true),
                    None => (name.strip_prefix(OLD_PREFIX)?.to_string(), false),
                }
            };
            Some((day, current, p))
        })
        .collect();
    files.sort();
    files.into_iter().map(|(_, current, p)| (p, current)).collect()
}

/// The newest log this program writes, for the Diagnostics page. Files left by
/// the old "Lalia" builds are skipped: their lines describe a program that is
/// no longer running.
pub fn newest_log(dir: &Path) -> Option<PathBuf> {
    dated_logs(dir).into_iter().filter(|(_, current)| *current).map(|(p, _)| p).last()
}

/// Keeps a week. Called whenever a new day's file is opened. Without this the
/// text log grew for the life of the install; only the journal was pruned.
fn prune(dir: &Path) {
    let files = dated_logs(dir);
    if files.len() <= KEEP_DAYS {
        return;
    }
    let cut = files.len() - KEEP_DAYS;
    for (p, _) in files.into_iter().take(cut) {
        let _ = std::fs::remove_file(p);
    }
}

pub fn init() {
    let dir = crate::paths::logs_dir();
    let _ = std::fs::create_dir_all(&dir);
    let (non_blocking, guard) = tracing_appender::non_blocking(LocalDaily::new(dir));
    let _ = GUARD.set(guard);

    // Our own modules at debug (state transitions, timings), everything else at info.
    let filter = EnvFilter::try_from_env("FUCKYOUFLOW_LOG").unwrap_or_else(|_| EnvFilter::new("info,fuckyouflow_lib=debug"));

    let file_layer = fmt::layer().with_ansi(false).with_target(true).with_timer(LocalTime).with_writer(non_blocking);
    let stderr_layer =
        fmt::layer().with_ansi(false).with_target(false).with_timer(LocalTime).with_writer(std::io::stderr);

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stderr_layer)
        .try_init();
}

pub fn set_redaction(enabled: bool) {
    REDACT.store(enabled, Ordering::Relaxed);
}

/// Returns text safe for logs: the real text only when redaction is off,
/// otherwise a length marker.
pub fn redact(text: &str) -> String {
    if REDACT.load(Ordering::Relaxed) {
        format!("<redacted {} chars>", text.chars().count())
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_into_a_file_named_for_the_local_day() {
        let dir = std::env::temp_dir().join(format!("lalia-log-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut w = LocalDaily::new(dir.clone());
        w.write_all(b"hello\n").expect("write");
        w.flush().expect("flush");

        let expected = dir.join(format!("{PREFIX}{}", LocalDaily::today()));
        let body = std::fs::read_to_string(&expected).expect("file named for the local day");
        assert_eq!(body, "hello\n");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn prune_keeps_the_last_seven_days() {
        let dir = std::env::temp_dir().join(format!("lalia-prune-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        for day in 1..=10 {
            std::fs::write(dir.join(format!("{PREFIX}2026-09-{day:02}")), b"x").expect("seed");
        }
        std::fs::write(dir.join("events-2026-09-01.jsonl"), b"x").expect("seed journal");

        prune(&dir);

        let mut left: Vec<String> = std::fs::read_dir(&dir)
            .expect("read")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with(PREFIX))
            .collect();
        left.sort();
        assert_eq!(left.len(), KEEP_DAYS, "keeps a week");
        assert_eq!(left.first().map(String::as_str), Some("fuckyouflow.log.2026-09-04"));
        assert!(dir.join("events-2026-09-01.jsonl").exists(), "leaves the journal alone");

        let _ = std::fs::remove_dir_all(&dir);
    }

    fn names_in(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .expect("read")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    /// Logs from the builds named Lalia sit in the same folder after the
    /// rename. They are older, so they go first, whatever their name sorts to.
    #[test]
    fn prune_removes_old_lalia_logs_before_new_ones() {
        let dir = std::env::temp_dir().join(format!("fuckyouflow-prune-rename-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        for day in 5..=10 {
            std::fs::write(dir.join(format!("{OLD_PREFIX}2026-09-{day:02}")), b"x").expect("seed old");
        }
        for day in 10..=16 {
            std::fs::write(dir.join(format!("{PREFIX}2026-09-{day:02}")), b"x").expect("seed new");
        }

        prune(&dir);

        let left = names_in(&dir);
        let expected: Vec<String> = (10..=16).map(|day| format!("{PREFIX}2026-09-{day:02}")).collect();
        assert_eq!(left, expected, "the week of new logs stays and every old one goes");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Diagnostics reads the newest file of the running program, never an old
    /// Lalia log or the crash log, whatever sorts last by name.
    #[test]
    fn newest_log_is_the_current_programs_latest_day() {
        let dir = std::env::temp_dir().join(format!("fuckyouflow-newest-log-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        for name in [
            format!("{PREFIX}2026-09-15"),
            format!("{PREFIX}2026-09-16"),
            format!("{OLD_PREFIX}2026-09-09"),
            "panic.log".to_string(),
            "events-2026-09-16.jsonl".to_string(),
        ] {
            std::fs::write(dir.join(name), b"x").expect("seed");
        }

        assert_eq!(newest_log(&dir), Some(dir.join(format!("{PREFIX}2026-09-16"))));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
