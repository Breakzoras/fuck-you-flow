# Known limitations (v0.1.0, 2026-09-05)

Honest list. Items marked "untested" have code but no verified run on the target PC yet.

## Verified on the target PC (Ryzen 7 5800X3D, RTX 3070, Windows 10 22H2)

- Hotkey hook, overlay, engine start on CUDA, Greek and English transcription, deterministic cleanup, clipboard paste into Notepad, history and statistics: verified with the end-to-end harness (`eval/e2e_notepad.py`) and unit tests (38 passing).
- Engine warm start on the GPU: 1.75 to 4.1 s from process spawn to first answer. Greek 5 s sentence: 0.6 to 1.2 s inference, 1.4 s from key release to text on screen including the paste.

## Not yet verified end to end

- Human voice through the real microphone from start to finish (the harness replaces the microphone with a WAV file; the microphone stream itself opens and delivers audio, and the level meter works).
- Word, Chrome text fields, Gmail, Google Docs, Telegram, Slack, Teams, Notion, VS Code, Cursor, ChatGPT and Claude inputs: the clipboard method is the same one Wispr Flow uses in these apps, but each one still needs a manual pass. Electron apps answered UI Automation slowly in one run; the check is now capped at 400 ms.
- Hands-free mode, tap-to-toggle, Escape cancel and paste-last: implemented, exercised by unit tests of the chord logic; a manual pass is still pending.
- Microphone unplugged mid-session: the watchdog re-opens the stream every 5 s when it goes silent; device-change notifications from Windows are not wired.
- Sleep and resume: the keyboard hook and the engine child survive in principle (job object, watchdog); no sleep cycle was run.
- CPU fallback: the retry path exists (`--no-gpu`) and was triggered once while diagnosing a GPU start failure; latency on CPU was not measured.
- Elevated target applications: the elevation check exists; no elevated app was tried.
- Multiple monitors and DPI changes: placement uses the monitor of the target window; only one monitor was available.

## By design in this version

- Insertion goes through the clipboard (Ctrl+V) with the previous text restored afterwards; only plain text is restored (images, files and rich formats on the clipboard are lost when a dictation lands). The overlay says "Στο πρόχειρο" when the app never read the paste.
- The Windows microphone indicator stays on while "keep the microphone open" is on.
- Releasing Ctrl+Win: a neutral key is injected so Start does not open. If Start still opens on some keyboards, use Right Ctrl as the shortcut.
- The AI cleanup stage (local language model) is designed but disabled; deterministic rules, Dictionary and snippets do the cleanup. Numbers said in words stay in words unless Whisper writes digits.
- History search is a plain substring search.
- The installer is unsigned: SmartScreen shows a warning on first launch. No auto-update.
- Cloud provider (OpenAI-compatible) is implemented but untested against a live endpoint.
- The evaluation corpus is synthetic (Edge neural voices). Real Greek accuracy on the user's own voice will differ; add private recordings under `eval/private/` (git-ignored) to measure it.

## Open bugs seen during development

- In one automated run, two of four injected Ctrl+Win chords did not start a recording while the other two worked; the pipeline now logs every hotkey event so the next run can show why (suspected: the synthetic focus change in the test script, not the hook).
- In the same run, one paste was reported as consumed by the target but the test could not read it back from Notepad; under investigation.

## Incident 2026-09-06: empty database after a reboot

After a PC restart the dashboard showed no history and no dictionary while every row was still on disk. The database had lived entirely in the SQLite write-ahead log since its creation (the main file was one page; the WAL was 1.5 MB) and the app came up reading an empty view. During the investigation the WAL on disk was found replaced by a 127 KB file containing only a fresh schema, dated the previous day at 12:38; where it came from was never established (the app never copies journal files). The main file, once checkpointed, held all 57 history rows and 30 dictionary rules and was restored with `VACUUM INTO`; the original files are kept in `%APPDATA%\Laliaackup-2026-09-06\`.

Hardening shipped the same morning: the database now uses a rollback journal (`journal_mode=DELETE`, `synchronous=FULL`) so the main file is the only source of truth; a copy of it is written to `%APPDATA%\Laliaackup\lalia-YYYY-MM-DD.db` at the first start of each day and kept for 14 days; the log records the path and the row counts at every open. Separately, no key press reached the app after that reboot: Windows removes a low-level keyboard hook without notice when its callback once exceeds 300 ms, which happens with cold pages right after boot; the hook is now re-registered every 30 seconds.
