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
    /// The settings a caller wanted while a start was already in flight. The
    /// runner picks them up and goes round again rather than dropping them.
    queued: RwLock<Option<Settings>>,
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
            queued: RwLock::new(None),
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
        let mut settings = settings.clone();
        loop {
            if !self.apply_once(app, &settings).await {
                // Another call is starting the engine and will pick the queued
                // settings up when it finishes. Taking them back here made this
                // call spin round its own queue entry for the whole start, up to
                // two minutes of a busy thread and a flooded log (Greptile,
                // 11 September 2026).
                break;
            }
            match self.queued.write().take() {
                Some(next) => {
                    tracing::info!("engine settings changed while starting; going round again");
                    settings = next;
                }
                None => break,
            }
        }
    }

    async fn apply_once(self: &Arc<Self>, app: &tauri::AppHandle, settings: &Settings) -> bool {
        *self.provider_name.write() = settings.asr.provider.clone();
        if settings.asr.provider == "openai_compatible" {
            *self.cloud.write() = Some(Arc::new(OpenAiCompat::new(settings.asr.openai_base_url.clone(), settings.asr.openai_model.clone())));
            let local = self.local.read().clone();
            if let Some(local) = local {
                local.stop().await;
            }
            self.emit(app);
            return true;
        }
        if self.starting.swap(true, std::sync::atomic::Ordering::SeqCst) {
            // Somebody is already starting one. Remember that the settings moved
            // on, so the runner can go round again with the new ones instead of
            // dropping this on the floor. Changing model or backend during a
            // start, or pressing Restart engine, used to do nothing at all for
            // up to two minutes and looked like the setting was ignored.
            *self.queued.write() = Some(settings.clone());
            tracing::info!("engine change queued: one is already starting");
            return false;
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
            // Forget the old one as well. Leaving it in place made info() and
            // transcribe() keep using a server the app had just announced as
            // missing, so an update that removed the install folder mid-session
            // said "engine missing" while dictation carried on regardless.
            *self.local.write() = None;
            self.starting.store(false, std::sync::atomic::Ordering::SeqCst);
            let _ = app.emit(
                "lalia://engine",
                EngineInfo { status: EngineStatus::Missing, provider: "whisper_local".into(), model_id: settings.asr.model_id.clone(), gpu: false, message: Some(message), warm_ms: None, backend: String::new() },
            );
            return true;
        };
        // Not just "is it there": a half-downloaded file is there and passes,
        // and whisper-server then exits with a bare code that names nothing.
        let size_ok = std::fs::metadata(&model_path).map(|m| m.len() == spec.size_bytes).unwrap_or(false);
        if !model_path.exists() || !size_ok {
            if model_path.exists() && !size_ok {
                tracing::warn!("engine not started: {} is the wrong size, the download did not finish", model_path.display());
                *self.local.write() = None;
                self.starting.store(false, std::sync::atomic::Ordering::SeqCst);
                let _ = app.emit(
                    "lalia://engine",
                    EngineInfo { status: EngineStatus::Missing, provider: "whisper_local".into(), model_id: spec.id.clone(), gpu: false, message: Some(format!("the file for {} is incomplete; download it again under Speech models", spec.display_name)), warm_ms: None, backend: String::new() },
                );
                return true;
            }
            tracing::warn!("engine not started: model file missing at {}", model_path.display());
            // Forget the dead server before giving up. Without this the watchdog
            // still sees a handle whose status is Ready and whose process is
            // gone, calls apply, lands here again and repeats every five
            // seconds for as long as the app runs. Measured on 6 September
            // 2026: sixteen rounds in seventy seconds, ended only by restarting
            // the app, with the tray icon present and every dictation failing.
            *self.local.write() = None;
            self.starting.store(false, std::sync::atomic::Ordering::SeqCst);
            let _ = app.emit(
                "lalia://engine",
                EngineInfo { status: EngineStatus::Missing, provider: "whisper_local".into(), model_id: spec.id.clone(), gpu: false, message: Some(format!("model {} not downloaded", spec.display_name)), warm_ms: None, backend: String::new() },
            );
            return true;
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
        // A freshly built server still says Missing, and nothing else is sent
        // until start() returns, which can take two minutes. The dashboard read
        // "engine missing" with a Download button for that whole time, and that
        // is what sent the first AMD tester hunting through Settings on
        // 7 September 2026. Say what is actually happening.
        let mut starting_info = server.info();
        starting_info.status = EngineStatus::Starting;
        starting_info.message = Some("the speech model is loading".into());
        let _ = app.emit("lalia://engine", starting_info);
        let mut result = server.start().await;
        if result.is_err() && use_gpu {
            tracing::warn!("GPU start failed, retrying on CPU");
            // Stop the one that failed first. A warm-up failure leaves its child
            // alive, and the fallback used to reuse the same port, so its own
            // port probe answered against the corpse of the first attempt and
            // reported success. The user was told it had moved to the CPU while
            // nothing had changed at all.
            server.stop().await;
            let mut cfg = server.config().clone();
            cfg.use_gpu = false;
            cfg.backend = "cpu".to_string();
            cfg.port = self.pick_port();
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
        true
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
        // Ready and Starting both mean a process should be running right now.
        //
        // Only Ready was checked before, so an engine whose process died while
        // it was still loading the model stayed "starting" for the rest of the
        // session and nothing ever brought it back. Loading takes up to two
        // minutes on a cold machine, which is a wide window to crash in.
        //
        // Nothing is started here when there is no engine at all. That case is
        // deliberate: a missing model file used to make this fire every five
        // seconds for as long as the app stayed open.
        let dead = match local {
            Some(s) => matches!(s.info().status, EngineStatus::Ready | EngineStatus::Starting) && !s.is_alive(),
            None => false,
        };
        if dead {
            tracing::warn!("the speech engine stopped on its own, starting it again");
            self.apply(app, settings).await;
        }
    }
}
