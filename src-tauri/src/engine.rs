//! Owns the active speech engine, starts and restarts it, and picks the right
//! provider from settings.

use std::sync::Arc;

use parking_lot::RwLock;
use tauri::Emitter;

use crate::asr::openai_compat::OpenAiCompat;
use crate::asr::whisper_server::{find_runtime_exe, WhisperServer, WhisperServerConfig};
use crate::asr::{AsrError, EngineInfo, EngineStatus, TranscriptionProvider, TranscriptionRequest, TranscriptionResult};
use crate::settings::Settings;

pub struct EngineManager {
    local: RwLock<Option<Arc<WhisperServer>>>,
    cloud: RwLock<Option<Arc<OpenAiCompat>>>,
    provider_name: RwLock<String>,
    starting: std::sync::atomic::AtomicBool,
    port: std::sync::atomic::AtomicU16,
}

/// Which engine build to run. "auto" prefers Vulkan on any real graphics card
/// (it measured as fast as CUDA on the RTX 3070 and it is the only option on
/// AMD and Intel), then CUDA, then the CPU. A wish for a backend whose build is
/// missing falls through to the next one.
fn choose_backend(wish: &str, use_gpu: bool) -> (String, Option<std::path::PathBuf>) {
    let hw = crate::hw::detect();
    let has_gpu = hw.best_gpu().is_some();
    let order: Vec<&str> = match wish {
        "cpu" => vec!["cpu"],
        "cuda" => vec!["cuda", "vulkan", "cpu"],
        "vulkan" => vec!["vulkan", "cuda", "cpu"],
        _ => vec!["vulkan", "cuda", "cpu"],
    };
    for b in order {
        let ok = match b {
            "vulkan" => use_gpu && has_gpu && hw.vulkan_runtime,
            "cuda" => use_gpu && hw.cuda_driver,
            _ => true,
        };
        if !ok {
            continue;
        }
        if let Some(exe) = crate::asr::whisper_server::find_runtime_exe_for(b) {
            return (b.to_string(), Some(exe));
        }
    }
    ("cpu".into(), crate::asr::whisper_server::find_runtime_exe())
}

impl EngineManager {
    pub fn new() -> Self {
        Self {
            local: RwLock::new(None),
            cloud: RwLock::new(None),
            provider_name: RwLock::new("whisper_local".into()),
            starting: std::sync::atomic::AtomicBool::new(false),
            port: std::sync::atomic::AtomicU16::new(0),
        }
    }

    fn pick_port(&self) -> u16 {
        let current = self.port.load(std::sync::atomic::Ordering::SeqCst);
        if current != 0 {
            return current;
        }
        // any free port near 47xxx
        let port = std::net::TcpListener::bind("127.0.0.1:0").ok().and_then(|l| l.local_addr().ok()).map(|a| a.port()).unwrap_or(47421);
        self.port.store(port, std::sync::atomic::Ordering::SeqCst);
        port
    }

