# Privacy and data-leak audit, Fuck You Flow (lalia)

Date: 2026-09-07
Scope: `src-tauri/src/`, `src/`, `vendor/whisper.cpp-src/examples/server/server.cpp`, git tree and history, built release binary.
Method: static reading of the code. The application was never run and nothing was built or modified.

Every finding is marked CONFIRMED (the code path was read end to end) or SUSPECTED (the code strongly implies it, no live proof).

Counts: 0 CRITICAL, 3 HIGH, 6 MEDIUM, 14 LOW. 23 findings in total.

---

## Top 5 things to fix before more people install this

| # | Finding | Severity | Where |
|---|---------|----------|-------|
| 1 | Daily database backups keep 14 days of full transcripts and survive "Delete everything" | HIGH | `src-tauri/src/db.rs:247` |
| 2 | The shipped release binary embeds the developer's Windows username | HIGH | `src-tauri/target/release/lalia.exe` |
| 3 | The rolling text log has no size or age limit and is never cleared by "Delete everything" | HIGH | `src-tauri/src/logging.rs:15` |
| 4 | Raw audio of the last utterance stays on disk after any insertion that did not succeed | MEDIUM | `src-tauri/src/pipeline.rs:571` |
| 5 | Dictated text is left on the clipboard whenever the target does not accept the paste | MEDIUM | `src-tauri/src/insertion.rs:668` |

---

## 1. Logging

### Retention: implemented versus documented

| Store | Path | Implemented retention | Documented | Cleared by "Delete everything"? |
|---|---|---|---|---|
| Event journal | `%LOCALAPPDATA%\Lalia\logs\events-YYYY-MM-DD.jsonl` | 7 files, pruned when a new day opens (`journal.rs:104`) | Yes, `journal.rs:9` says "seven days kept" | No (`commands.rs:417`) |
| Rolling text log | `%LOCALAPPDATA%\Lalia\logs\lalia.log.YYYY-MM-DD` | None. Unbounded. | Nowhere | No |
| Debug bundle | `%LOCALAPPDATA%\Lalia\logs\debug-bundle.txt` | None. Overwritten on each request, never removed. | Nowhere | No |
| History rows | `%APPDATA%\Lalia\lalia.db` | `retention_days`, default `None` meaning forever (`settings.rs:256`), applied only at startup (`app.rs:144`) | Yes, `docs/THREAT-MODEL.md:22` | Yes |
| Database backups | `%APPDATA%\Lalia\backup\lalia-YYYY-MM-DD.db` | 14 days (`db.rs:256`) | Nowhere | No |
| Kept audio | `%LOCALAPPDATA%\Lalia\audio\*.wav` | Tied to history rows | Yes, `docs/THREAT-MODEL.md:21` | Yes |
| Recovery audio | `%LOCALAPPDATA%\Lalia\recovery\last.wav` | Deleted only on a successful or copied dictation (`pipeline.rs:811`) | Partly, `docs/THREAT-MODEL.md:21` | Yes |

The row that matters most is the rolling text log: unbounded, undocumented, and outside every delete path.

### L1. HIGH. The rolling text log grows forever and no delete path touches it

CONFIRMED. `src-tauri/src/logging.rs:15`

`tracing_appender::rolling::daily(&dir, "lalia.log")` is the plain constructor, which carries no `max_log_files`. A new file is created per day and the old ones are kept forever. `delete_all_data` (`src-tauri/src/commands.rs:417`) wipes the database, the `audio` folder and the `recovery` folder, and never touches `logs_dir()`. The same is true of `delete_all_history` (`src-tauri/src/commands.rs:191`).

Why it matters: the log contains the exe name of every application the user dictated into, timings, machine details, and, if the user ever switches redaction off in Privacy settings (`settings.rs:249`, exposed at `src/pages/SettingsPage.tsx:306`), the full transcript text at `pipeline.rs:709`. A user who turns redaction on for debugging, forgets, and later presses "Delete everything" is left believing the text is gone while it sits in plain text on disk for the life of the machine.

Fix: build the appender with `tracing_appender::rolling::Builder::new().rotation(Rotation::DAILY).max_log_files(7).filename_prefix("lalia").build(dir)`. Add `let _ = std::fs::remove_dir_all(crate::paths::logs_dir());` followed by a recreate to both `delete_all_data` and `delete_all_history`. Document the log retention in `docs/THREAT-MODEL.md`.

### L2. MEDIUM. The journal and the text log record which application the user dictates into, on every dictation, outside the redaction switch

