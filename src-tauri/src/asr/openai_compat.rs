//! Optional cloud engine (disabled by default): any OpenAI-compatible
//! `/audio/transcriptions` endpoint. The API key lives in Windows Credential
//! Manager, never in settings.json.

use std::time::{Duration, Instant};

use super::{AsrError, EngineInfo, EngineStatus, TranscriptionProvider, TranscriptionRequest, TranscriptionResult};

pub const KEYRING_SERVICE: &str = "Lalia";
pub const KEYRING_USER: &str = "openai_compatible_api_key";

pub struct OpenAiCompat {
    base_url: String,
    model: String,
    http: reqwest::Client,
}

impl OpenAiCompat {
    pub fn new(base_url: String, model: String) -> Self {
        let http = reqwest::Client::builder().timeout(Duration::from_secs(60)).build().expect("http client");
        Self { base_url: base_url.trim_end_matches('/').to_string(), model, http }
    }

    pub fn store_key(key: &str) -> Result<(), String> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| e.to_string())?;
        entry.set_password(key).map_err(|e| e.to_string())
    }

    pub fn delete_key() -> Result<(), String> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| e.to_string())?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn has_key() -> bool {
        keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).and_then(|e| e.get_password()).map(|k| !k.is_empty()).unwrap_or(false)
    }

    fn key() -> Result<String, AsrError> {
        keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
            .and_then(|e| e.get_password())
            .map_err(|_| AsrError::NotReady("no API key stored".into()))
    }
}

#[async_trait::async_trait]
impl TranscriptionProvider for OpenAiCompat {
    fn name(&self) -> &'static str {
        "openai_compatible"
    }

    async fn transcribe(&self, req: TranscriptionRequest) -> Result<TranscriptionResult, AsrError> {
        let key = Self::key()?;
        let started = Instant::now();
        let file = reqwest::multipart::Part::bytes(req.wav).file_name("audio.wav").mime_str("audio/wav").map_err(|e| AsrError::Request(e.to_string()))?;
        let mut form = reqwest::multipart::Form::new().part("file", file).text("model", self.model.clone()).text("response_format", "json");
        if req.language != "auto" {
            form = form.text("language", req.language.clone());
        }
        if let Some(p) = req.prompt.filter(|p| !p.trim().is_empty()) {
            form = form.text("prompt", p);
        }
        let resp = self
            .http
            .post(format!("{}/audio/transcriptions", self.base_url))
            .bearer_auth(key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AsrError::Offline(e.to_string()))?;
        let status = resp.status();
        let body = resp.text().await.map_err(|e| AsrError::Request(e.to_string()))?;
        if !status.is_success() {
            return Err(AsrError::Request(format!("HTTP {status}: {}", body.chars().take(200).collect::<String>())));
        }
        let v: serde_json::Value = serde_json::from_str(&body).map_err(|e| AsrError::BadResponse(e.to_string()))?;
        let text = v.get("text").and_then(|t| t.as_str()).unwrap_or("").trim().to_string();
        Ok(TranscriptionResult {
            text,
            detected_language: v.get("language").and_then(|l| l.as_str()).map(|s| s.to_string()),
            language_probability: None,
            no_speech_prob: None,
            engine: "openai_compatible".into(),
            model: self.model.clone(),
            inference_ms: started.elapsed().as_millis() as u64,
        })
    }

    fn info(&self) -> EngineInfo {
        EngineInfo {
            status: if Self::has_key() { EngineStatus::Ready } else { EngineStatus::Missing },
            provider: "openai_compatible".into(),
            model_id: self.model.clone(),
            gpu: false,
            message: if Self::has_key() { None } else { Some("no API key stored".into()) },
            warm_ms: None,
            backend: "cloud".into(),
        }
    }
}
