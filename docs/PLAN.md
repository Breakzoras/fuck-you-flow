# Lalia build plan (living document)

Owner: Claude session 2026-09-04. Lu decided: build it, decide the small things myself.
Decisions taken without asking (reversible): name Lalia, Greek UI with string table, default push-to-talk Ctrl+Win, serial build.

## Architecture (final)
- Tauri 2 + Rust backend + React/TS (Vite). Two windows: `main` (dashboard, hidden at start) and `overlay` (transparent pill).
- ASR: whisper.cpp `whisper-server.exe` from release b4938 (`whisper-cublas-12.4.0-bin-x64.zip`, sha256 c1b17166e1e31a91cc8e9c1f910d3785e3ce757bb2958bf9dce13fdb4880005f), spawned as managed sidecar on 127.0.0.1, model kept warm. Default model large-v3-turbo-q5_0 pending benchmark vs medium-q5_0.
- VAD: energy gate in Rust (empty-recording rejection) + whisper-server `--vad` with ggml-silero-v5.1.2.bin.
- Hotkey: own WH_KEYBOARD_LL hook thread (windows crate). Win key Start-menu suppression by injecting a neutral key on Win release.
- Insertion: clipboard paste with save/restore (excluded from clipboard history) -> SendInput unicode typing fallback -> copy-only. Focus captured at hotkey down, restored before paste, refuse if target changed. UIA IsPassword check.
- Data: SQLite (rusqlite bundled) in %APPDATA%\Lalia, settings JSON, secrets in Credential Manager (keyring).
- Cleanup: deterministic (fillers GR/EN, self-corrections, punctuation, capitalization) -> dictionary -> snippets -> optional llama-server -> divergence check.

## Phases and status (2026-09-05 08:45 local)
- [x] P0 scaffold, deps, downloads, git
- [x] P1 hotkey + audio + overlay + tray + settings window (record/cancel loop) - hook verified, overlay untested by a person
- [x] P2 whisper-server sidecar + model manager + raw transcript history - engine ready 1.75 to 4.1 s on GPU
- [x] P3 insertion (clipboard/paste/restore, focus, fallback, paste-last) - Notepad verified by harness; other apps pending
- [x] P4 cleanup + dictionary + snippets + hints - 38 unit tests green
- [x] P5 per-app styles + suggested learning - code + tests, no live run
- [x] P6 dashboard, stats, privacy, export/delete - built, walk-through pending
- [~] P7 benchmark harness (running), eval set (18 synthetic items), installer (built, unsigned), docs (7 files)

## Open investigations
- 2 of 4 injected chords did not start recording in one e2e run; hotkey events now logged at debug.
- en-01 paste reported consumed but text not read back by the script.
- Lu's live test after the UIA timeout fix.

## Files of record
- research/ (6 agent reports + last30days) -> used for decisions, summarised in docs/RESEARCH-SUMMARY.md when all arrive
- docs/ARCHITECTURE.md, docs/THREAT-MODEL.md, docs/BENCHMARKS.md, docs/KNOWN-LIMITATIONS.md, docs/USER-GUIDE.md, docs/DEV-SETUP.md

## Gotchas learned
- `create-tauri-app -f` wipes the target directory. Never again.
- Models download in seconds from HF (xet). whisper zip ~7 MB/s.