CONFIRMED. `src-tauri/src/pipeline.rs:363` and `src-tauri/src/pipeline.rs:364`

Every dictation writes `tracing::info!("recording started ... (target {}, {})", ctx.friendly_name, ctx.target.process_name)` and a matching journal line `{"app": "<process>.exe", "friendly": "<label>"}`. `insertion.rs:597` and `insertion.rs:670` add window class names. `friendly_name` is a fixed label from `context.rs:66` and holds no window title, which is the right choice, and `ctx.target.title` is never logged. So the leak is limited to the process name and window class.

The redaction switch in `logging.rs:38` covers transcript text alone. It has no effect on these lines.

Why it matters: for a public app, an application-usage timeline is sensitive on its own. A log showing `1password.exe`, `signal.exe`, a bank tab class and timestamps describes the user's day, and this file is the one users are most likely to attach to a bug report.

Fix: gate the process name behind the same redaction flag, or reduce it to the `AppCategory` (`context.rs:25`) which already exists and carries the diagnostic value without naming the application.

### L3. MEDIUM. The debug bundle is written to disk unasked, is never removed, and is built to be shared

CONFIRMED. `src-tauri/src/commands.rs:634`

`debug_bundle` writes `logs\debug-bundle.txt` containing the hardware profile, the hotkey settings, the last 300 journal events and the last 60 log lines flagged as problems (`commands.rs:459`). Nothing deletes it. The `is_problem` filter at `commands.rs:466` matches `" WARN "` and `" ERROR "`, so if redaction is off the transcript never appears here, since transcript lines are logged at `info` level. That part is safe.

What does end up in the file is 300 journal events, each naming an application (see L2), plus the user's shortcut configuration.

Why it matters: the whole purpose of this file is that a stranger pastes it into a GitHub issue. It should contain the least possible amount of personal detail, and it currently contains an application-usage timeline.

Fix: redact or categorise the `app` field when building the bundle, print a one-line notice in the interface saying what the file contains, and delete the file after the user copies it.

### L4. LOW. Redaction is a hand-applied convention with no compile-time guard

CONFIRMED. `src-tauri/src/logging.rs:38`, three call sites only

`crate::logging::redact` is called at `pipeline.rs:185`, `pipeline.rs:698` and `pipeline.rs:709`. All 74 sites matching `tracing::(info|debug|warn|error|trace)!` across `src-tauri/src/` were read, and none of the others interpolates transcript text. A separate search for `println!`, `eprintln!`, `print!`, `event!`, `span!` and `dbg!` returned two hits, both inside the `#[cfg(test)]` block at `whisper_server.rs:433` and `whisper_server.rs:436`, which print a full `TranscriptionResult` to the console. That block is the machine-bound manual test described in R2 and never runs in a shipped build. `pipeline.rs:783` interpolates `r.message`, and every `InsertReport.message` value in `insertion.rs` is a fixed English string. So today the invariant holds.

Why it matters: the invariant is enforced by discipline. One future `tracing::debug!("cleaned: {final_text}")` during a debugging session breaks the product promise silently, and the promise is written down at `docs/THREAT-MODEL.md:52`.

Fix: make the type carry the rule. Wrap transcript strings in a newtype whose `Display` and `Debug` implementations call `redact()`, so the plain text cannot reach a format string by accident.

### L5. LOW. The whisper sidecar's stdout and stderr are drained verbatim into the app log

CONFIRMED for the drain, CONFIRMED NEGATIVE for the transcript. `src-tauri/src/asr/whisper_server.rs:150` and `src-tauri/src/asr/whisper_server.rs:166`

Both pipes of `whisper-server.exe` are read line by line and forwarded to `tracing` with no filter and no redaction. This was checked against the vendored upstream source. `whisper_print_segment_callback`, which prints recognised text with `printf` at `vendor/whisper.cpp-src/examples/server/server.cpp:437`, is installed only when `params.print_realtime` is true (`server.cpp:972`). That flag is settable from the command line alone (`server.cpp:232`) and Lalia never passes `-pr` (`whisper_server.rs:100`). It is absent from `get_req_parameters` (`server.cpp:480` to `server.cpp:628`), so no HTTP caller can turn it on either. Transcript text does not reach the log by this route today.

What does reach the log is `printf("Received request: %s\n", filename)` at `server.cpp:836`, and Lalia always sends the constant name `audio.wav` (`whisper_server.rs:276`).

