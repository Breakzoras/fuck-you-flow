//! Model Manager and runtime installer: catalog of supported speech models,
//! download with progress, SHA-256 verification, removal. Also fetches the
//! whisper.cpp CUDA runtime on first run (the installer stays small).

use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    pub id: String,
    pub file_name: String,
    pub display_name: String,
    pub url: String,
    pub sha256: String,
    pub size_bytes: u64,
    /// Approximate GPU memory needed with CUDA (MB).
    pub vram_mb: u32,
    /// Approximate RAM needed on CPU (MB).
    pub ram_mb: u32,
    pub languages: String,
    pub notes: String,
    pub recommended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    #[serde(flatten)]
    pub spec: ModelSpec,
    pub installed: bool,
    pub verified: bool,
    pub path: Option<String>,
}

const HF: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/";

pub fn catalog() -> Vec<ModelSpec> {
    vec![
        ModelSpec {
            id: "large-v3-q5_0".into(),
            file_name: "ggml-large-v3-q5_0.bin".into(),
            display_name: "Whisper large-v3 (q5_0)".into(),
            url: format!("{HF}ggml-large-v3-q5_0.bin"),
            sha256: "d75795ecff3f83b5faa89d1900604ad8c780abd5739fae406de19f23ecd98ad1".into(),
            size_bytes: 1_081_140_203,
            vram_mb: 2600,
            ram_mb: 2400,
            languages: "Ελληνικά, Αγγλικά και 97 ακόμα".into(),
            notes: "Η καλύτερη ακρίβεια στα ελληνικά (WER 11 στο FLEURS). Προεπιλογή.".into(),
            recommended: true,
        },
        ModelSpec {
            id: "large-v3-turbo-q5_0".into(),
            file_name: "ggml-large-v3-turbo-q5_0.bin".into(),
            display_name: "Whisper large-v3-turbo (q5_0)".into(),
            url: format!("{HF}ggml-large-v3-turbo-q5_0.bin"),
            sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2".into(),
            size_bytes: 574_041_195,
            vram_mb: 1600,
            ram_mb: 1400,
            languages: "Ελληνικά, Αγγλικά και 97 ακόμα".into(),
            notes: "Πιο γρήγορο, αλλά μετρημένα χειρότερο στα ελληνικά. Εφεδρικό για λίγη μνήμη.".into(),
            recommended: false,
        },
        ModelSpec {
            id: "large-v3-turbo-q8_0".into(),
            file_name: "ggml-large-v3-turbo-q8_0.bin".into(),
            display_name: "Whisper large-v3-turbo (q8_0)".into(),
            url: format!("{HF}ggml-large-v3-turbo-q8_0.bin"),
            sha256: "317eb69c11673c9de1e1f0d459b253999804ec71ac4c23c17ecf5fbe24e259a1".into(),
            size_bytes: 874_188_075,
            vram_mb: 2000,
            ram_mb: 1800,
            languages: "Ελληνικά, Αγγλικά και 97 ακόμα".into(),
            notes: "Turbo με λιγότερη συμπίεση.".into(),
            recommended: false,
        },
        ModelSpec {
            id: "medium-q5_0".into(),
            file_name: "ggml-medium-q5_0.bin".into(),
            display_name: "Whisper medium (q5_0)".into(),
            url: format!("{HF}ggml-medium-q5_0.bin"),
            sha256: "19fea4b380c3a618ec4723c3eef2eb785ffba0d0538cf43f8f235e7b3b34220f".into(),
            size_bytes: 539_212_467,
            vram_mb: 1500,
            ram_mb: 1300,
            languages: "Ελληνικά, Αγγλικά και 97 ακόμα".into(),
            notes: "Μικρότερο, για αδύναμα μηχανήματα. Χειρότερα ελληνικά.".into(),
            recommended: false,
        },
    ]
}

