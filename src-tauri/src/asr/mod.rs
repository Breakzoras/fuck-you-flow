//! Speech-to-text providers behind one interface. The pipeline never knows which
//! engine is behind it.

pub mod openai_compat;
pub mod whisper_server;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TranscriptionRequest {
    /// 16 kHz mono 16-bit WAV bytes.
    pub wav: Vec<u8>,
    /// "el", "en" or "auto".
    pub language: String,
    /// Recognition hint (dictionary terms, previous context).
    pub prompt: Option<String>,
    pub beam_size: u32,
    /// Server-side voice activity detection. Off for warm-up requests: with VAD on,
    /// whisper-server crashes on audio that contains no speech at all.
    pub vad: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TranscriptionResult {
    pub text: String,
    pub detected_language: Option<String>,
    pub language_probability: Option<f32>,
    /// Highest no-speech probability among segments (1.0 = surely silence).
    pub no_speech_prob: Option<f32>,
    pub engine: String,
    pub model: String,
    pub inference_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EngineStatus {
    /// No runtime or no model on disk.
    Missing,
    Starting,
    Ready,
    Failed,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    pub status: EngineStatus,
    pub provider: String,
    pub model_id: String,
    pub gpu: bool,
    pub message: Option<String>,
    pub warm_ms: Option<u64>,
    /// "vulkan", "cuda", "cpu" or "cloud"
    #[serde(default)]
    pub backend: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AsrError {
    #[error("engine not ready: {0}")]
    NotReady(String),
    #[error("engine request failed: {0}")]
    Request(String),
    #[error("engine returned bad data: {0}")]
    BadResponse(String),
    #[error("cloud provider unavailable: {0}")]
    Offline(String),
}

#[async_trait::async_trait]
pub trait TranscriptionProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn transcribe(&self, req: TranscriptionRequest) -> Result<TranscriptionResult, AsrError>;
    fn info(&self) -> EngineInfo;
}
