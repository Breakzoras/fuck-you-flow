# Lalia architecture

Lalia is a local-first voice keyboard for Windows 10 and 11. Hold a shortcut, speak Greek or English, release, and the text appears at the cursor of whatever application had focus. Nothing leaves the machine unless the user explicitly selects a cloud provider.

## Stack

| Layer | Choice | Why |
|---|---|---|
| Desktop shell | Tauri 2 (Rust backend, WebView2 UI) | Small binary, native tray and windows, typed commands between UI and backend |
| UI | React 19 + TypeScript, plain CSS | Two windows: the dashboard (`index.html`) and the floating pill (`overlay.html`) |
| Speech to text | whisper.cpp `whisper-server.exe` (release b4938, CUDA 12.4 build) run as a managed child process | Official prebuilt CUDA binaries, model stays loaded between dictations, HTTP interface, no compiler or CUDA toolkit needed on the user's PC |
| Voice activity | Energy gate in Rust plus Silero VAD inside whisper-server | Fast rejection of empty presses, fewer hallucinations on noise |
| Audio capture | cpal 0.18 (WASAPI shared mode) | Native Windows audio, device enumeration, 16 kHz mono conversion in the callback |
| Global shortcut | Own `WH_KEYBOARD_LL` hook on a dedicated thread | Only way to see key release for chords such as Ctrl+Win, and to ignore auto-repeat |
| Text insertion | Clipboard paste with delayed rendering (`WM_RENDERFORMAT`) and clipboard restore; Unicode `SendInput` typing as fallback; copy-only as last resort | Works across Win32, Chromium, Electron and UWP apps; restore happens after the target really read the clipboard |
| Storage | SQLite (rusqlite, bundled, WAL) in `%APPDATA%\Lalia`, settings as JSON, secrets in Windows Credential Manager (keyring) | Local, inspectable, backed up before every migration |
| Logs | tracing to daily files in `%LOCALAPPDATA%\Lalia\logs`, transcript text redacted by default | Diagnostics without leaking what the user said |

## Processes

```
lalia.exe (Tauri)
  main thread          Tauri event loop, windows, tray
  lalia-keyboard-hook  WH_KEYBOARD_LL + message loop, sends HotkeyEvent over a channel
  lalia-audio          owns the cpal stream, fills the ring buffer, level meter
  lalia-clipboard      hidden owner window, serves WM_RENDERFORMAT, restores clipboard
  tokio runtime        pipeline task, engine manager, downloads, watchdog
whisper-server.exe     child process in a job object (dies with the app), 127.0.0.1:<random port>
```

## The dictation pipeline

```
IDLE
 -> hotkey down          capture foreground window, process, title, elevation
 -> classify target      chat / email / document / code / terminal / sensitive
 -> overlay Starting     within a few ms, before the microphone is touched
 -> microphone           warm stream (default) or opened now; pre-roll buffer copied in
 -> UIA check            password field? then refuse and discard audio
 -> RECORDING            level meter events at 30 Hz, Escape cancels, hard cap 10 min
 -> hotkey up            (or second press in hands-free mode)
 -> energy gate          < 200 ms of speech -> "no speech", under 500 ms after release
 -> recovery copy        %LOCALAPPDATA%\Lalia\recovery\last.wav until the text is inserted
 -> engine ready?        waits up to 8 s for a starting engine, else Failed with retry
 -> whisper-server       HTTP multipart, language el / en / auto, dictionary terms as prompt
 -> hallucination guard  known silence phrases, empty text -> "no speech"
 -> cleanup              deterministic rules -> dictionary -> snippets (Rust, microseconds)
 -> insertion            restore focus if it drifted, refuse elevated targets, paste / type / copy
 -> history + stats      raw, cleaned, final text, app, timings, rules applied
 -> SUCCESS -> IDLE      overlay shows a preview, hides after 1.4 s
```

Every step has a timeout or a cancel path: the engine call is capped at 90 s, clipboard operations at 2 s each, and a stuck key cannot record past the configured maximum.

## Services and interfaces (Rust modules)