    /// (Re)start according to settings. Emits "lalia://engine" with EngineInfo.
    pub async fn apply(self: &Arc<Self>, app: &tauri::AppHandle, settings: &Settings) {
        *self.provider_name.write() = settings.asr.provider.clone();
        if settings.asr.provider == "openai_compatible" {
            *self.cloud.write() = Some(Arc::new(OpenAiCompat::new(settings.asr.openai_base_url.clone(), settings.asr.openai_model.clone())));
            let local = self.local.read().clone();
            if let Some(local) = local {
                local.stop().await;
            }
            self.emit(app);
            return;
        }
        if self.starting.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        let (backend, exe) = choose_backend(&settings.asr.backend, settings.asr.use_gpu);
        let model = crate::models::find_model(&settings.asr.model_id);
        let (Some(exe), Some((spec, model_path))) = (exe, model) else {
            let has_runtime = find_runtime_exe().is_some();
            tracing::warn!("engine not started: runtime found = {}, model '{}' known = {}", has_runtime, settings.asr.model_id, crate::models::find_model(&settings.asr.model_id).is_some());
            // Name the missing piece. "Speech engine not installed" on its own
            // sent the first AMD tester hunting through Settings on
            // 7 September 2026 with nothing to go on.
            let message = if has_runtime {
                format!("the model {} is not on this machine; choose another one under Speech models", settings.asr.model_id)
            } else {
                "the speech engine files are missing from this installation; reinstall the app to restore them".to_string()
            };
            self.starting.store(false, std::sync::atomic::Ordering::SeqCst);
            let _ = app.emit(
                "lalia://engine",
                EngineInfo { status: EngineStatus::Missing, provider: "whisper_local".into(), model_id: settings.asr.model_id.clone(), gpu: false, message: Some(message), warm_ms: None, backend: String::new() },
            );
            return;
        };
        if !model_path.exists() {
            tracing::warn!("engine not started: model file missing at {}", model_path.display());
            self.starting.store(false, std::sync::atomic::Ordering::SeqCst);
            let _ = app.emit(
                "lalia://engine",
                EngineInfo { status: EngineStatus::Missing, provider: "whisper_local".into(), model_id: spec.id.clone(), gpu: false, message: Some(format!("model {} not downloaded", spec.display_name)), warm_ms: None, backend: String::new() },
            );
            return;
        }
        let use_gpu = backend != "cpu";
        tracing::info!("speech engine: backend {backend}, {}", exe.display());
        let cfg = WhisperServerConfig {
            exe,
            model_path,
            model_id: spec.id.clone(),
            vad_model: if settings.asr.vad { crate::models::vad_path() } else { None },
            use_gpu,
            threads: settings.asr.threads.max(1),
            port: self.pick_port(),
            backend: backend.clone(),
        };
        let server = Arc::new(WhisperServer::new(cfg));
        let old = self.local.write().replace(server.clone());
        if let Some(old) = old {
            old.stop().await;
        }
        let _ = app.emit("lalia://engine", server.info());
        let mut result = server.start().await;
        if result.is_err() && use_gpu {
            tracing::warn!("GPU start failed, retrying on CPU");
            let mut cfg = server.config().clone();
            cfg.use_gpu = false;
            let cpu = Arc::new(WhisperServer::new(cfg));
            *self.local.write() = Some(cpu.clone());
            result = cpu.start().await;
            let mut info = cpu.info();
            if result.is_ok() {
                info.message = Some("The graphics card did not respond; running on the CPU (slower).".into());
            }
            let _ = app.emit("lalia://engine", info);
        } else {
            self.emit(app);
        }
        if let Err(e) = result {
            tracing::error!("engine start failed: {e}");
        }
        self.starting.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn emit(&self, app: &tauri::AppHandle) {
        let _ = app.emit("lalia://engine", self.info());
    }

    pub fn info(&self) -> EngineInfo {
        if self.provider_name.read().as_str() == "openai_compatible" {
            if let Some(c) = self.cloud.read().clone() {
                return c.info();
            }
        }
        match self.local.read().clone() {
            Some(s) => s.info(),
            None => EngineInfo { status: EngineStatus::Missing, provider: "whisper_local".into(), model_id: String::new(), gpu: false, message: Some("engine not started".into()), warm_ms: None, backend: String::new() },
        }
    }

    pub fn is_ready(&self) -> bool {
        self.info().status == EngineStatus::Ready
    }

    /// The process holding the speech model, when it is a local one. A cloud
    /// provider has none, and neither does an engine that never started.
    pub fn local_pid(&self) -> Option<u32> {
        self.local.read().clone().and_then(|s| s.pid())
    }

    pub async fn transcribe(&self, req: TranscriptionRequest) -> Result<TranscriptionResult, AsrError> {
        let is_cloud = self.provider_name.read().as_str() == "openai_compatible";
        if is_cloud {
            let cloud = self.cloud.read().clone();
            let c = cloud.ok_or_else(|| AsrError::NotReady("cloud provider not configured".into()))?;
            return c.transcribe(req).await;
        }
        let local = self.local.read().clone();
        let s = local.ok_or_else(|| AsrError::NotReady("engine not started".into()))?;
        if !s.is_alive() {
            return Err(AsrError::NotReady("engine process is not running".into()));
        }
        s.transcribe(req).await
    }

    pub async fn stop(&self) {
        let local = self.local.read().clone();
        if let Some(s) = local {
            s.stop().await;
        }
    }

    /// Called by the watchdog: restart if the child died.
    pub async fn ensure_alive(self: &Arc<Self>, app: &tauri::AppHandle, settings: &Settings) {
        if self.provider_name.read().as_str() != "whisper_local" {
            return;
        }
        let local = self.local.read().clone();
        let dead = match local {
            Some(s) => s.info().status == EngineStatus::Ready && !s.is_alive(),
            None => false,
        };
        if dead {
            tracing::warn!("whisper-server died, restarting");
            self.apply(app, settings).await;
        }
    }
}