pub fn vad_spec() -> ModelSpec {
    ModelSpec {
        id: "silero-vad".into(),
        file_name: "ggml-silero-v5.1.2.bin".into(),
        display_name: "Silero VAD".into(),
        url: "https://huggingface.co/ggml-org/whisper-vad/resolve/main/ggml-silero-v5.1.2.bin".into(),
        sha256: "29940d98d42b91fbd05ce489f3ecf7c72f0a42f027e4875919a28fb4c04ea2cf".into(),
        size_bytes: 885_098,
        vram_mb: 0,
        ram_mb: 10,
        languages: "όλες".into(),
        notes: "Ανιχνευτής ομιλίας, κόβει τη σιωπή και τους θορύβους.".into(),
        recommended: true,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSpec {
    pub url: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub version: String,
}

pub fn runtime_spec() -> RuntimeSpec {
    RuntimeSpec {
        url: "https://github.com/ggml-org/whisper.cpp/releases/download/b4938/whisper-cublas-12.4.0-bin-x64.zip".into(),
        sha256: "c1b17166e1e31a91cc8e9c1f910d3785e3ce757bb2958bf9dce13fdb4880005f".into(),
        size_bytes: 671_045_732,
        version: "b4938 (CUDA 12.4)".into(),
    }
}

pub fn model_path(spec: &ModelSpec) -> PathBuf {
    let app = crate::paths::models_dir().join(&spec.file_name);
    if app.exists() {
        return app;
    }
    // shipped inside the installer
    if let Some(b) = crate::paths::bundled_dir() {
        let p = b.join("models").join(&spec.file_name);
        if p.exists() {
            return p;
        }
    }
    // dev checkout fallback: <repo>/models/<file>  (dev_vendor_dir is <repo>/vendor/whisper)
    let dev = crate::asr::whisper_server::dev_vendor_dir().parent().and_then(|p| p.parent()).map(|p| p.join("models").join(&spec.file_name));
    match dev {
        Some(p) if p.exists() => p,
        _ => app,
    }
}

pub fn find_model(id: &str) -> Option<(ModelSpec, PathBuf)> {
    catalog().into_iter().find(|m| m.id == id).map(|m| {
        let p = model_path(&m);
        (m, p)
    })
}

pub fn vad_path() -> Option<PathBuf> {
    let p = model_path(&vad_spec());
    if p.exists() {
        Some(p)
    } else {
        None
    }
}

pub fn list_status() -> Vec<ModelStatus> {
    catalog()
        .into_iter()
        .map(|spec| {
            let p = model_path(&spec);
            let installed = p.exists() && std::fs::metadata(&p).map(|m| m.len() == spec.size_bytes).unwrap_or(false);
            ModelStatus { path: if p.exists() { Some(p.display().to_string()) } else { None }, installed, verified: installed && marker_ok(&p, &spec.sha256), spec }
        })
        .collect()
}

/// A tiny side file records the verified checksum so we do not hash a gigabyte
/// on every startup.
fn marker_path(p: &Path) -> PathBuf {
    p.with_extension("sha256")
}

fn marker_ok(p: &Path, expected: &str) -> bool {
    std::fs::read_to_string(marker_path(p)).map(|s| s.trim() == expected).unwrap_or(false)
}

pub fn sha256_file(p: &Path) -> anyhow::Result<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(p)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn verify(p: &Path, expected: &str) -> anyhow::Result<bool> {
    let actual = sha256_file(p)?;
    let ok = actual == expected;
    if ok {
        let _ = std::fs::write(marker_path(p), expected);
    }
    Ok(ok)
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub id: String,
    pub received: u64,
    pub total: u64,
    pub phase: String, // downloading | verifying | done | error
    pub message: Option<String>,
}

/// Download `url` to `dest` (through a .part file), verify the checksum, emit
/// progress on the "lalia://download" event.
pub async fn download_verified(app: &tauri::AppHandle, id: &str, url: &str, dest: &Path, expected_sha: &str, total_hint: u64) -> anyhow::Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let part = dest.with_extension("part");
    let client = reqwest::Client::builder().user_agent("Lalia/0.1").build()?;
    let resp = client.get(url).send().await?.error_for_status()?;
    let total = resp.content_length().unwrap_or(total_hint);
    let mut stream = resp.bytes_stream();
    let mut file = tokio::fs::File::create(&part).await?;
    let mut received = 0u64;
    let mut last_emit = std::time::Instant::now();
    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        received += chunk.len() as u64;
        if last_emit.elapsed().as_millis() > 200 {
            let _ = app.emit("lalia://download", DownloadProgress { id: id.into(), received, total, phase: "downloading".into(), message: None });
            last_emit = std::time::Instant::now();
        }
    }
    file.flush().await?;
    drop(file);
    let _ = app.emit("lalia://download", DownloadProgress { id: id.into(), received, total, phase: "verifying".into(), message: None });
    let part2 = part.clone();
    let expected = expected_sha.to_string();
    let ok = tokio::task::spawn_blocking(move || verify(&part2, &expected)).await??;
    if !ok {
        let _ = std::fs::remove_file(&part);
        let _ = app.emit("lalia://download", DownloadProgress { id: id.into(), received, total, phase: "error".into(), message: Some("checksum mismatch".into()) });
        anyhow::bail!("checksum mismatch for {id}");
    }
    std::fs::rename(&part, dest)?;
    let _ = std::fs::write(marker_path(dest), expected_sha);
    let _ = app.emit("lalia://download", DownloadProgress { id: id.into(), received, total, phase: "done".into(), message: None });
    Ok(())
}

pub async fn download_model(app: &tauri::AppHandle, id: &str) -> anyhow::Result<()> {
    let spec = if id == "silero-vad" { vad_spec() } else { catalog().into_iter().find(|m| m.id == id).ok_or_else(|| anyhow::anyhow!("unknown model {id}"))? };
    let dest = crate::paths::models_dir().join(&spec.file_name);
    download_verified(app, id, &spec.url, &dest, &spec.sha256, spec.size_bytes).await
}