| Interface from the brief | Module | Notes |
|---|---|---|
| AudioCaptureService | `audio.rs` | `AudioCapture` handle, ring buffer, resampler, WAV encoder |
| VoiceActivityService | `audio.rs::analyze_speech` + whisper-server `--vad` | Energy gate first, neural VAD second |
| TranscriptionProvider | `asr/mod.rs` trait, `asr/whisper_server.rs`, `asr/openai_compat.rs` | New engines implement one trait |
| CleanupProvider | `cleanup/` (`deterministic.rs`, `divergence.rs`) | LLM cleanup is a planned provider behind the same `CleanupOutcome` |
| DictionaryEngine | `cleanup/dictionary.rs` | Unicode word boundaries, case preservation, exceptions, hint terms |
| SnippetEngine | `cleanup/snippets.rs` | Whole-phrase triggers |
| ContextService | `context.rs` | Process classification, UIA focus inspection, style overrides |
| InsertionService | `insertion.rs` | Clipboard thread, SendInput typing, focus restore, elevation check |
| HistoryRepository | `db.rs` | History, dictionary, snippets, suggestions, styles, daily stats |
| PersonalizationService | `learning.rs` | Edit classification, suggestion after two identical corrections |
| ModelManager | `models.rs` | Catalog with SHA-256, download with progress, runtime installer |
| ShortcutManager | `hotkey.rs` | Chord parsing, hook, Start-menu masking, shortcut recorder |
| OverlayStateController | `overlay.rs` + `pipeline.rs` | State events to the pill, placement per monitor work area |

The UI never touches audio, the clipboard or model binaries. It calls typed commands in `commands.rs` and listens to events (`lalia://overlay`, `lalia://level`, `lalia://engine`, `lalia://download`, `lalia://history-changed`).

## Data locations

| What | Where |
|---|---|
| Settings | `%APPDATA%\Lalia\settings.json` |
| Database | `%APPDATA%\Lalia\lalia.db` (backup copies `lalia.pre-vN.bak` before migrations) |
| Models | `%LOCALAPPDATA%\Lalia\models\` (dev checkout falls back to `<repo>\models\`) |
| Engine runtime | `%LOCALAPPDATA%\Lalia\runtime\whisper\` (dev checkout: `<repo>\vendor\whisper\Release\`) |
| Logs | `%LOCALAPPDATA%\Lalia\logs\lalia.log.<date>` |
| Recovery audio | `%LOCALAPPDATA%\Lalia\recovery\last.wav`, deleted after a successful insertion |
| Optional kept audio | `%LOCALAPPDATA%\Lalia\audio\<id>.wav` only when "keep audio" is on |
| Cloud API key | Windows Credential Manager, service `Lalia` |

## Decisions worth knowing

- **Clipboard over UI Automation for insertion.** UIA `ValuePattern.SetValue` replaces the whole field and `TextPattern` cannot insert at the caret. Wispr Flow itself pastes through the clipboard (confirmed by users who inspected the app bundle). Lalia uses UIA only to detect password fields and, when Context Awareness is on, to read a little text near the caret.
- **Delayed clipboard rendering.** Lalia publishes an empty promise for `CF_UNICODETEXT`, sends Ctrl+V, and hands the text over only when the target asks for it. The previous clipboard text is restored once render traffic goes quiet. This is the fix for the "old clipboard got pasted" bug that fixed-delay implementations suffer from.
- **Clipboard history opt-out.** Every dictation paste carries the three formats that keep it out of Win+V history and cloud clipboard sync.
- **Own keyboard hook.** `RegisterHotKey` never reports key release, cannot express Ctrl+Win, and steals the chord from every other app. The hook ignores only key events Lalia itself injected (a `dwExtraInfo` signature), so AutoHotkey and remote desktop keys still work.
- **Start menu masking.** When Win is part of the chord, a neutral key (VK 0xE8) is injected before the Win key-up reaches Windows, so releasing the chord does not open Start.
- **Plain json responses from whisper-server.** With Silero VAD enabled, a request whose audio contains no speech makes the server resolve an invalid language id while building `verbose_json`, and the process dies. `json` avoids that path; language is detected by script in the cleanup stage instead.
- **Warm-up without VAD.** The warm-up request (0.5 s of silence) is sent with `vad=false` for the same reason.
- **Job object for the engine.** whisper-server is assigned to a job with kill-on-close so a crash of Lalia can never leave a 1 GB model resident on the GPU.
- **large-v3 over large-v3-turbo as default.** Published measurements put turbo second worst of 38 languages for Greek (FLEURS 10.9 to 13.0 WER); full large-v3 in q5_0 loads in under 2 s on the RTX 3070 and transcribes 5 s of Greek in about 0.6 to 1.2 s. See `docs/BENCHMARKS.md`.
