//! Local engine: whisper.cpp `whisper-server.exe` run as a managed child process.
//!
//! Why a sidecar and not a compiled binding: the official release zip ships a
//! CUDA build with its runtime DLLs, so nothing needs the CUDA toolkit on the
//! user's PC and the app installer stays small. The process is bound to a
//! Windows job object so it dies with the app, and the model stays loaded
//! between dictations (warm).

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use serde::Deserialize;

use super::{AsrError, EngineInfo, EngineStatus, TranscriptionProvider, TranscriptionRequest, TranscriptionResult};

#[derive(Debug, Clone)]
pub struct WhisperServerConfig {
    pub exe: PathBuf,
    pub model_path: PathBuf,
    pub model_id: String,
    pub vad_model: Option<PathBuf>,
    pub use_gpu: bool,
    pub threads: u32,
    pub port: u16,
    /// Which build this is, for the interface: "vulkan", "cuda" or "cpu".
    pub backend: String,
}

struct Inner {
    child: Option<tokio::process::Child>,
    status: EngineStatus,
    message: Option<String>,
    warm_ms: Option<u64>,
    gpu_active: bool,
}

pub struct WhisperServer {
    cfg: WhisperServerConfig,
    inner: Arc<Mutex<Inner>>,
    http: reqwest::Client,
}

#[derive(Deserialize)]
struct Segment {
    text: String,
    #[serde(default)]
    no_speech_prob: Option<f32>,
}

#[derive(Deserialize)]
struct VerboseJson {
    #[serde(default)]
    text: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    detected_language: Option<String>,
    #[serde(default)]
    detected_language_probability: Option<f32>,
    #[serde(default)]
    segments: Vec<Segment>,
}

impl WhisperServer {
    pub fn new(cfg: WhisperServerConfig) -> Self {
        // no_proxy: a system proxy (VPN clients, corporate settings) must never sit
        // between the app and its own local engine.
        let http = reqwest::Client::builder().no_proxy().timeout(Duration::from_secs(65)).build().expect("http client");
        Self {
            cfg,
            inner: Arc::new(Mutex::new(Inner { child: None, status: EngineStatus::Missing, message: None, warm_ms: None, gpu_active: false })),
            http,
        }
    }

    pub fn config(&self) -> &WhisperServerConfig {
        &self.cfg
    }

    fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.cfg.port)
    }

    /// Spawn the process and wait until the model answers a tiny request.
    pub async fn start(&self) -> Result<(), AsrError> {
        if !self.cfg.exe.exists() {
            self.set(EngineStatus::Missing, Some("whisper-server.exe missing".into()));
            return Err(AsrError::NotReady("runtime missing".into()));
        }
        if !self.cfg.model_path.exists() {
            self.set(EngineStatus::Missing, Some(format!("model file missing: {}", self.cfg.model_path.display())));
            return Err(AsrError::NotReady("model missing".into()));
        }
        self.stop().await;
        self.set(EngineStatus::Starting, None);
        let started = Instant::now();

        let mut cmd = tokio::process::Command::new(&self.cfg.exe);
        cmd.arg("-m")
            .arg(&self.cfg.model_path)
            .arg("--host")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(self.cfg.port.to_string())
            .arg("-t")
            .arg(self.cfg.threads.to_string())
            .arg("-l")
            .arg("auto")
            .arg("--no-timestamps")
            .arg("--suppress-nst")
            .arg("--flash-attn")
            .arg("--inference-path")
            .arg("/inference");
        if !self.cfg.use_gpu {
            cmd.arg("--no-gpu");
        }
        if let Some(vad) = &self.cfg.vad_model {
            if vad.exists() {
                cmd.arg("--vad").arg("--vad-model").arg(vad).arg("--vad-threshold").arg("0.5");
            }
        }
        if let Some(dir) = self.cfg.exe.parent() {
            cmd.current_dir(dir);
        }
        cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        cmd.kill_on_drop(true);

        let mut child = cmd.spawn().map_err(|e| {
            self.set(EngineStatus::Failed, Some(format!("cannot start whisper-server: {e}")));
            AsrError::NotReady(e.to_string())
        })?;

        #[cfg(windows)]
        if let Some(pid) = child.id() {
            crate::jobobject::assign_to_app_job(pid);
        }

        // Drain stderr into the log; also detect CUDA usage.
        let inner = self.inner.clone();
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut lines = tokio::io::BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if (line.contains("CUDA") || line.contains("Vulkan")) && (line.contains("found") || line.contains("Device") || line.contains("device")) {
                        inner.lock().gpu_active = true;
                    }
                    if line.contains("error") || line.contains("failed") {
                        tracing::warn!(target: "whisper-server", "{line}");
                    } else if line.contains("listening") || line.contains("CUDA") || line.contains("Vulkan") || line.contains("model size") || line.contains("backend") {
                        tracing::info!(target: "whisper-server", "{line}");
                    } else {
                        tracing::debug!(target: "whisper-server", "{line}");
                    }
                }
            });
        }
        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut lines = tokio::io::BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if !line.trim().is_empty() {
                        tracing::info!(target: "whisper-server", "{line}");
                    }
                }
            });
        }
        self.inner.lock().child = Some(child);

        // Wait for the port, then warm the model with 0.5 s of silence.
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            if Instant::now() > deadline {
                self.set(EngineStatus::Failed, Some("whisper-server did not become ready in 120 s".into()));
                self.stop().await;
                return Err(AsrError::NotReady("timeout".into()));
            }
            let exited: Option<String> = {
                let mut g = self.inner.lock();
                match g.child.as_mut() {
                    Some(child) => match child.try_wait() {
                        Ok(Some(status)) => Some(status.to_string()),
                        _ => None,
                    },
                    None => None,
                }
            };
            if let Some(status) = exited {
                self.set(EngineStatus::Failed, Some(format!("whisper-server exited early ({status})")));
                return Err(AsrError::NotReady("process exited".into()));
            }
            if tokio::net::TcpStream::connect(("127.0.0.1", self.cfg.port)).await.is_ok() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        let silence = crate::audio::encode_wav(&vec![0.0f32; 8000]);
        let warm = self
            .transcribe(TranscriptionRequest { wav: silence, language: "en".into(), prompt: None, beam_size: 1, vad: false })
            .await;
        match warm {
            Ok(_) => {
                let ms = started.elapsed().as_millis() as u64;
                {
                    let mut g = self.inner.lock();
                    g.status = EngineStatus::Ready;
                    g.message = None;
                    g.warm_ms = Some(ms);
                }
                tracing::info!("whisper-server ready in {ms} ms (model {})", self.cfg.model_id);
                crate::journal::info(
                    "engine.ready",
                    serde_json::json!({ "backend": self.cfg.backend, "model": self.cfg.model_id, "warm_ms": ms }),
                );
                Ok(())
            }
            Err(e) => {
                self.set(EngineStatus::Failed, Some(format!("warm-up failed: {e}")));
                Err(e)
            }
        }
    }

    pub async fn stop(&self) {
        let child = self.inner.lock().child.take();
        if let Some(mut c) = child {
            let _ = c.kill().await;
            let _ = c.wait().await;
        }
        let mut g = self.inner.lock();
        if g.status != EngineStatus::Missing {
            g.status = EngineStatus::Stopped;
        }
    }

    pub fn is_ready(&self) -> bool {
        self.inner.lock().status == EngineStatus::Ready
    }

    /// True when the child process is still alive.
    pub fn is_alive(&self) -> bool {
        let mut g = self.inner.lock();
        match g.child.as_mut() {
            Some(c) => matches!(c.try_wait(), Ok(None)),
            None => false,
        }
    }

    fn set(&self, status: EngineStatus, message: Option<String>) {
        let mut g = self.inner.lock();
        g.status = status;
        g.message = message;
    }
}

#[async_trait::async_trait]
impl TranscriptionProvider for WhisperServer {
    fn name(&self) -> &'static str {
        "whisper_local"
    }

    async fn transcribe(&self, req: TranscriptionRequest) -> Result<TranscriptionResult, AsrError> {
        let started = Instant::now();
        let wav_len = req.wav.len();
        let file = reqwest::multipart::Part::bytes(req.wav)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| AsrError::Request(e.to_string()))?;
        let mut form = reqwest::multipart::Form::new()
            .part("file", file)
            // "json" and not "verbose_json": the verbose variant makes the server
            // resolve a language id even when VAD removed all audio, which crashes it.
            .text("response_format", "json")
            .text("temperature", "0.0")
            .text("temperature_inc", "0.2")
            .text("no_timestamps", "true")
            .text("suppress_non_speech", "true")
            .text("language", req.language.clone())
            .text("vad", if req.vad { "true" } else { "false" })
            .text("beam_size", req.beam_size.max(1).to_string());
        if let Some(p) = req.prompt.filter(|p| !p.trim().is_empty()) {
            form = form.text("prompt", p);
        }
        // Inference time grows with the recording. Measured on the RTX 3070 with
        // large-v3-q5_0: about 0.09 s per second of audio, so a ten-minute
        // recording needs close to a minute. Allow half a second per audio second
        // plus a fixed margin, never less than the client default.
        let audio_secs = wav_len as u64 / 32_000;
        let timeout = Duration::from_secs((20 + audio_secs / 2).max(65));
        let resp = self
            .http
            .post(format!("{}/inference", self.base_url()))
            .timeout(timeout)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AsrError::Request(format!("{e} (cause: {:?})", std::error::Error::source(&e).map(|s| s.to_string()))))?;
        let status = resp.status();
        let body = resp.text().await.map_err(|e| AsrError::Request(e.to_string()))?;
        if !status.is_success() {
            return Err(AsrError::Request(format!("HTTP {status}: {}", body.chars().take(200).collect::<String>())));
        }
        let parsed: VerboseJson = serde_json::from_str(&body).map_err(|e| AsrError::BadResponse(format!("{e}: {}", body.chars().take(200).collect::<String>())))?;
        let text = if parsed.text.trim().is_empty() {
            parsed.segments.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join("")
        } else {
            parsed.text
        };
        let no_speech = parsed.segments.iter().filter_map(|s| s.no_speech_prob).fold(None, |m: Option<f32>, v| Some(m.map_or(v, |x| x.max(v))));
        let lang = parsed.detected_language.or(parsed.language);
        Ok(TranscriptionResult {
            text: text.trim().to_string(),
            detected_language: lang.map(normalize_lang),
            language_probability: parsed.detected_language_probability,
            no_speech_prob: no_speech,
            engine: "whisper_local".into(),
            model: self.cfg.model_id.clone(),
            inference_ms: started.elapsed().as_millis() as u64,
        })
    }

    fn info(&self) -> EngineInfo {
        let g = self.inner.lock();
        EngineInfo {
            status: g.status.clone(),
            provider: "whisper_local".into(),
            model_id: self.cfg.model_id.clone(),
            gpu: self.cfg.use_gpu && g.gpu_active,
            message: g.message.clone(),
            warm_ms: g.warm_ms,
            backend: if self.cfg.use_gpu { self.cfg.backend.clone() } else { "cpu".into() },
        }
    }
}