Why it matters: the guarantee depends on an upstream default in a vendored binary. A future whisper.cpp release that prints results by default, or a build that flips `print_realtime`, would start writing every transcript into `lalia.log` with nobody changing a line of Lalia.

Fix: pass an explicit opt-out flag if upstream adds one, and run the drained lines through an allow-list of known-safe prefixes, dropping anything unrecognised to `debug` with a length marker.

### L6. LOW. The panic hook writes absolute dependency paths into the log

CONFIRMED. `src-tauri/src/lib.rs:29`

`tracing::error!("PANIC: {info}")` records the full panic payload, whose location strings are the absolute build-machine paths described in R1.

Fix: fixed by fixing R1. Alternatively log `info.location()` file and line only.

### Checked and clean in this area

- The low-level keyboard hook logs modifier keys alone (`src-tauri/src/hotkey.rs:342`). `VK_LMENU`, `VK_RMENU`, `VK_LCONTROL`, `VK_RCONTROL`, `VK_LWIN`, `VK_RWIN` and nothing else. This confirms the claim at `docs/THREAT-MODEL.md:41`.
- The journal never carries transcript text. Every `journal::` call site was read (`app.rs:86`, `app.rs:91`, `app.rs:115`, `whisper_server.rs:220`, `commands.rs:588`, `insertion.rs:597`, `insertion.rs:670`, `pipeline.rs:364`, `pipeline.rs:559`, `pipeline.rs:785`). Character counts, durations, outcomes and window classes only.
- Verbose journal mode is off by default and reachable only through five clicks on the Diagnostics title (`settings.rs:281`, `commands.rs:581`).

---

## 2. Clipboard

### C1. MEDIUM. Dictated text is left on the clipboard, and the previous clipboard is abandoned, whenever the target does not consume the paste

CONFIRMED. `src-tauri/src/insertion.rs:668`

When no render request arrives after Ctrl+V, the code posts `WM_LALIA_SETTEXT`, which puts the real transcript on the clipboard (`insertion.rs:328`), returns `PasteNotConsumed` with the message "the application did not accept the paste; the text is on the clipboard", and returns without ever posting `WM_LALIA_RESTORE`. Retries were deliberately disabled on 2026-09-05 (`insertion.rs:648`), so this branch is reached after a single attempt whenever the target reads the clipboard without emitting `WM_RENDERFORMAT`.

The same happens for `unusable_target` (`insertion.rs:595`), for elevated targets (`pipeline.rs:799`), and by design in `CopyOnly` mode (`insertion.rs:714`).

The text then sits on the clipboard until the user copies something else. Any process running as the user can read it with `GetClipboardData`.

Why it matters: this is the routine path, and it is the one place where dictated speech leaves the application's own storage and becomes readable machine-wide for an unbounded time. The comment at `insertion.rs:648` shows the detection is known to be unreliable, so this branch fires more often than the wording "did not accept" suggests.

Fix: start a timer when this branch is taken and clear the clipboard after a bounded interval unless the user has pasted, or offer a Privacy setting "clear the clipboard N seconds after a failed insertion". Say in the overlay message that the text stays on the clipboard until it is replaced.

### C2. LOW. The clipboard history and cloud sync opt-out formats are set on the paths that publish text

CONFIRMED for the publish paths, SUSPECTED for survival through the delayed render. `src-tauri/src/insertion.rs:247`

`set_optout_formats` registers and writes all three documented formats: `ExcludeClipboardContentFromMonitorProcessing`, `CanIncludeInClipboardHistory` and `CanUploadToCloudClipboard` (`insertion.rs:248`). It is called on the delayed-render publish (`insertion.rs:314`) and on the plain copy path (`insertion.rs:337`), which together cover every path that puts dictated text on the clipboard. Windows Clipboard History and cloud sync should therefore ignore the text. This is a genuine strength and most dictation tools omit it.

The unverified part: `render_pending` (`insertion.rs:258`) supplies the real text during `WM_RENDERFORMAT` without calling `set_optout_formats` again. The reasoning is that the opt-out format data placed at publish time is still on the clipboard, since a delayed render must not call `EmptyClipboard`. That reasoning was never tested against a live clipboard-history listener.

Fix: verify with a live test that Win+V shows nothing after a normal dictation. If it does show the text, call `set_optout_formats()` inside `render_pending` as well. Add the test to the manual release checklist.

### C3. LOW. Delayed render on shutdown may publish the real text

SUSPECTED. `src-tauri/src/insertion.rs:286`

