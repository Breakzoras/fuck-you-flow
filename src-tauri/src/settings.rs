//! Typed settings, persisted as JSON. Every field has a default so that an old
//! settings file keeps working when new fields are added.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LanguageMode {
    Greek,
    English,
    Auto,
    Multi,
}

impl LanguageMode {
    /// Value passed to whisper-server. Multi means "auto" with a bilingual prompt.
    pub fn whisper_code(&self) -> &'static str {
        match self {
            LanguageMode::Greek => "el",
            LanguageMode::English => "en",
            LanguageMode::Auto | LanguageMode::Multi => "auto",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CleanupIntensity {
    Off,
    Light,
    Normal,
    Strong,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InsertionMethod {
    Auto,
    Paste,
    Type,
    CopyOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverlayPosition {
    BottomCenter,
    TopCenter,
    BottomRight,
    BottomLeft,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HotkeySettings {
    /// Hold to record. Release to transcribe and insert.
    pub push_to_talk: String,
    /// Press once to start, press again to stop and insert.
    pub hands_free: String,
    /// Paste the last successful transcript again.
    pub paste_last: String,
    /// A quick tap (shorter than tap_ms) of the push-to-talk chord toggles hands-free.
    pub tap_toggles_hands_free: bool,
    pub tap_ms: u64,
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            // Right Alt (AltGr), the key right of the space bar. It is a modifier,
            // so holding it is never swallowed and AltGr symbols keep working.
            // Right Alt (AltGr), the key right of the space bar, as a toggle: one
            // press starts listening, the next press stops and inserts the text.
            // It is a modifier, so it is never swallowed and AltGr symbols keep
            // working. Hold-to-talk stays available on Ctrl+Win.
            push_to_talk: "Ctrl+Win".into(),
            hands_free: "RAlt".into(),
            paste_last: "Shift+LAlt+Z".into(),
            tap_toggles_hands_free: true,
            tap_ms: 280,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    /// Device name as reported by the OS. None = system default.
    pub device_name: Option<String>,
    /// Keep the microphone stream open so the first syllable is never clipped.
    pub keep_stream_warm: bool,
    pub preroll_ms: u64,
    /// Recordings with less speech than this are rejected as empty.
    pub min_speech_ms: u64,
    /// Hard cap for a single dictation (safety against a stuck key).
    pub max_recording_seconds: u64,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            device_name: None,
            keep_stream_warm: true,
            preroll_ms: 350,
            min_speech_ms: 200,
            max_recording_seconds: 600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AsrSettings {
    /// "whisper_local" or "openai_compatible"
    pub provider: String,
    pub model_id: String,
    pub use_gpu: bool,
    pub vad: bool,
    pub threads: u32,
    pub beam_size: u32,
    pub hints_from_dictionary: bool,
    /// Max number of dictionary terms to send as a recognition hint.
    pub max_hint_terms: usize,
    /// Transcribe finished phrases during pauses while the key is still held,
    /// so only the last phrase remains when the key is released.
    pub segment_while_speaking: bool,
    /// "auto", "vulkan" (any graphics card), "cuda" (NVIDIA) or "cpu".
    pub backend: String,
    pub openai_base_url: String,
    pub openai_model: String,
}

impl Default for AsrSettings {
    fn default() -> Self {
        Self {
            provider: "whisper_local".into(),
            model_id: "large-v3-q5_0".into(),
            use_gpu: true,
            vad: true,
            threads: 8,
            // 3 instead of 5: same accuracy on the corpus, about 6% faster.
            beam_size: 3,
            hints_from_dictionary: true,
            max_hint_terms: 40,
            segment_while_speaking: true,
            backend: "auto".into(),
            openai_base_url: "https://api.openai.com/v1".into(),
            openai_model: "whisper-1".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CleanupSettings {
    pub intensity: CleanupIntensity,
    pub remove_fillers: bool,
    pub resolve_self_corrections: bool,
    pub auto_punctuate: bool,
    pub auto_capitalize: bool,
    pub llm_enabled: bool,
    pub llm_model_path: Option<String>,
    pub cloud_cleanup_enabled: bool,
    /// Add a question mark when the voice rises at the end of a phrase that
    /// has no question word. Tuned on the developer's voice; can misfire.
    pub intonation_questions: bool,
}

impl Default for CleanupSettings {
    fn default() -> Self {
        Self {
            intensity: CleanupIntensity::Normal,
            remove_fillers: true,
            resolve_self_corrections: true,
            auto_punctuate: true,
            auto_capitalize: true,
            llm_enabled: false,
            llm_model_path: None,
            cloud_cleanup_enabled: false,
            intonation_questions: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InsertionSettings {
    pub method: InsertionMethod,
    pub restore_clipboard: bool,
    /// Milliseconds to wait after the paste keystroke before restoring the clipboard.
    pub paste_settle_ms: u64,
    /// Add a trailing space after inserted text (useful for chat apps).
    pub trailing_space: bool,
    /// Open the notepad window when the words could not reach their target.
    /// On by default, because a first tester lost a whole dictation to the
    /// clipboard without knowing it on 7 September 2026. Off for anyone who
    /// finds a window appearing mid-work more disruptive than the loss, which
    /// Lu did the same evening: the clipboard alone is then the fallback.
    #[serde(default = "yes")]
    pub notepad_when_lost: bool,
}

fn yes() -> bool {
    true
}

impl Default for InsertionSettings {
    fn default() -> Self {
        Self {
            method: InsertionMethod::Auto,
            restore_clipboard: true,
            // Measured 2026-09-05: Electron and Chromium read the clipboard once,
            // 2 to 3 ms after Ctrl+V. 60 ms of quiet is plenty; 180 was dead time.
            paste_settle_ms: 60,
            trailing_space: true,
            notepad_when_lost: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OverlaySettings {
    pub position: OverlayPosition,
    pub custom_x: i32,
    pub custom_y: i32,
    pub monitor_name: Option<String>,
    /// Hide the pill completely while idle (it appears on hotkey press).
    pub hide_when_idle: bool,
    pub scale: f32,
    /// "full" (state text and waveform) or "minimal" (one dot with a coloured ring).
    pub style: String,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            position: OverlayPosition::BottomCenter,
            custom_x: 0,
            custom_y: 0,
            monitor_name: None,
            hide_when_idle: true,
            scale: 1.0,
            style: "full".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PrivacySettings {
    pub keep_history: bool,
    /// None = keep forever.
    pub retention_days: Option<u32>,
    pub keep_audio: bool,
    pub context_awareness: bool,
    pub learning_enabled: bool,
    pub learn_from_edits: bool,
    pub redact_logs: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            keep_history: true,
            retention_days: None,
            keep_audio: false,
            context_awareness: false,
            learning_enabled: true,
            learn_from_edits: false,
            redact_logs: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralSettings {
    /// "el" or "en"
    pub ui_language: String,
    /// "system", "dark", "light"
    pub theme: String,
    pub autostart: bool,
    pub first_run_done: bool,
    pub play_sounds: bool,
    /// The machine was inspected once and threads/model/backend were set from it.
    pub machine_profiled: bool,
    /// Hidden developer mode: records every event to the journal and shows the
    /// Debug panel in Diagnostics. Turned on by clicking the Diagnostics title
    /// five times; nothing in the normal interface mentions it.
    #[serde(default)]
    pub debug_mode: bool,
    /// The user chose the graphics-card switch or the acceleration themselves.
    /// Until they do, the app is allowed to correct a machine that ended up on
    /// the processor while a usable card sits in it, which is what happened to
    /// the first AMD tester on 7 September 2026.
    #[serde(default)]
    pub gpu_choice_by_user: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            ui_language: "en".into(),
            theme: "dark".into(),
            autostart: false,
            first_run_done: false,
            play_sounds: true,
            machine_profiled: false,
            debug_mode: false,
            gpu_choice_by_user: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Settings {
    pub general: GeneralSettings,
    pub hotkeys: HotkeySettings,
    pub audio: AudioSettings,
    pub language: LanguageModeSetting,
    pub asr: AsrSettings,
    pub cleanup: CleanupSettings,
    pub insertion: InsertionSettings,
    pub overlay: OverlaySettings,
    pub privacy: PrivacySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LanguageModeSetting {
    pub mode: LanguageMode,
}

impl Default for LanguageModeSetting {
    fn default() -> Self {
        Self { mode: LanguageMode::Multi }
    }
}

/// Set when settings.json exists but could not be read at all this run.
static READ_FAILED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// True when this run started from defaults because the file would not open.
/// The startup save checks it, so the defaults never land on top of the real
/// file (audit, 11 September 2026).
pub fn read_failed() -> bool {
    READ_FAILED.load(std::sync::atomic::Ordering::SeqCst)
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        // Another program holding the file for a moment (antivirus, a backup or
        // sync tool) fails the read with a sharing error. Any error used to
        // mean defaults, and the startup save then wrote those defaults over
        // the user's real choices. Wait the moment out first.
        let mut last_err = None;
        for attempt in 0..5u64 {
            match std::fs::read(path) {
                Ok(bytes) => {
                    // A file saved as ANSI by an editor is not UTF-8; read what
                    // can be read instead of throwing all of it away.
                    let text = String::from_utf8(bytes).unwrap_or_else(|e| {
                        tracing::warn!("settings.json is not UTF-8; reading what can be read");
                        String::from_utf8_lossy(e.as_bytes()).into_owned()
                    });
                    return Self::parse(path, &text);
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Settings::default(),
                Err(e) => {
                    last_err = Some(e);
                    std::thread::sleep(std::time::Duration::from_millis(100 * (attempt + 1)));
                }
            }
        }
        tracing::warn!("settings.json could not be opened ({:?}); running on defaults and leaving the file as it is", last_err);
        let _ = std::fs::copy(path, path.with_extension("json.unread.bak"));
        READ_FAILED.store(true, std::sync::atomic::Ordering::SeqCst);
        Settings::default()
    }

    fn parse(path: &Path, text: &str) -> Self {
        // Notepad and PowerShell write UTF-8 with a byte-order mark, which
        // serde_json rejects as "expected value at line 1 column 1".
        let text = text.trim_start_matches('\u{feff}');
        match serde_json::from_str::<Settings>(text) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("settings.json has something the app cannot read ({e}); keeping every part that still makes sense");
                let _ = std::fs::copy(path, path.with_extension("json.bak"));
                Settings::salvage(text)
            }
        }
    }

    /// Read the file one section at a time, keeping everything that still works.
    ///
    /// A single word the app does not recognise, a hand edit gone wrong or a
    /// setting removed by a later version used to fail the whole file, and the
    /// user came back to a program that had forgotten their shortcut, their
    /// language, their microphone and everything else they had ever chosen.
    /// Now only the part that is actually broken goes back to its default.
    fn salvage(text: &str) -> Self {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
            tracing::warn!("settings.json is not readable as a file at all; starting from the defaults");
            return Settings::default();
        };
        let Some(obj) = value.as_object() else {
            return Settings::default();
        };
        let mut out = Settings::default();
        macro_rules! part {
            ($name:literal, $field:ident) => {
                if let Some(v) = obj.get($name) {
                    match serde_json::from_value(v.clone()) {
                        Ok(parsed) => out.$field = parsed,
                        Err(e) => tracing::warn!("the {} settings could not be read ({e}); that part went back to its defaults", $name),
                    }
                }
            };
        }
        part!("general", general);
        part!("hotkeys", hotkeys);
        part!("audio", audio);
        part!("language", language);
        part!("asr", asr);
        part!("cleanup", cleanup);
        part!("insertion", insertion);
        part!("overlay", overlay);
        part!("privacy", privacy);
        out
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        {
            // On the disk before the rename, so a power cut cannot leave an
            // empty settings.json behind a successful-looking save.
            use std::io::Write;
            let mut f = std::fs::File::create(&tmp)?;
            f.write_all(serde_json::to_string_pretty(self)?.as_bytes())?;
            f.sync_all()?;
        }
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file an editor saved as ANSI is not UTF-8. It used to count as
    /// unreadable and every choice went back to its default.
    #[test]
    fn a_file_that_is_not_utf8_keeps_its_choices() {
        let dir = std::env::temp_dir().join(format!("fyf-settings-ansi-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        let mut s = Settings::default();
        s.hotkeys.hands_free = "F13".into();
        s.general.ui_language = "el".into();
        let mut bytes = serde_json::to_vec(&s).unwrap();
        // Replace the "F13" value with "F13\xE9": one Latin-1 byte, invalid UTF-8.
        let pos = bytes.windows(5).position(|w| w == b"\"F13\"").unwrap();
        bytes.insert(pos + 4, 0xE9);
        std::fs::write(&path, &bytes).unwrap();
        let got = Settings::load(&path);
        assert_eq!(got.general.ui_language, "el", "the rest of the file survived");
        assert!(!read_failed());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// One word the app does not know must not cost the user everything else
    /// they ever set. Before this, a single bad value reset the shortcut, the
    /// language, the microphone and every other choice at once.
    #[test]
    fn a_broken_section_does_not_take_the_others_with_it() {
        let text = r#"{
            "general": { "ui_language": "en" },
            "hotkeys": { "push_to_talk": "Mouse4" },
            "audio": { "preroll_ms": "not a number" },
            "insertion": { "paste_settle_ms": 120 }
        }"#;
        // The whole file cannot be read, which is the case that used to wipe everything.
        assert!(serde_json::from_str::<Settings>(text).is_err(), "this file must be the broken kind");

        let s = Settings::salvage(text);
        assert_eq!(s.general.ui_language, "en", "the language survives");
        assert_eq!(s.hotkeys.push_to_talk, "Mouse4", "the shortcut survives");
        assert_eq!(s.insertion.paste_settle_ms, 120, "the paste setting survives");
        assert_eq!(
            s.audio.preroll_ms,
            Settings::default().audio.preroll_ms,
            "only the broken part goes back to its default"
        );
    }

    #[test]
    fn defaults_round_trip() {
        let s = Settings::default();
        let json = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.hotkeys.hands_free, "RAlt", "the toggle key is the right Alt");
        assert_eq!(back.hotkeys.push_to_talk, "Ctrl+Win");
        assert_eq!(back.language.mode, LanguageMode::Multi);
        assert!(back.asr.segment_while_speaking);
        assert_eq!(back.general.ui_language, "en");
    }

    #[test]
    fn partial_file_gets_defaults() {
        let back: Settings = serde_json::from_str(r#"{"general":{"ui_language":"en"}}"#).unwrap();
        assert_eq!(back.general.ui_language, "en");
        assert!(back.audio.keep_stream_warm);
    }
}