pub fn remove_model(id: &str) -> anyhow::Result<()> {
    let spec = catalog().into_iter().find(|m| m.id == id).ok_or_else(|| anyhow::anyhow!("unknown model {id}"))?;
    let p = crate::paths::models_dir().join(&spec.file_name);
    if p.exists() {
        std::fs::remove_file(&p)?;
    }
    let _ = std::fs::remove_file(marker_path(&p));
    Ok(())
}

/// Files we keep from the runtime zip (everything else is examples and tests).
const RUNTIME_KEEP: &[&str] = &[
    "whisper-server.exe",
    "whisper.dll",
    "ggml.dll",
    "ggml-base.dll",
    "ggml-cpu.dll",
    "ggml-cuda.dll",
    "cublas64_12.dll",
    "cublasLt64_12.dll",
    "cudart64_12.dll",
    "nvblas64_12.dll",
    "nvrtc64_120_0.dll",
    "nvrtc-builtins64_124.dll",
];

pub fn runtime_installed() -> bool {
    crate::asr::whisper_server::find_runtime_exe().is_some()
}

/// Download the whisper.cpp release zip, verify it, extract the server and its
/// DLLs into the runtime directory. Uses the tar.exe that ships with Windows 10+.
pub async fn install_runtime(app: &tauri::AppHandle) -> anyhow::Result<()> {
    let spec = runtime_spec();
    let dir = crate::paths::runtime_dir();
    std::fs::create_dir_all(&dir)?;
    let zip = dir.join("whisper-runtime.zip");
    if !(zip.exists() && marker_ok(&zip, &spec.sha256)) {
        download_verified(app, "whisper-runtime", &spec.url, &zip, &spec.sha256, spec.size_bytes).await?;
    }
    let target = crate::paths::whisper_runtime_dir();
    std::fs::create_dir_all(&target)?;
    let _ = app.emit("lalia://download", DownloadProgress { id: "whisper-runtime".into(), received: spec.size_bytes, total: spec.size_bytes, phase: "extracting".into(), message: None });
    let zip2 = zip.clone();
    let target2 = target.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let mut cmd = std::process::Command::new("tar");
        cmd.arg("-xf").arg(&zip2).arg("-C").arg(&target2).arg("--strip-components=1");
        // tar wants member names as they appear in the archive
        for f in RUNTIME_KEEP {
            cmd.arg(format!("Release/{f}"));
        }
        // ggml-cpu-*.dll variants (CPU feature levels) are needed by ggml-cpu dispatch
        let out = cmd.output()?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            // tar errors on a missing member; fall back to extracting all of Release/
            tracing::warn!("selective extract failed ({err}); extracting whole Release folder");
            let out2 = std::process::Command::new("tar").arg("-xf").arg(&zip2).arg("-C").arg(&target2).arg("--strip-components=1").arg("Release").output()?;
            if !out2.status.success() {
                anyhow::bail!("tar failed: {}", String::from_utf8_lossy(&out2.stderr));
            }
        }
        // Always also extract the CPU variant DLLs (names vary by release).
        let _ = std::process::Command::new("tar").arg("-xf").arg(&zip2).arg("-C").arg(&target2).arg("--strip-components=1").arg("--wildcards").arg("Release/ggml-cpu-*.dll").output();
        Ok(())
    })
    .await??;
    let _ = std::fs::remove_file(&zip);
    let _ = app.emit("lalia://download", DownloadProgress { id: "whisper-runtime".into(), received: spec.size_bytes, total: spec.size_bytes, phase: "done".into(), message: None });
    Ok(())
}

/// Is a CUDA capable NVIDIA driver present? (nvcuda.dll ships with the driver.)
/// The model to start with on a machine that has never run the app: the most
/// accurate one when an NVIDIA driver is present, the fast one otherwise, and
/// always one that is actually on disk. Falls back to the shipped default.
pub fn preferred_installed_model(default_id: &str) -> Option<String> {
    // large-v3 needs about 2.6 GB of graphics memory; below that, or on the
    // CPU, the turbo model is the one that stays usable.
    let hw = crate::hw::detect();
    let strong_gpu = hw.best_gpu().map(|g| g.vram_mb >= 3000).unwrap_or(false);
    let order: &[&str] = if strong_gpu {
        &["large-v3-q5_0", "large-v3-turbo-q5_0", "medium-q5_0"]
    } else {
        &["large-v3-turbo-q5_0", "medium-q5_0", "large-v3-q5_0"]
    };
    if let Some((_, p)) = find_model(default_id) {
        if p.exists() {
            return None; // the configured model is there, keep it
        }
    }
    order.iter().find(|id| find_model(id).map(|(_, p)| p.exists()).unwrap_or(false)).map(|id| id.to_string())
}

pub fn cuda_driver_present() -> bool {
    let sys = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    Path::new(&sys).join("System32").join("nvcuda.dll").exists()
}