`WM_RENDERALLFORMATS` opens the clipboard and calls `render_pending`, which writes the real text. Windows sends this message to a clipboard owner that is about to be destroyed while a promise is outstanding. If the application exits between the paste keystroke and the restore, the transcript can be materialised onto the clipboard as a parting act, with no restore afterwards. This was reasoned from the message flow and never observed.

Fix: clear `pending_text` in the shutdown path before the owner window is destroyed, so `render_pending` has nothing to publish.

### C4. LOW. Only `CF_UNICODETEXT` is saved and restored; every other clipboard format is destroyed

CONFIRMED. `src-tauri/src/insertion.rs:300` and `src-tauri/src/insertion.rs:348`

`read_clipboard_text` captures `CF_UNICODETEXT` alone (`insertion.rs:228`), then `EmptyClipboard()` (`insertion.rs:301`) destroys everything on the clipboard. A copied image, a file selection, rich text or HTML is gone and the restore at `insertion.rs:355` puts back plain text alone. This limitation is disclosed at `docs/THREAT-MODEL.md:24`.

Why it matters: this is data loss more than data leak, and it is disclosed, which is why the severity is low. It still surprises users who dictate a caption after copying a picture.

Fix: enumerate formats with `EnumClipboardFormats` and save the handles for the common ones, or say in the interface that a non-text clipboard is replaced.

### C5. LOW. The previous clipboard text stays in process memory for the life of the process on the not-consumed path

CONFIRMED. `src-tauri/src/insertion.rs:320` with `src-tauri/src/insertion.rs:350`

`sh.saved_text` receives the previous clipboard text at publish time. It is taken and dropped by `WM_LALIA_RESTORE` at `insertion.rs:350`. The C1 branch returns without posting that message, so `saved_text` is never taken and holds the user's previous clipboard, which may be a password copied out of a password manager, until the next dictation overwrites it or the process exits.

Fix: clear `saved_text` explicitly on every exit path from `paste`, and zero the buffer before dropping it.

---

## 3. Network

Every outbound destination in the codebase, with what is sent:

| Host | Reached from | Payload | Anything user-specific? |
|---|---|---|---|
| `huggingface.co` | `models.rs:40`, `models.rs:104` | HTTP GET for a model file | No. `user-agent: Lalia/0.1` (`models.rs:229`) and nothing else. |
| `github.com` | `models.rs:125` | HTTP GET for the whisper.cpp CUDA release zip | No. Same client. |
| `127.0.0.1:<ephemeral>` | `whisper_server.rs:84` | The recorded audio as `multipart/form-data`, plus the dictionary hint prompt | Audio and hint terms, on the loopback interface only. |
| `api.openai.com` or a user-set base URL | `openai_compat.rs:68` | The recorded audio, the hint prompt, a bearer API key | Yes, when the user opts in. Off by default. |

There is no telemetry, no analytics, no crash reporting and no update check anywhere in the tree. The frontend makes zero network calls: a search of `src/` for `fetch(`, `XMLHttpRequest`, `analytics`, `sentry`, `telemetry`, `posthog`, `gtag` and `http` returned nothing, and the Tauri CSP at `src-tauri/tauri.conf.json:44` is `default-src 'self'; ...; connect-src ipc: http://ipc.localhost`, which blocks web-view network access anyway.

### N1. MEDIUM. The local speech server has no authentication and answers every origin

CONFIRMED. `vendor/whisper.cpp-src/examples/server/server.cpp:730`, with `src-tauri/src/asr/whisper_server.rs:100` and `src-tauri/src/engine.rs:68`

`whisper-server.exe` is started with `--host 127.0.0.1` (`whisper_server.rs:102`), so the network cannot reach it. That part is correct. Inside the machine there is no barrier at all:

- Any process running as the user can `POST /inference` with its own audio and read the transcript, using the model and the GPU the user paid for.
- The upstream server sets `Access-Control-Allow-Origin: *` and `Access-Control-Allow-Headers: content-type, authorization` on every response (`server.cpp:729`), and registers an `OPTIONS` handler for the inference path (`server.cpp:814`). Any web page the user visits can therefore script a cross-origin `fetch` to `http://127.0.0.1:<port>/inference` and read the answer. The page must find the port first.
- The port varies per launch. `engine.rs:68` binds `127.0.0.1:0`, takes whatever the OS hands back, and passes it to the child. So a page has to scan, which is noisy and slow but well within reach of a determined script.
- There is no request-path secret. Lalia passes `--inference-path /inference` (`whisper_server.rs:117`) and leaves `--request-path` (`server.cpp:254`) empty, so the endpoint is at the predictable default once the port is known.

