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

impl Settings {
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            // Notepad and PowerShell write UTF-8 with a byte-order mark, which
            // serde_json rejects as "expected value at line 1 column 1".
            Ok(text) => match serde_json::from_str::<Settings>(text.trim_start_matches('\u{feff}')) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("settings.json unreadable ({e}); using defaults and keeping a backup");
                    let _ = std::fs::copy(path, path.with_extension("json.bak"));
                    Settings::default()
                }
            },
            Err(_) => Settings::default(),
        }
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
