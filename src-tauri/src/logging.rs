//! Structured local logs. Transcript text and audio are never logged unless the
//! user disables redaction in Privacy settings (and even then only text, never audio).

use std::sync::atomic::{AtomicBool, Ordering};

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static REDACT: AtomicBool = AtomicBool::new(true);
static GUARD: once_cell::sync::OnceCell<tracing_appender::non_blocking::WorkerGuard> =
    once_cell::sync::OnceCell::new();

pub fn init() {
    let dir = crate::paths::logs_dir();
    let _ = std::fs::create_dir_all(&dir);
    let file_appender = tracing_appender::rolling::daily(&dir, "lalia.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let _ = GUARD.set(guard);

    // Our own modules at debug (state transitions, timings), everything else at info.
    let filter = EnvFilter::try_from_env("LALIA_LOG").unwrap_or_else(|_| EnvFilter::new("info,lalia_lib=debug"));

    let file_layer = fmt::layer().with_ansi(false).with_target(true).with_writer(non_blocking);
    let stderr_layer = fmt::layer().with_ansi(false).with_target(false).with_writer(std::io::stderr);

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