Why it matters: the product promise is that speech stays on the machine. It does, and this finding does not break that. What it does break is the narrower expectation that the microphone-to-text engine belongs to this application. Local malware gets free high-quality transcription, and a malicious page gets the same over CORS.

Fix: two cheap steps that need no upstream change. First, generate a random path prefix at each launch and pass it as `--request-path /<32 random hex>`, so the endpoint is unguessable even after a port scan. Second, if upstream ever gains an origin or token option, use it. A stronger fix is to move off the HTTP sidecar to a named pipe or to an in-process binding, which removes the local attack surface entirely.

### N2. LOW. The cloud transcription provider contradicts a literal reading of the README

CONFIRMED. `src-tauri/src/asr/openai_compat.rs:55` and `README.md:7`

Selecting the `openai_compatible` provider uploads the raw WAV and the hint prompt to `openai_base_url` (`settings.rs:149`, user-editable) with the API key as a bearer header. `README.md:7` says "Nothing leaves the computer", and `README.md:36` then says "optional cloud provider exists and is off". The default is `whisper_local` (`settings.rs:138`) and the interface labels the option honestly: "OpenAI-compatible API (paid, uploads audio)" at `src/i18n.ts:496`. The key is held in Windows Credential Manager (`openai_compat.rs:24`) and never written to `settings.json`.

Note that this client is built without `no_proxy()` (`openai_compat.rs:20`), unlike the local one (`whisper_server.rs:71`), so a system or corporate proxy sits in the path. That is the correct behaviour for an internet call and is mentioned here for completeness.

Why it matters: the absolute sentence sits 29 lines above its own qualifier. A reviewer or a journalist quoting line 7 will be quoting something that a setting can make untrue.

Fix: rewrite `README.md:7` as "Nothing leaves the computer unless you switch on the optional cloud provider, which is off." Add a confirmation dialog the first time the provider is selected. Consider clearing the keyring entry from `delete_all_data`.

### Checked and clean in this area

- No telemetry, analytics, crash reporting or update check exists anywhere in the tree.
- The frontend makes zero network calls, and the Tauri CSP (`src-tauri/tauri.conf.json:44`) blocks web-view network access regardless.
- Model and runtime downloads are hash-pinned and verified. `download_verified` (`models.rs:224`) streams to a `.part` file, computes SHA-256, compares against the pinned constant and refuses on mismatch (`models.rs:251`). Every catalogue entry carries a hash (`models.rs:49`), and the runtime zip that delivers executable DLLs is covered by the same guard (`models.rs:126` pinned, `models.rs:307` verified). The only residual is that the hashes live in the source, so a compromised repository could change a URL and its hash together. Low priority for this threat model.
- Nothing user-specific reaches huggingface.co or github.com. The client sends `user-agent: Lalia/0.1` (`models.rs:229`) and no machine id, account name, email or install id.

---

## 4. Storage at rest

### S1. HIGH. Daily database backups hold 14 days of transcripts, survive "Delete everything", and defeat the retention setting

CONFIRMED. `src-tauri/src/db.rs:247` with `src-tauri/src/commands.rs:417`

`Db::open` calls `daily_backup` before anything else touches the file (`db.rs:277`). It copies `%APPDATA%\Lalia\lalia.db` to `%APPDATA%\Lalia\backup\lalia-YYYY-MM-DD.db` once per day and prunes copies older than 14 days (`db.rs:256`).

Neither `delete_all_data` (`commands.rs:417`) nor `delete_all_history` (`commands.rs:191`) nor `wipe_everything` (`db.rs:839`) touches the `backup` folder. The retention purge at `app.rs:144` deletes rows from the live database alone.

The result: a user who sets retention to 7 days, or who presses the red "Delete everything" button, still has up to 14 days of complete transcript history sitting in plain SQLite files a few directories away. Nothing in the interface or in `docs/THREAT-MODEL.md` mentions that the folder exists.

Why it matters: this is the single clearest gap between what the application promises and what is on the disk. The delete button is the promise, and it is incomplete. For a public tool people install because it is private, a delete that leaves 14 days behind is the finding that costs trust.

Fix: delete the `backup` folder inside `delete_all_data` and `delete_all_history`. Apply the same retention window to backups that the user chose for history, so a 7-day retention prunes backups older than 7 days. Document the folder in `docs/THREAT-MODEL.md` and mention it under the delete button.

