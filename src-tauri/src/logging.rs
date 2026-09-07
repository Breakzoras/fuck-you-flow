//! Structured local logs. Transcript text and audio are never logged unless the
//! user disables redaction in Privacy settings (and even then only text, never audio).
//!
//! `%LOCALAPPDATA%\Lalia\logs\lalia.log.YYYY-MM-DD`, seven days kept. Both the
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

const PREFIX: &str = "lalia.log.";
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

/// Keeps a week. Called whenever a new day's file is opened. Without this the
/// text log grew for the life of the install; only the journal was pruned.
fn prune(dir: &Path) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut files: Vec<PathBuf> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with(PREFIX)))
        .collect();
    if files.len() <= KEEP_DAYS {
        return;
    }
    files.sort();
    let cut = files.len() - KEEP_DAYS;
    for p in files.into_iter().take(cut) {
        let _ = std::fs::remove_file(p);
    }
}

pub fn init() {
    let dir = crate::paths::logs_dir();
    let _ = std::fs::create_dir_all(&dir);
    let (non_blocking, guard) = tracing_appender::non_blocking(LocalDaily::new(dir));
    let _ = GUARD.set(guard);

    // Our own modules at debug (state transitions, timings), everything else at info.
    let filter = EnvFilter::try_from_env("LALIA_LOG").unwrap_or_else(|_| EnvFilter::new("info,lalia_lib=debug"));

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
        assert_eq!(left.first().map(String::as_str), Some("lalia.log.2026-09-04"));
        assert!(dir.join("events-2026-09-01.jsonl").exists(), "leaves the journal alone");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
