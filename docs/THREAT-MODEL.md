# Lalia privacy and threat model

Scope: a single-user desktop app that records the microphone on demand, converts speech to text locally, and pastes text into the focused application. No account, no telemetry, no cloud by default.

## Assets

1. Microphone audio (the most sensitive asset: it can contain anything said near the PC).
2. Transcripts (raw, cleaned, final) and the history database.
3. The Dictionary and snippets (they reveal names, projects, email signatures).
4. The user's clipboard contents before a paste.
5. Text near the caret when Context Awareness is enabled.
6. The optional cloud API key.
7. Model files and the engine binary (integrity matters: a tampered engine sees all audio).

## Data flow and controls

| Flow | Default | Control |
|---|---|---|
| Microphone -> memory | Stream open while the app runs ("keep warm"), samples live only in a 350 ms ring buffer until a hotkey press | Setting "keep the microphone open" can be turned off; the buffer is cleared after every dictation |
| Audio -> whisper-server | Local HTTP on 127.0.0.1, random port, multipart WAV, no proxy | Loopback only; a system proxy is bypassed explicitly |
| Audio -> disk | Never, except `recovery\last.wav` between release and successful insertion (deleted on success) | "Keep audio with history" is off by default and per dictation when on |
| Transcript -> history DB | Stored locally when "keep history" is on | Retention in days, per-entry delete, delete all, export all |
| Transcript -> logs | Redacted to a length marker | "Redact transcript text from logs" can be disabled for debugging |
| Clipboard | Previous text (`CF_UNICODETEXT` only) held in memory for at most a few seconds, then restored | If the target never reads the paste, Lalia leaves the transcript on the clipboard and says so |
| Context near caret | Not read | Off by default; never read from password fields or sensitive apps; discarded after the dictation; flagged in history (`ctx`) |
| Cloud provider | Disabled | If enabled: audio goes to the configured base URL over HTTPS; key in Credential Manager; the overlay shows "Offline" when unreachable |
| Learning | Local edit pairs only | Pause, delete learning data, suggestions are inspectable and reversible |

## Threats and mitigations

| Threat | Mitigation |
|---|---|
| Dictation into a password field | UIA `IsPassword` check right after the hotkey press: audio discarded, overlay shows "sensitive field" |
| Dictation into banking or password manager apps | Process and title classification (`context.rs::SENSITIVE_PROCESSES`, browser titles with "bank", "login"): refused |
| Text pasted into the wrong window because focus moved | Target window captured at hotkey down; if the foreground changed and cannot be restored, the text goes to the clipboard and the overlay says "target changed" |
| Elevated target application (UIPI blocks SendInput and the paste keystroke silently) | Token elevation check; text placed on the clipboard with a clear message |
| Clipboard manager steals the delayed-render promise | Opt-out formats set on every paste; renders that happen before the paste keystroke are ignored when deciding whether the target consumed the text |
| Malicious or corrupted model file | SHA-256 verified after download against values pinned in `models.rs`; `.sha256` marker stored beside the file; "Verify" button in Model Manager |
| Tampered engine binary | Runtime zip SHA-256 pinned (`c1b17166...`); download over HTTPS from the whisper.cpp GitHub release |
| Engine process outliving the app and holding audio | Job object with kill-on-close; child stdout and stderr captured, never a console window |
| Key logging concern (the app installs a keyboard hook) | The hook only tracks which keys are down to detect chords; it never stores or logs key codes beyond the chord state; Escape is swallowed only while recording; the shortcut recorder reports keys only while the settings dialog asked for it |
| Injected key events (AutoHotkey, RDP) confusing the hook | Only Lalia's own injections (signature in `dwExtraInfo`) are ignored |
| API key leakage | Stored in Credential Manager via `keyring`; never in settings.json, never logged; sent only as a bearer header to the user-configured base URL |
| Local database read by other users of the PC | Files live under the user's profile with default ACLs; no encryption at rest (documented limitation) |
| Update mechanism abuse | No auto-update in v0.1; the NSIS installer is unsigned (documented); a future updater must use Tauri's signed update manifests |
| Overlay covering a UAC prompt | UAC runs on the secure desktop where no user-mode overlay can appear |

## What Lalia never does

- Never uploads audio, transcripts or the dictionary anywhere by default.
- Never reads the screen or takes screenshots.
- Never logs raw transcript text unless the user disables redaction.
- Never keeps the raw recording after a successful insertion unless "keep audio" is on.
- Never trains a model on the user's data; suggested rules are plain replacements the user can see and delete.

## Known gaps (v0.1)

- No encryption at rest for `lalia.db` and `settings.json`.
- Unsigned installer, so SmartScreen warns on first run.
- The "sensitive application" list is a heuristic; a user-editable block list is on the backlog (per-application styles today only change formatting).
- Windows shows the microphone indicator permanently while "keep the microphone open" is on; this is the honest cost of never clipping the first syllable.