### S2. MEDIUM. Transcripts are kept forever by default, in plain text, and retention runs only at startup

CONFIRMED. `src-tauri/src/settings.rs:255`, `src-tauri/src/db.rs:19`, `src-tauri/src/app.rs:144`

`PrivacySettings::default` sets `keep_history: true` and `retention_days: None`, and the comment on `settings.rs:244` states plainly that `None` means keep forever. The `history` table stores `raw_text`, `cleaned_text`, `final_text` and `edited_text` as TEXT (`db.rs:22` to `db.rs:44`). There is no `PRAGMA key` and no SQLCipher anywhere in the file, so the database is readable by any process running as the user and by anyone with the disk. `learning_events` stores `before_text` and `after_text` in the same way (`db.rs:79`).

The purge only runs during startup (`app.rs:144`). An application left running for a month applies no retention during that month.

The delete paths that do exist are real and complete for what they cover: per entry with its audio file (`commands.rs:183`), all history with the audio and recovery folders (`commands.rs:191`), learning data alone (`db.rs:601`), and everything (`commands.rs:417`). Export is available too (`commands.rs:410`).

Why it matters: the default is the setting almost everyone keeps. Every sentence a user has ever dictated, including passwords typed by voice into a field the sensitive-application heuristic did not catch, accumulates without limit in a file that any program on the machine can open.

Fix: ship a bounded default such as 90 days. Run the retention purge on a timer as well as at startup. Consider encrypting `lalia.db` with a key held in Credential Manager, which the application already uses at `openai_compat.rs:25`.

### S3. MEDIUM. Raw audio of the most recent utterance stays on disk after any insertion that did not succeed

CONFIRMED. `src-tauri/src/pipeline.rs:571` with `src-tauri/src/pipeline.rs:811`

Every dictation writes the encoded WAV of the captured speech to `%LOCALAPPDATA%\Lalia\recovery\last.wav` (`pipeline.rs:573`). It is removed only when `status` is `success` or `copied` (`pipeline.rs:811`). Any other outcome, including a crash, a forced exit or the application being killed between those two points, leaves the file behind. The next dictation overwrites it, so at most one utterance is on disk at a time, and that one utterance can stay there without limit.

This write happens regardless of `keep_audio: false` (`settings.rs:257`) and regardless of `keep_history` and `retention_days`. `docs/THREAT-MODEL.md:21` describes the file and says it is deleted on success, so the mechanism is disclosed. The residue after a failure is not.

The opt-in audio path is separate and behaves correctly: `%LOCALAPPDATA%\Lalia\audio\<id>.wav` is written only when `keep_audio` and `keep_history` are both on (`pipeline.rs:818`), its path is stored on the history row, and deleting the row deletes the file (`commands.rs:184`).

Why it matters: the Privacy tab shows "Also keep the audio" as off, which a user reads as a promise that voice never touches the disk. One failed paste turns that into a recording of their voice sitting in AppData until they dictate again, which could be next week.

Fix: delete `recovery\last.wav` on every terminal outcome, keeping it only while a retry is genuinely offered, and delete it at startup as a crash sweep. State the failure case in `docs/THREAT-MODEL.md:21`.

### S4. LOW. Turning learning off still stores the user's edited text

CONFIRMED. `src-tauri/src/commands.rs:226`

`record_edit` calls `set_history_edit` at `commands.rs:228`, which writes the edited text into `history.edited_text`. The `learning_enabled` check is at `commands.rs:231`, three lines later, and only skips the derivation of dictionary suggestions. So the corrected text is persisted even with learning switched off.

Why it matters: a user who switches learning off expects their corrections to stop being kept. What stops is the suggestion engine.

Fix: move the gate above `set_history_edit`, or relabel the setting so it describes suggestion generation alone.

### S5. LOW. Learning events store complete before and after text pairs

CONFIRMED. `src-tauri/src/db.rs:79`

`learning_events` holds `before_text` and `after_text` as full strings, so it is a second plaintext copy of the transcript alongside the history row. It is bounded by no retention rule; `delete_history_older_than` (`db.rs:423`) touches the `history` table alone. A dedicated wipe exists at `db.rs:601` and the full wipe at `db.rs:839` covers it.

Fix: include `learning_events` in the retention purge, or store only the differing tokens, which is all `learning.rs:56` needs.

### S6. LOW. A temp directory is created on every start and never used