/// whisper.cpp reports full names ("greek", "english"); the app uses ISO codes.
fn normalize_lang(s: String) -> String {
    match s.to_ascii_lowercase().as_str() {
        "greek" | "el" => "el".into(),
        "english" | "en" => "en".into(),
        other => other.to_string(),
    }
}

/// Locate whisper-server.exe: the app runtime dir first, then a dev checkout.
/// Any engine at all, for "is the runtime installed" checks.
pub fn find_runtime_exe() -> Option<PathBuf> {
    find_runtime_exe_for("vulkan").or_else(|| find_runtime_exe_for("cuda"))
}

/// The engine build for a backend: "vulkan" (works on AMD, Intel and NVIDIA,
/// 54 MB), "cuda" (NVIDIA only, 1.1 GB) or "cpu" (either build with the GPU
/// switched off). Installer copies first, then the developer checkout.
pub fn find_runtime_exe_for(backend: &str) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    let bundled = crate::paths::bundled_dir().cloned();
    let vendor = dev_vendor_dir();
    let vulkan_dirs = |c: &mut Vec<PathBuf>| {
        if let Some(b) = &bundled {
            c.push(b.join("engine-vulkan"));
        }
        c.push(vendor.parent().map(|p| p.join("whisper-vulkan")).unwrap_or_default());
    };
    let cuda_dirs = |c: &mut Vec<PathBuf>| {
        c.push(crate::paths::whisper_runtime_dir());
        if let Some(b) = &bundled {
            c.push(b.join("engine-cuda"));
            c.push(b.join("engine"));
        }
        c.push(vendor.join("Release"));
    };
    match backend {
        "cuda" => cuda_dirs(&mut candidates),
        "vulkan" => vulkan_dirs(&mut candidates),
        _ => {
            vulkan_dirs(&mut candidates);
            cuda_dirs(&mut candidates);
        }
    }
    candidates.into_iter().map(|d| d.join("whisper-server.exe")).find(|p| p.exists())
}

pub fn dev_vendor_dir() -> PathBuf {
    // <repo>/src-tauri -> <repo>/vendor/whisper
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    here.parent().map(|p| p.join("vendor").join("whisper")).unwrap_or_else(|| PathBuf::from("vendor/whisper"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_verbose_json() {
        let body = r#"{"text":" Γεια σου κόσμε.","language":"greek","segments":[{"text":" Γεια σου κόσμε.","no_speech_prob":0.02}]}"#;
        let p: VerboseJson = serde_json::from_str(body).unwrap();
        assert_eq!(p.segments.len(), 1);
        assert_eq!(normalize_lang(p.language.unwrap()), "el");
    }
}

#[cfg(test)]
mod live_tests {
    use super::*;

    /// Manual check against a whisper-server started by hand on port 47555.
    /// Run: cargo test live_inference -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn live_inference() {
        let cfg = WhisperServerConfig {
            exe: PathBuf::from("x"),
            model_path: PathBuf::from("x"),
            model_id: "manual".into(),
            vad_model: None,
            backend: "cuda".into(),
            use_gpu: true,
            threads: 8,
            port: 47555,
        };
        let s = WhisperServer::new(cfg);
        let silence = crate::audio::encode_wav(&vec![0.0f32; 8000]);
        let r = s.transcribe(TranscriptionRequest { wav: silence, language: "en".into(), prompt: None, beam_size: 1, vad: false }).await;
        println!("silence -> {r:?}");
        let wav = std::fs::read("C:/Claude Projects/lalia/eval/corpus/el-01.wav").unwrap();
        let r = s.transcribe(TranscriptionRequest { wav, language: "el".into(), prompt: None, beam_size: 5, vad: true }).await;
        println!("el-01 -> {r:?}");
        assert!(r.is_ok());
    }
}
