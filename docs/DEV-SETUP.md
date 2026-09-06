# Developer setup

## Prerequisites (Windows 10/11)

- Rust stable with the MSVC toolchain (`rustup default stable-x86_64-pc-windows-msvc`), tested with 1.96
- Visual Studio Build Tools with the C++ workload (for the MSVC linker)
- Node 24 and pnpm 10 (`corepack enable` or `npm i -g pnpm`)
- WebView2 runtime (present on any Windows with Edge)
- Python 3.12+ only for the evaluation scripts (`pip install edge-tts jiwer requests`)
- ffmpeg on PATH only for building the synthetic corpus
- An NVIDIA driver that supports CUDA 12.4 (driver 550 or newer). No CUDA toolkit is needed: the engine ships its own runtime DLLs.

## First build

```bash
pnpm install
pnpm tauri dev
```

`pnpm tauri dev` starts Vite on port 1420 and runs the debug binary. The first Rust build takes a few minutes (about 500 crates), later builds take 20 to 40 seconds.

### Engine and models in a dev checkout

The app looks for the engine and models in `%LOCALAPPDATA%\Lalia\...` first and falls back to the repository:

- `vendor/whisper/Release/whisper-server.exe` (unzip `whisper-cublas-12.4.0-bin-x64.zip` from the whisper.cpp release b4938 into `vendor/whisper/`)
- `models/ggml-large-v3-q5_0.bin`, `models/ggml-silero-v5.1.2.bin` and any other model from the catalog in `src-tauri/src/models.rs`

Both folders are git-ignored. Alternatively install them from inside the app (Home page, "First-run setup"), which downloads into `%LOCALAPPDATA%\Lalia` with checksum verification.

## Tests

```bash
cd src-tauri
cargo test                       # 38 unit tests: hotkey parsing, audio gate, resampler, cleanup, dictionary, snippets, learning, db, settings
cargo test live_inference -- --ignored --nocapture   # needs a whisper-server started by hand on port 47555
pnpm exec tsc --noEmit           # type-check the UI
```

End-to-end (debug build only): `pnpm tauri dev` in one terminal, then

```bash
python eval/e2e_notepad.py el-05 en-01
```

The script starts Notepad, points the app at a corpus WAV through `%LOCALAPPDATA%\Lalia\fake_mic.txt`, injects the Ctrl+Win chord with `keybd_event`, and reads the Notepad text back. Everything except the microphone is real. Do not run it while a person is using the keyboard.

Benchmarks:

```bash
python eval/make_corpus.py       # once, builds eval/corpus with Edge TTS voices
python eval/bench.py --models large-v3-q5_0,large-v3-turbo-q5_0,medium-q5_0
```

Results land in `eval/bench-results.json` and `eval/bench-summary.md`.

## Useful scripts

- `scripts/dev-restart.sh [seconds]`: kills the app, Vite and any orphan engine, relaunches `pnpm tauri dev`, prints the newest log lines.
- `design/make_icon.py`: regenerates the icon source; `pnpm tauri icon design/icon-src.png` produces all sizes.

## Logs and data while developing

- Logs: `%LOCALAPPDATA%\Lalia\logs\lalia.log.<date>` (set `LALIA_LOG=debug` for engine chatter)
- Settings: `%APPDATA%\Lalia\settings.json`
- Database: `%APPDATA%\Lalia\lalia.db`

## Release build and installer

```bash
pnpm tauri build
```

For a runnable binary without the installer:

```bash
pnpm tauri build --no-bundle
```

Always build through the Tauri CLI. A plain `cargo build --release` leaves out the `custom-protocol` feature, and Tauri treats that as a development build: the dashboard window then tries to load `http://localhost:1420` (the Vite dev server) and shows a blank page or "can't reach this page" when no dev server is running. Both URLs are present as strings in every binary, so the only reliable check is opening the dashboard (`scripts\window-shot.ps1` photographs it).

To put a new build live, run `bash scripts/swap-release.sh`: it refuses while a recording is in progress (a restart mid-dictation loses the audio), then restarts the app and prints the startup lines of the log.

If the app is running while you rebuild, the linker cannot replace `lalia.exe`. Rename the running file first (`mv lalia.exe lalia-running.exe`); Windows allows renaming a running executable, and the build writes a fresh `lalia.exe` next to it.

Produces `src-tauri/target/release/bundle/nsis/Fuck You Flow_0.1.0_x64-setup.exe` (per-user install, no administrator rights, Greek and English installer language).

The installer carries everything the app needs, so a fresh machine works offline right after setup: the whisper.cpp engine built with the Vulkan backend (`scripts/build-vulkan.ps1`, from the same tag as the official release; it runs on AMD, Intel and NVIDIA cards and doubles as the CPU build with `--no-gpu`), the `large-v3-q5_0` and `large-v3-turbo-q5_0` models, and the Silero VAD model. The official CUDA build stays in `vendor/whisper/Release` for comparison; on the RTX 3070 Vulkan measured 593 ms against CUDA's 663 ms for the same phrase and model, so shipping CUDA (1.1 GB) buys nothing.

`scripts/stage-bundle.py` hard-links the files into `src-tauri/bundled/`, which `bundle.resources` ships as `bundled/**/*`. Clear `src-tauri/target/release/bundled` before a rebuild after changing the staged files, or the build script stops with "Cannot create a file when that file already exists". They are declared in `bundle.resources` and land in `<install dir>/resources/bundled/{engine,models}`; `paths::set_bundled_dir` records that folder at startup and `find_runtime_exe` plus `model_path` consult it. Payload before compression is about 2.6 GB.

On the first start of a machine, `hw::detect` reads the graphics adapters through DXGI, the core count and the memory (logged as `machine: ...`). `app::build` then sets the thread count to three quarters of the logical cores (the measured optimum on an 8-core Ryzen) and picks `large-v3-q5_0` for a card with at least 3 GB of memory, `large-v3-turbo-q5_0` otherwise; `engine::choose_backend` prefers Vulkan on any real card, then CUDA, then the CPU, and `asr.backend` in the settings can pin one. The Speech models tab shows what was detected.

`webviewInstallMode` is `embedBootstrapper`, so a PC without the WebView2 runtime gets it during setup.

The download path in `models.rs` is still there and is what the Models page in Settings uses to fetch a model that was not shipped.

## Code map

See `docs/ARCHITECTURE.md`. Entry points: `src-tauri/src/lib.rs` (Tauri builder), `src-tauri/src/app.rs` (state, tray, startup), `src-tauri/src/pipeline.rs` (state machine), `src/App.tsx` (dashboard), `src/overlay/main.tsx` (pill).
