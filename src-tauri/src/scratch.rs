//! The little notepad of our own.
//!
//! A dictation that cannot reach the window the user was in still produced
//! words, and those words have to end up somewhere the user can see. The
//! clipboard alone is not that place: it is invisible, and a first tester on
//! 7 September 2026 pressed the key twice with no text box in front, spoke,
//! saw nothing appear and concluded the app was broken. It was not, and the
//! sentences were on the clipboard the whole time.
//!
//! So: a small window on top, the text inside it, already selected, with a
//! copy button. It is also where a transcribed audio file lands.

use parking_lot::Mutex;
use tauri::{Emitter, Manager};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScratchPayload {
    pub text: String,
    /// Why the text is here rather than in the user's app, as a code the
    /// interface turns into a sentence in the user's own language:
    /// "not_taken", "window_closed", "elevated", "focus_lost",
    /// "insert_failed" or "file".
    pub reason: String,
    /// Extra context for the sentence, such as the name of the sound file.
    pub detail: Option<String>,
    /// "insert" (a dictation that could not land) or "file" (a transcribed file).
    pub source: String,
}

/// The last thing shown, so a webview that reloads can ask for it again
/// instead of coming up empty.
static LAST: Mutex<Option<ScratchPayload>> = Mutex::new(None);

pub fn last() -> Option<ScratchPayload> {
    LAST.lock().clone()
}

/// Show `text` in the notepad window and bring it to the front.
pub fn show(app: &tauri::AppHandle, text: &str, reason: &str, detail: Option<String>, source: &str) {
    if text.trim().is_empty() {
        return;
    }
    let payload = ScratchPayload { text: text.trim().to_string(), reason: reason.to_string(), detail, source: source.to_string() };
    *LAST.lock() = Some(payload.clone());
    let Some(w) = app.get_webview_window("scratch") else {
        tracing::warn!("the notepad window does not exist; text kept on the clipboard only");
        return;
    };
    let _ = app.emit_to("scratch", "lalia://scratch", &payload);
    let _ = w.show();
    let _ = w.unminimize();
    let _ = w.set_focus();
    tracing::info!("notepad shown ({source}): {} chars", payload.text.chars().count());
    crate::journal::info("scratch.shown", serde_json::json!({ "source": source, "reason": reason, "chars": payload.text.chars().count() }));
}