CONFIRMED. `src-tauri/src/paths.rs:57` and `src-tauri/src/paths.rs:72`

`temp_dir()` is created by `ensure_all` and referenced nowhere else in `src-tauri/src/`. Audio reaches the engine in memory as a multipart body (`whisper_server.rs:274`) and never as a temp file, which is the right design and worth stating because it removes a whole class of leak. The unused directory is dead code.

Fix: remove `temp_dir` from `ensure_all` and from `paths.rs`, so no future contributor assumes it is a sanctioned place to spill audio.

---

## 5. Secrets and repository hygiene

### R1. HIGH. The shipped release binary embeds the developer's Windows username

CONFIRMED. `src-tauri/target/release/lalia.exe`

A byte scan of the built release binary returns 274 distinct strings containing `C:\Users\luram\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\...`. These are the panic-location strings that Rust embeds for dependency crates, one per crate that can panic: `tauri-2.11.5`, `tokio-1.53.1`, `regex-automata`, `parking_lot_core`, `base64-0.22.1`, `anyhow-1.0.104` and many more.

Anyone who downloads the executable, from GitHub releases or fuckyouflow.app, can read the developer's Windows account name out of it with a text editor.

Why it matters: this is the classic public-release leak. The account name is often a real first name or a handle reused elsewhere, and it hands a doxxing or spear-phishing starting point to every downloader of a privacy tool. It also undercuts the audit story: a binary that leaks its author's home directory is hard to present as careful about paths.

Fix: build with path remapping. Add to `src-tauri/.cargo/config.toml`:

```toml
[build]
rustflags = ["--remap-path-prefix=C:\\Users\\luram\\.cargo=/cargo", "--remap-path-prefix=C:\\Claude Projects\\lalia=/src"]
```

Verify after the next build with a scan of the binary for `Users`. Better still, produce release builds in CI on a neutral runner, which removes the class of problem permanently.

### R2. LOW. Three tracked files carry the developer's absolute project path

CONFIRMED. `scripts/build-vulkan.ps1:10`, `scripts/build-vulkan.ps1:11`, `site/build_site.py:10`, `src-tauri/src/asr/whisper_server.rs:434`

All four lines hardcode `C:\Claude Projects\lalia` or `C:/Claude Projects/lalia`. The `whisper_server.rs` one is inside a `#[cfg(test)]` block that reads `C:/Claude Projects/lalia/eval/corpus/el-01.wav`, so that test can only ever pass on one machine.

The username itself does not appear. A search of the working tree and of the complete git history for `Users\luram` and `Users/luram` returned zero hits.

Why it matters: minor on its own. It tells a reader the layout of the author's disk and it makes the repository harder to build for a contributor, which matters for a project inviting contributions.

Fix: derive the paths from `$PSScriptRoot`, from `Path(__file__).parent` and from `env!("CARGO_MANIFEST_DIR")` respectively. Mark the machine-bound test `#[ignore]`.

### R3. LOW. Two privacy-shaped settings are serialized but never read

CONFIRMED. `src-tauri/src/settings.rs:165` and `src-tauri/src/settings.rs:248`

`cleanup.cloud_cleanup_enabled` and `privacy.learn_from_edits` are written to `settings.json`, are declared in the frontend type at `src/api.ts:20` and `src/api.ts:27`, and are read by no Rust code. Neither is rendered as a control: the Privacy tab (`src/pages/SettingsPage.tsx:299` to `src/pages/SettingsPage.tsx:306`) shows five toggles and none of them is `learn_from_edits`, and no tab renders `cloud_cleanup_enabled`.

So these are dead serialized fields, and no switch in the interface makes a false promise about them, which is why the severity is low. The risk is that `cloud_cleanup_enabled` appears in a user's `settings.json` and reads as a promise that cloud cleanup is under their control.

Fix: delete both fields, or implement them and render them.

### R4. LOW. Deleting everything leaves the cloud API key in Credential Manager

CONFIRMED. `src-tauri/src/commands.rs:417` with `src-tauri/src/asr/openai_compat.rs:29`

`delete_all_data` never calls `OpenAiCompat::delete_key`, so the stored bearer token survives a full wipe. A `delete_key` function already exists and is exposed.

Fix: call `OpenAiCompat::delete_key()` from `delete_all_data`, or list the credential explicitly in the confirmation text so the user knows it stays.

### Checked and clean in this area

