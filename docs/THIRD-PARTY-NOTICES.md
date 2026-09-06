# Third-party software and models

This application is a clean-room implementation. It contains no code, assets or branding from Wispr Flow. The components below are used under their own licenses.

## Runtime components shipped inside the installer

| Component | Version | License | Source |
|---|---|---|---|
| whisper.cpp (`whisper-server.exe`, `whisper.dll`, `ggml*.dll`) | b4938 | MIT | https://github.com/ggml-org/whisper.cpp |
| NVIDIA CUDA runtime libraries inside the same zip (`cudart64_12.dll`, `cublas64_12.dll`, `cublasLt64_12.dll`) | CUDA 12.4 | NVIDIA CUDA Toolkit EULA (redistributable runtime components) | https://docs.nvidia.com/cuda/eula/ |
| Whisper large-v3, large-v3-turbo, medium (GGML conversions) | ggerganov/whisper.cpp on Hugging Face | MIT (OpenAI Whisper model weights) | https://huggingface.co/ggerganov/whisper.cpp |
| Silero VAD v5.1.2 (GGML conversion) | ggml-org/whisper-vad | MIT | https://huggingface.co/ggml-org/whisper-vad |

The CUDA runtime DLLs are redistributed by the whisper.cpp project inside its release archive and by this application inside its installer. The NVIDIA EULA permits redistribution of these runtime libraries with an application; the EULA text applies to them.

## Rust crates (compiled into lalia.exe)

| Crate | License |
|---|---|
| tauri, tauri-build, tauri-plugin-opener, tauri-plugin-autostart, tauri-plugin-single-instance, tauri-plugin-dialog | MIT or Apache-2.0 |
| tokio, reqwest, hyper, rustls | MIT (rustls: Apache-2.0 / ISC / MIT) |
| serde, serde_json, anyhow, thiserror, once_cell, parking_lot, crossbeam-channel, futures-util | MIT or Apache-2.0 |
| rusqlite (bundled SQLite) | MIT (SQLite: public domain) |
| cpal | Apache-2.0 |
| hound | Apache-2.0 |
| regex, fancy-regex | MIT or Apache-2.0 (fancy-regex: MIT) |
| chrono, uuid, sha2, hex, dirs | MIT or Apache-2.0 |
| keyring | MIT or Apache-2.0 |
| tracing, tracing-subscriber, tracing-appender | MIT |
| windows (windows-rs) | MIT or Apache-2.0 |
| async-trait | MIT or Apache-2.0 |

The full list with exact versions is in `src-tauri/Cargo.lock`. `cargo license` (or `cargo about`) regenerates it.

## JavaScript packages (dashboard and overlay)

| Package | License |
|---|---|
| react, react-dom | MIT |
| @tauri-apps/api, @tauri-apps/cli | MIT or Apache-2.0 |
| vite, @vitejs/plugin-react, typescript | MIT (typescript: Apache-2.0) |

## Evaluation corpus

The synthetic evaluation audio in `eval/corpus/` is generated with Microsoft Edge neural voices through the `edge-tts` package (MIT). It is used only to measure the pipeline locally and stays inside the repository.

## Fonts and icons

The UI uses the system font (Segoe UI). The application icon was drawn for Lalia (`design/make_icon.py`).