- No secrets in the working tree. A search of tracked files for `sk-`, `ghp_`, `github_pat_`, `AKIA`, `xox[baprs]-` and PEM private-key headers returned nothing.
- No secrets in git history. The same search across `git log --all -p` returned nothing, and no `.env`, `.db`, `.wav` or key-named file was ever committed and later removed.
- The API key is handled correctly. It lives in Windows Credential Manager through `keyring` (`openai_compat.rs:24`), is fetched per request (`openai_compat.rs:42`), is never written to `settings.json`, and appears in no log line. This confirms the claim at `docs/THREAT-MODEL.md:43`.
- `.gitignore` correctly excludes `/models/`, `/vendor/`, `*.wav`, `*.log`, `/eval/private/` and `/src-tauri/bundled/`. The local `dev.log`, `build*.log` and `test*.log` files in the repository root are untracked.

---

## Summary of every finding

| ID | Severity | Title | Location | Status |
|---|---|---|---|---|
| S1 | HIGH | Database backups keep 14 days of transcripts and survive "Delete everything" | `db.rs:247` | CONFIRMED |
| R1 | HIGH | Shipped release binary embeds the developer's Windows username | `target/release/lalia.exe` | CONFIRMED |
| L1 | HIGH | Rolling text log grows forever and no delete path touches it | `logging.rs:15` | CONFIRMED |
| S3 | MEDIUM | Recovery audio stays on disk after any insertion that did not succeed | `pipeline.rs:571` | CONFIRMED |
| C1 | MEDIUM | Dictated text left on the clipboard when the target refuses the paste | `insertion.rs:668` | CONFIRMED |
| S2 | MEDIUM | Transcripts kept forever by default, plaintext, retention only at startup | `settings.rs:255` | CONFIRMED |
| N1 | MEDIUM | Local speech server is unauthenticated and answers every origin | `server.cpp:730` | CONFIRMED |
| L2 | MEDIUM | Application usage recorded on every dictation, outside redaction | `pipeline.rs:363` | CONFIRMED |
| L3 | MEDIUM | Debug bundle written unasked, never removed, built to be shared | `commands.rs:634` | CONFIRMED |
| L4 | LOW | Redaction is a hand-applied convention with no compile-time guard | `logging.rs:38` | CONFIRMED |
| L5 | LOW | Whisper sidecar output drained verbatim into the app log | `whisper_server.rs:166` | CONFIRMED |
| L6 | LOW | Panic hook writes absolute dependency paths into the log | `lib.rs:29` | CONFIRMED |
| C2 | LOW | Clipboard opt-out formats set, survival through delayed render untested | `insertion.rs:247` | SUSPECTED |
| C3 | LOW | Delayed render on shutdown may publish the real text | `insertion.rs:286` | SUSPECTED |
| C4 | LOW | Only `CF_UNICODETEXT` saved and restored, other formats destroyed | `insertion.rs:300` | CONFIRMED |
| C5 | LOW | Previous clipboard text held in memory for the process lifetime | `insertion.rs:320` | CONFIRMED |
| N2 | LOW | Cloud provider contradicts a literal reading of the README | `README.md:7` | CONFIRMED |
| S4 | LOW | Turning learning off still stores the user's edited text | `commands.rs:226` | CONFIRMED |
| S5 | LOW | Learning events store complete before and after text pairs | `db.rs:79` | CONFIRMED |
| S6 | LOW | Unused temp directory created on every start | `paths.rs:57` | CONFIRMED |
| R2 | LOW | Three tracked files carry the developer's absolute project path | `scripts/build-vulkan.ps1:10` | CONFIRMED |
| R3 | LOW | Two privacy-shaped settings serialized but never read | `settings.rs:165` | CONFIRMED |
| R4 | LOW | Deleting everything leaves the cloud API key in Credential Manager | `commands.rs:417` | CONFIRMED |

Counts: 0 CRITICAL, 3 HIGH, 6 MEDIUM, 14 LOW, 23 in total. Two of the LOW findings are SUSPECTED and need a live test to settle.

## What could not be verified without running the application

- Whether Windows Clipboard History and cloud sync genuinely ignore the dictated text after a delayed render (C2). This needs Win+V checked after a real dictation.
- Whether `WM_RENDERALLFORMATS` publishes the transcript on exit (C3). This needs the application killed between a paste keystroke and the restore.
- Whether the shipped installer contains the same `C:\Users\luram` strings as `target/release/lalia.exe` (R1). The scan covered the built executable, and the NSIS bundle was not examined.
