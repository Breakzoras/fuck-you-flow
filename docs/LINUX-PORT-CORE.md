# Linux port: the Rust core

Every `.rs` file under `src-tauri/src/` was read in full and classified. The core is 31 files and
12,352 lines (`wc -l`), of which 4,414 lines in 17 files are portable as written, 4,500 lines in 8
files carry small isolated Windows bits, and 3,438 lines in 6 files are Windows implementations of
capabilities that Linux also needs. No file is Windows-only: every Windows module in this tree
exists to deliver something the product still requires on Linux. The whole dictation path is
concentrated in the twin group, and the Linux build does not compile today for two concrete
reasons: `hw.rs` calls `GetUserDefaultLocaleName` at line 357 with no `cfg` guard at all, and
`insertion.rs` exposes only one of its nine public functions on non-Windows targets, so twelve
call sites across `pipeline.rs`, `app.rs` and `commands.rs` have nothing to bind to. A second and
more dangerous class already compiles on Linux while silently doing nothing: the global hotkey in
`hotkey.rs` installs no hook, the sensitive-application refusal in `context.rs` never matches, the
floating pill in `overlay.rs` falls back to a hardcoded screen rectangle, and
`models::cuda_driver_present` reports every machine as having no NVIDIA driver. `hw.rs` belongs to
both classes: one unguarded call breaks the build, and once that is fixed its stubs report every
machine as having no graphics card.

## What was checked

Each file was opened and read. Across the tree, `grep -n` was run for `windows::`,
`#[cfg(windows)]`, `#[cfg(not(windows))]`, `#[cfg(target_os = "windows")]`, `std::os::windows::`,
raw Win32 symbols (`SendInput`, `keybd_event`, `SetWindowsHookEx`, `HWND`, `GetForegroundWindow`,
clipboard APIs, UI Automation), `.exe`, `.dll`, `creation_flags`, `CREATE_NO_WINDOW`, `winreg`,
`CoInitialize`, `keyring`, `AppData`, `LOCALAPPDATA`, `ProgramData`, `MSIX`, drive letters and
backslash separators. Every hit was read in place.

Three verified negatives are worth recording, because they remove work from the plan:

- **No registry access anywhere.** There is no `winreg` dependency, no `Win32_System_Registry`
  feature in `Cargo.toml`, and no `RegOpenKey` or `HKEY_` in the source. The autostart entry is
  written by `tauri_plugin_autostart`, which has its own Linux backend.
- **No MSIX or packaged-app layout logic in the Rust source.** `grep -ri "msix|WindowsApps"` over
  `src-tauri/src/` returns nothing. `paths.rs` reaches the data directories through the `dirs`
  crate.
- **The `[target.'cfg(windows)'.dependencies.windows]` feature list in `Cargo.toml` is the
  bounding box, and nothing escapes it.** Three of the twenty features it enables have no user in
  `src/`: `Win32_UI_HiDpi`, `Win32_System_ProcessStatus` and `Win32_Graphics_Dxgi_Common`.

The task brief described this tree as roughly 22 files and 8,500 lines. The real figures from
`wc -l` are 31 files and 12,352 lines, and three of the named files are larger than the brief
states (`hotkey.rs` 927 against 580, `insertion.rs` 1,194 against 900, `pipeline.rs` 1,183 against
985). The counts in this document are the measured ones throughout.

## File by file

| File | Lines | Verdict | What binds it to Windows | What Linux needs instead |
|---|---|---|---|---|
| `insertion.rs` | 1194 | NEEDS A LINUX TWIN | `mod win` (124-1186) is the whole file: clipboard delayed render, `SendInput`, hidden message window, elevation checks | Synthetic input and clipboard ownership for X11 and Wayland, plus a foreground-window snapshot |
| `hotkey.rs` | 927 | NEEDS A LINUX TWIN | `SetWindowsHookExW(WH_KEYBOARD_LL)` and `WH_MOUSE_LL` in `mod win` (436-796), with a 1 s re-install watchdog | A global key grab that reports key-up and can swallow a chord |
| `models.rs` | 644 | NEEDS A LINUX TWIN | Pinned Windows CUDA zip (146), a twelve-entry `.exe`/`.dll` manifest (476-489), `tar` extraction (511-529), `nvcuda.dll` probe (560-563) | A Linux acquisition path for the whisper.cpp binaries and a Linux CUDA probe |
| `hw.rs` | 403 | NEEDS A LINUX TWIN | DXGI adapter enumeration, PDH performance counters, `GlobalMemoryStatusEx`, and an unguarded `GetUserDefaultLocaleName` (357) | GPU and VRAM detection, total RAM, Vulkan and CUDA presence, user locale |
| `context.rs` | 216 | NEEDS A LINUX TWIN | UI Automation password detection (145-192) and a process table keyed on `.exe` names (55-113) | Accessibility introspection for password fields and a Linux process-name table |
| `jobobject.rs` | 54 | NEEDS A LINUX TWIN | `mod imp` (5-47): job object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` (22) | A guarantee that the engine child dies with the app after a hard crash |
| `pipeline.rs` | 1183 | MOSTLY PORTABLE | One `#[cfg(windows)]` block (407-429) for the UIA password check; everything else reaches Windows through `insertion::` | The `insertion` and `context` twins; 23 lines of its own |
| `commands.rs` | 1183 | MOSTLY PORTABLE | `explorer` spawn (599, 605), Windows virtual-key capture in `record_shortcut` (672-738), `hook_rehooks` diagnostic (625) | A portable folder opener and a Linux keycode capture and name table |
| `settings.rs` | 523 | MOSTLY PORTABLE | Default chords `Ctrl+Win` and `RAlt` (78-80), a sharing-violation retry loop (355-383), BOM strip (388) | A Linux default chord and explicit 0600 file permissions |
| `asr/whisper_server.rs` | 452 | MOSTLY PORTABLE | `whisper-server.exe` lookup (402), `CREATE_NO_WINDOW` (129-133), job-object assignment (141-144) | The binary name without a suffix, Linux candidate directories, and a child-lifetime guard |
| `app.rs` | 429 | MOSTLY PORTABLE | `uninstall.exe` marker (75) gates autostart (54-61); tray construction failure aborts startup (322, 419) | A Linux definition of "installed copy" and a non-fatal tray path |
| `paths.rs` | 423 | MOSTLY PORTABLE | `is_installation` checks for `.exe` files (31); doc comment names `%APPDATA%` (3-4) | Nothing structural; `dirs::config_dir` and `dirs::data_local_dir` already map to XDG |
| `lib.rs` | 213 | MOSTLY PORTABLE | `MessageBoxW` startup failure dialog (33-64) | A graphical failure dialog, and the `panic.log` write the Linux arm currently drops |
| `overlay.rs` | 94 | MOSTLY PORTABLE | `MonitorFromWindow` and `GetMonitorInfoW` work-area lookup (61-76) | The work area of the monitor holding the focused window |
| `db.rs` | 1014 | PORTABLE | none | Explicit 0600 permissions on the database and its backups |
| `audio.rs` | 911 | PORTABLE | none | Nothing at compile time; device names saved on Windows will never match ALSA names |
| `cleanup/deterministic.rs` | 380 | PORTABLE | none | nothing |
| `engine.rs` | 304 | PORTABLE | none | nothing |
| `cleanup/questions.rs` | 322 | PORTABLE | none | nothing |
| `cleanup/dictionary.rs` | 242 | PORTABLE | none | nothing |
| `audiofile.rs` | 198 | PORTABLE | none | nothing |
| `journal.rs` | 186 | PORTABLE | none | nothing |
| `logging.rs` | 185 | PORTABLE | none | nothing |
| `cleanup/divergence.rs` | 120 | PORTABLE | none | nothing |
| `cleanup/mod.rs` | 114 | PORTABLE | none | nothing |
| `learning.rs` | 108 | PORTABLE | none | nothing |
| `asr/openai_compat.rs` | 103 | PORTABLE | none | A Secret Service provider on the machine at runtime |
| `cleanup/snippets.rs` | 90 | PORTABLE | none | nothing |
| `asr/mod.rs` | 76 | PORTABLE | none | nothing |
| `scratch.rs` | 55 | PORTABLE | none | Nothing at compile time; window raising may be refused under Wayland |
| `main.rs` | 6 | PORTABLE | `windows_subsystem` attribute (2), documented as ignored on other targets | nothing |

## The six that need a Linux twin

### `insertion.rs` (1194 lines)

The hardest file in the tree. `mod win` spans lines 124 to 1186, which is 1,063 of the 1,194
lines. The re-export at lines 1188-1189 is `#[cfg(windows)]` and publishes nine functions:
`capture_target`, `copy_only`, `ensure_started`, `foreground_hwnd`, `paste`,
`read_clipboard_string`, `restore_focus`, `type_text`, `window_alive`. The `#[cfg(not(windows))]`
arm at lines 1191-1194 supplies exactly one of them, `capture_target`, returning
`Target::default()`. The other eight do not exist on Linux, so twelve call sites have nothing to
bind to: `pipeline.rs` lines 867, 876, 897 (twice), 898, 901, 902, 903, 905, 1112 and 1127, plus
`app.rs` line 292 (`ensure_started`) and `commands.rs` line 956 (`copy_only`). This is a hard
compile break, and it is also the honest signal that nobody has begun the Linux side of this
module.

The mechanisms `paste` (line 798) tries, in the order it tries them:

1. **Start the clipboard-owning thread.** `ensure_started` (589) spawns `thread_main` (552), which
   registers window class `FuckYouFlowClipboardOwner` (555) and creates an `HWND_MESSAGE` window
   (558), then runs a `GetMessageW` loop (582). Every Win32 call in the module is marshalled to
   this one thread by `PostMessageW`.
2. **Publish a delayed-render promise.** `WM_LALIA_PUBLISH` (810) drives `snapshot_clipboard`
   (292), which saves every existing format, then advertises `CF_UNICODETEXT` with no data.
   `set_optout_formats` (386, via 370) registers `CanUploadToCloudClipboard`,
   `ExcludeClipboardContentFromMonitorProcessing` and `CanIncludeInClipboardHistory` (371-374) so
   the transcript stays out of cloud clipboard and clipboard history.
3. **Wait for the user to release the hotkey.** `wait_for_modifier_release` (666) polls
   `GetAsyncKeyState` (661) for up to 400 ms, then `release_modifiers` (717) synthesises key-ups
   through `SendInput` (728).
4. **Refuse windows that cannot hold text.** `unusable_target` (695) reads the class name with
   `GetClassNameW` (682) and rejects `Progman`, `WorkerW`, `Shell_TrayWnd`,
   `Shell_SecondaryTrayWnd` and `tray_icon_app` through `unusable_class` (33).
5. **Wait for the clipboard to be free.** Lines 913-926 poll `GetOpenClipboardWindow` for up to
   250 ms so the paste keystroke does not race another process holding the clipboard open.
6. **Read the clipboard back.** `clipboard_holds` (752) confirms the clipboard still carries this
   transcript; a replacement aborts the paste (934-951).
7. **Send the chord.** `send_paste_chord` (734) issues Ctrl+V, or Ctrl+Shift+V for terminals,
   through `SendInput` (746).
8. **Measure who consumed it.** `render_pending` (390) answers `WM_RENDERFORMAT` and records which
   process asked, so a read by the target process counts as proof of paste and a read by a
   clipboard manager does not. A settle sleep follows at line 980.
9. **Restore the previous clipboard.** `WM_LALIA_RESTORE` (1005) drives `restore_snapshot` (337).

Two fallbacks sit beside it. `type_text` (1033) sends each UTF-16 unit as a
`KEYEVENTF_UNICODE` scan code through `SendInput` (1049-1060), with `VK_RETURN` for newlines
(1041). `copy_only` (1017) leaves the text on the clipboard through `WM_LALIA_SETTEXT`. The choice
among the three is made in `pipeline.rs` lines 891-906: an elevated target forces `CopyOnly`, a
dead window (867) forces `copy_only`, and focus that drifted away forces `restore_focus` (1163)
followed by `copy_only` when the restore fails (897-898).

Supporting Win32 surface: `is_process_elevated` (1085) and `self_elevated` (1103) use
`OpenProcessToken` with `TokenElevation`; `process_name` (1117) uses
`QueryFullProcessImageNameW`; `capture_target` (1132) uses `GetForegroundWindow`,
`GetWindowThreadProcessId` and `GetWindowTextW`; `restore_focus` (1163) uses
`SetForegroundWindow` with the documented ALT-press trick (1175); `focus_diagnostics` (701) uses
`GetGUIThreadInfo`.

A Linux implementation has to supply four capabilities behind the same nine-function API:
synthetic key injection, clipboard ownership with deferred content, a foreground-window snapshot
carrying process name and title, and focus restoration. On X11 these map to XTEST, the ICCCM
selection protocol (whose owner-serves-on-request model is a close analogue of the delayed render
promise), and `_NET_ACTIVE_WINDOW`. On Wayland none of the four is available to an ordinary
client: input injection needs the virtual-keyboard protocol, a portal, or `uinput`; clipboard
writes need a focused surface or the data-control protocol; there is no way to learn which window
is focused; and focus stealing is refused by policy. Which of these a given compositor allows is
**not verified**. The "did the target actually read the clipboard" measurement in step 8 has no
counterpart on either display server, so the success reporting will have to be rebuilt on a
different signal. The class-name table at lines 33-44 is pure string matching and compiles
anywhere, and its contents are Win32 window classes, so it needs a Linux equivalent list.

### `hotkey.rs` (927 lines)

`mod win` spans lines 436 to 796, which is 361 lines. The rest of the file is chord bookkeeping
that is structurally portable while being keyed on Windows virtual-key numbers.

**How the hook is installed.** `run_hook_thread` (713) first raises its own thread to
`THREAD_PRIORITY_HIGHEST` (722-725), because Windows silently unhooks a callback that misses the
`LowLevelHooksTimeout`. It then calls `SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), hmod, 0)`
at line 727 and `SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), hmod, 0)` at line 738. The
mouse hook exists so the two side buttons can hold a chord. `hook_proc` (500) filters out its own
injected events by matching `LLKHF_INJECTED` against `LALIA_INJECT_SIG` (505), records only
modifier keys to the log (516-526), and hands everything to `dispatch_key` (543). Returning
`SWALLOW`, defined as `LRESULT(1)` at line 450, consumes the key so no other application sees it.

**The watchdog.** Lines 748-793. `SetTimer(None, 1, 1_000, None)` at line 762 fires `WM_TIMER`
once a second. On each tick the loop installs a **fresh** keyboard hook first (766), and only then
unhooks the old one (768), so no key press can fall through the gap; the same is done for the
mouse hook at 781-788. `HOOK_REHOOKS` (770) counts the re-installs and is surfaced in the
diagnostics panel through `commands.rs` line 625. The comment at lines 754-760 records the
measurement that drove the one-second cadence: on 10 September 2026 at 100 percent CPU with 732
processes, eleven key events were delivered by the keyboard and seen by no hook.

`install` (799) wraps the thread spawn in `#[cfg(windows)]` at line 801. On Linux this file
therefore compiles and installs nothing, and the app runs with a dead hotkey and no error. The
only other guarded line is `reset_pressed_state` (407), whose `GetAsyncKeyState` sweep at 409-410
is skipped on Linux, so stale key state is never pruned.

Linux needs a global key grab that reports key-up as well as key-down, ignores auto-repeat, and
can suppress the chord for other applications. Candidate mechanisms: X11 `XGrabKey` or an
XInput2 passive grab; `org.freedesktop.portal.GlobalShortcuts` on Wayland; or reading `evdev` and
writing `uinput` directly, which needs the user in the `input` group. Suppression is the awkward
part: an `evdev` grab takes the whole device, and the portal decides suppression for you. Whether
any of these can reproduce the Right Alt push-to-talk behaviour, where AltGr must keep typing
accented characters, is **not verified**. The chord engine itself (`Chord::parse` at 102,
`is_down` at 169, `vk_name` at 236, `refresh_active` at 422) is reusable once a Linux keycode is
mapped onto the same `u16` numbering.

### `models.rs` (644 lines)

The model catalogue half of this file is portable. The runtime installer half is entirely
Windows. `runtime_spec` pins
`whisper-cublas-12.4.0-bin-x64.zip` at line 146 with a SHA-256 at line 147. `RUNTIME_KEEP`
(476-489) names `whisper-server.exe` and eleven DLLs. `install_runtime` (497) extracts with
`std::process::Command::new("tar")` at lines 511-512, 523 and 529, which works on Windows 10 and
later because the bundled `tar.exe` is bsdtar and reads zip archives; line 529 additionally passes
`--wildcards`, which is GNU tar syntax, so no single tar binary on Linux satisfies both
requirements. `cuda_driver_present` (560-563) reads `SystemRoot`, falls back to the literal
`"C:\\Windows"`, and probes `System32\nvcuda.dll`. That function compiles on Linux and returns
`false` on every machine, which makes it a dummy-value stub by accident.

Linux needs a different answer to "where do the whisper.cpp binaries come from": a distribution
package, a build from source, or a Linux artifact produced by this project. Whether release
`b4938` even carries a Linux asset is **not verified**. `cuda_driver_present` needs a
`libcuda.so.1` probe or an equivalent. The catalogue (53-134), `download_verified` (420-457),
`sha256_file` and `marker_ok` (381-407) and the `move_models` migration (216-346) are portable as
written and should be kept.

### `hw.rs` (403 lines)

Roughly 194 of the 403 lines sit inside `#[cfg(windows)]` blocks, and the file has one unguarded
Win32 call that breaks the Linux build outright.

- **`user_locale` (356-364) has no `cfg` guard.** Line 357 is
  `use windows::Win32::Globalization::GetUserDefaultLocaleName;` inside a plain `pub fn`. The
  nearest preceding `#[cfg(windows)]` is at line 327 and belongs to a test function. Its only caller is `app.rs` line 139, on first run, to
  pick the interface and dictation language, so that is where the build actually fails. Linux needs
  `LANG`, `LC_ALL` or the equivalent, normalised to a BCP-47 tag.
- **`system32` (74-77) has no `cfg` guard either, and it compiles.** It reads `SystemRoot`, falls
  back to `"C:\\Windows"`, and tests for `vulkan-1.dll` (69) and `nvcuda.dll` (70). On Linux it
  returns `false` for both, so `vulkan_runtime` and `cuda_driver` are always false, and
  `should_switch_to_gpu` (56) can never return true, because line 61 requires `vulkan_runtime`.
  The result is a Linux build that always runs on the processor without saying so. Linux needs a
  probe for `libvulkan.so.1` and `libcuda.so.1`.
- **`gpus` (97-124)** enumerates adapters through `CreateDXGIFactory1` and `EnumAdapters1`, skips
  software adapters, and maps `VendorId` to a name at 114-119. The `#[cfg(not(windows))]` arm at
  126-129 returns an empty vector, which is a stub, so `best_gpu` (30) is always `None`.
- **`ram_mb` (79-89)** uses `GlobalMemoryStatusEx`. The `#[cfg(not(windows))]` arm at 91-94
  returns `0`, another stub. Linux needs `/proc/meminfo` or `sysinfo`.
- **`gpu_memory` (155-160), `card` (164-188), `adapter_dedicated_usage_mb` (212-215) and
  `pdh_sum_mb` (220-288)** read live VRAM through PDH performance counters. The comment at 149-154
  records why: DXGI reported 7,249 MB free on a card `nvidia-smi` showed with 1,897 MB free. The
  `#[cfg(not(windows))]` arms at 205-208 and 290-293 return `None`, stubs again. Linux needs NVML
  for NVIDIA and sysfs or DRM for AMD and Intel; a single cross-vendor counter such as the PDH one
  is **not verified** to exist.

`best_gpu` (30), `engine_threads` (36), `should_switch_to_gpu` (56) and `summary` (40) are pure
and portable.

### `context.rs` (216 lines)

Two separate problems in one small file.

The `uia` module (145-192) is guarded by `#[cfg(windows)]` at line 145 and has **no
`#[cfg(not(windows))]` counterpart at all**. `inspect_focus` (162) spawns a fresh thread so the COM
apartment never leaks, calls `CoInitializeEx` (164), creates an `IUIAutomation` instance (165),
takes `GetFocusedElement` (166), and reads `CurrentIsPassword` (167), a `TextPattern` document
range (172-174) and a `ValuePattern` value (180-183). Its single caller is `pipeline.rs` line 410,
inside that file's own `#[cfg(windows)]` block. Linux needs AT-SPI2 over D-Bus for the same two
jobs: deciding whether the focused control is a password field, and reading a little text near the
caret when Context Awareness is on. AT-SPI exposes `STATE_PROTECTED` and a password role, but
coverage depends on the toolkit, and Electron applications require accessibility to be turned on
explicitly. Whether AT-SPI answers reliably across GTK, Qt and Electron on Wayland is **not
verified**.

The second problem is quieter and is a security regression. `SENSITIVE_PROCESSES` (55-58) lists
thirteen process names, every one carrying a `.exe` suffix, and line 63 matches them with exact
equality. `classify_process` (60-97) matches the rest of the table the same way at line 66. On
Linux a process is named `keepassxc`, so nothing in the sensitive list will ever match, the
category falls through to `AppCategory::Unknown` at line 95, and the refusal that protects
password managers **fails open** without any error. `classify_browser` (99-113) matches on window
title and needs no change. Linux needs a process-name table without suffixes, and the sensitive
check is a correctness requirement, and it should be treated as one.

### `jobobject.rs` (54 lines)

The mechanism is Win32 and the guarantee is needed on Linux, which is why this counts as a twin
and not as a Windows-only file. `mod imp` (5-47) creates one job object with `CreateJobObjectW`
(20), sets `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` (22) through `SetInformationJobObject` (23), and
`assign` (36) puts each spawned engine process into it with `AssignProcessToJobObject` (40). The
effect is that the whisper.cpp server dies with the app however the app dies, so it never lingers
holding the GPU. The caller is `asr/whisper_server.rs` lines 141-144, itself `#[cfg(windows)]`
gated. `assign_to_app_job` (49) has a `#[cfg(not(windows))]` no-op at 52-53.

On Linux, `kill_on_drop(true)` at `asr/whisper_server.rs` line 134 already covers an orderly exit,
and it does not cover a hard crash of the app. Candidate mechanisms for the missing half:
`prctl(PR_SET_PDEATHSIG, SIGKILL)` in a `pre_exec` hook on the child, a dedicated process group
killed on shutdown, or a cgroup. `PR_SET_PDEATHSIG` fires when the parent **thread** dies, which in a
tokio runtime can happen while the process is still alive, and that is a known trap. The correct
shape here is **not verified**.

## The eight with small isolated Windows bits

### `pipeline.rs` (1183 lines)

Genuinely almost all orchestration. Its only direct Windows code is the `#[cfg(windows)]` block at
lines 407-429, which is 23 lines: the UIA password check, capped at 400 ms so a slow answer from
Chromium cannot freeze the pipeline while the user talks, and the nearby-text read at 426-428.
Everything else reaches Windows through other modules: `insertion::capture_target` (352),
`overlay::show` with `ctx.target.hwnd` (358, 1110, 1124), `insertion::window_alive` (867),
`insertion::copy_only` (876, 898, 901, 1112), `insertion::foreground_hwnd` and `restore_focus`
(897), `insertion::type_text` (902), `insertion::paste` (903, 905, 1127), and
`hotkey::reset_pressed_state` (503, 554). It also reads `ctx.target.elevated` at 891 and 929 to
decide that an elevated target gets clipboard-only treatment. Once `insertion.rs` and `context.rs`
have Linux twins behind the same APIs, this file needs the 23-line block guarded for the new
platform and nothing else. The `hwnd: isize` field it threads through is an opaque window handle
and would carry an X11 `Window` id or a Wayland surface token equally well.

### `commands.rs` (1183 lines)

The bulk is database, settings and JSON glue. Four Windows-bound areas:
`open_data_folder` (597-600) and `open_logs_folder` (602-606) both spawn
`std::process::Command::new("explorer")` with no `cfg` guard, so they compile on Linux and fail at
runtime; `tauri_plugin_opener` is already registered and would serve. `record_shortcut` (672-738)
is itself a small Windows implementation of shortcut capture: hardcoded virtual-key numbers at
line 685, VK ordering at 709-715, and a VK-to-name map at 718-727 including `"Win"`. The
`hook_rehooks` diagnostic (625) and `recent_keys` (800-802) both surface hook-watchdog state that
has no Linux meaning yet. Lines 73-83 in `save_settings` receive the autostart failure described
under `app.rs` and write `autostart = false` back to disk.

### `settings.rs` (523 lines)

The default chords at lines 78-80 are `Ctrl+Win`, `RAlt` and `Shift+LAlt+Z`, parsed by `hotkey.rs`
into Windows virtual-key codes. Super is normally claimed by the Linux compositor and AltGr is
load-bearing for many Linux layouts, so this is the one place in the file that genuinely wants a
`cfg` arm. The five-attempt retry loop in `load` (355-383) was written for the Windows sharing
violation raised when antivirus holds `settings.json`, and it is harmless on Linux. The BOM strip
in `parse` (388) is worth keeping for files copied across from Windows. `save` (437-452) fsyncs
the temp file and renames without setting a mode or fsyncing the parent directory; Linux should
add 0600 and consider the directory fsync.

### `asr/whisper_server.rs` (452 lines)

`find_runtime_exe_for` (376-403) is the only place the engine binary is located, and line 402
hardcodes `whisper-server.exe`. The candidate directories at 380-393 include `vendor/whisper/
Release`, which is the MSVC multi-config layout. `start` carries two guarded blocks:
`CREATE_NO_WINDOW` at 129-133 (correctly gated, since tokio only exposes `creation_flags` on
Windows) and the job-object assignment at 141-144. Line 90 puts `whisper-server.exe` in a message
the user reads. Everything else already works on Linux: the CLI flags at 102-123 are the same on
both platforms, the readiness probe at line 201 is a plain TCP connect to `127.0.0.1`, and
`transcribe` is HTTP multipart with JSON.

### `app.rs` (429 lines)

Two chains matter. First, `installed_marker` (74-76) treats `uninstall.exe` as the proof that this
is the installed copy, `is_installed_copy` (78-80) therefore returns false for every Linux install
shape, `set_autostart` (54-61) returns `false` before reaching `manager.enable()` at 63, and
`commands.rs` 73-83 then forces `autostart = false` and persists it. The net effect on Linux is an
autostart switch that cannot be turned on and silently reverts itself. Second, `build_tray` (356
onwards) bails at line 361 when `default_window_icon()` is `None` and propagates any
`TrayIconBuilder` error at 419; `build` passes that up at line 322 and `lib.rs` 107-112 turns it
into a refusal to start. On Linux a missing appindicator would therefore stop the app from
launching at all, which argues for making the tray non-fatal. Whether Linux tray backends deliver
the left-click event handled at 409-418 is **not verified**.

### `paths.rs` (423 lines)

This file is in better shape than the brief's question implies. It hardcodes neither AppData nor
any MSIX layout. `config_dir` (271-277) is built from `dirs::config_dir()` and `local_dir`
(279-285) from `dirs::data_local_dir()`, which resolve to `%APPDATA%` and `%LOCALAPPDATA%` on
Windows and to `$XDG_CONFIG_HOME` and `$XDG_DATA_HOME` on Linux with no change. On top of those
two roots it places `settings.json` (300), `fuckyouflow.db` (304-332), `models` (334), `runtime`
and `runtime/whisper` (338, 342), `logs` (346), `temp` (350), `recovery` (354) and `audio` (359).
`bundled_dir` (296) is set at startup from the installer's resource directory. The module doc at
lines 3-4 names `%APPDATA%` and `%LOCALAPPDATA%` in prose and should be reworded. The one function
with a Windows assumption in code is `is_installation` (30-32), which looks for `uninstall.exe`,
`fuckyouflow.exe` or `lalia.exe`; its purpose is to stop the folder rename logic from adopting the
installation directory, which on Windows sits one name away under `%LOCALAPPDATA%`. On Linux the
program lives in `/usr` or `/opt`, so the guard is inert and harmless. The whole `Lalia` to
`FuckYouFlow` migration (`adopt` at 148-188, `take_orphan_histories` at 75-94,
`set_aside_folders` at 101-120) is dead weight on a Linux build, because no such folders exist
there, and it is safe to leave in place.

### `lib.rs` (213 lines)

`say_it_cannot_start` has a `#[cfg(windows)]` arm (33-64) that appends the reason to
`logs/panic.log` (36-45) and then shows a `MessageBoxW` (57-62), and a `#[cfg(not(windows))]` arm
(66-70) that is a working implementation and no stub: it logs and writes to stderr. Two
things are lost on Linux there. A windowed application launched from a desktop icon has no
terminal, so the user sees nothing, and the `panic.log` write is dropped as well.
`tauri_plugin_dialog` is already registered at line 104 and could carry the message. Three plugins
have platform-sensitive behaviour that is **not verified** here: `single_instance` (96), `updater`
(105) and `autostart` (106).

### `overlay.rs` (94 lines)

`work_area_for` (61-76) is the only Windows code: `MonitorFromWindow` with
`MONITOR_DEFAULTTOPRIMARY` and `GetMonitorInfoW`, returning the work area of the monitor holding
the target window. The `#[cfg(not(windows))]` arm at 78-81 returns `None`, which is a stub, and
`place` (83-93) then falls back to a hardcoded `(0, 0, 1920, 1040)` rectangle at line 85. On Linux
that means the floating pill lands in the wrong place on any screen of a different width and on
every secondary monitor. Tauri's own `current_monitor` and `available_monitors` are cross-platform
and would cover most of this; a true work area that excludes panels and docks needs
`_NET_WORKAREA` on X11, and the Wayland equivalent is **not verified**.

## Notes on the portable files

Five of the seventeen deserve a line even though they need no code change.

- **`db.rs` (1014)** sets no file mode. At a typical umask of 022 the history database, its
  backups (249) and the migration copies (367) land at 0644 and become readable by every local
  user. The Windows AppData ACL covered this implicitly. A Linux build should create the data
  directory 0700 and the database 0600.
- **`audio.rs` (911)** uses `cpal::default_host()` (219, 235) with no platform assumption and
  handles F32, I16 and U16 (370, 382, 395). It compiles and runs on Linux. A device name stored in
  settings on Windows will never match an ALSA description, so `find_device` (240) will log the
  warning at 245 and fall back to the default input.
- **`asr/openai_compat.rs` (103)** is the only user of the `keyring` crate, at lines 25, 30, 33,
  39 and 43. `Cargo.toml` enables the feature `windows-native-keyring-store` without
  `default-features = false`, so keyring's own target gating still selects the zbus Secret Service
  store on Linux. `Cargo.lock` confirms `secret-service`, `zbus` and `dbus` are already resolved.
  This is therefore a runtime requirement (a Secret Service provider such as gnome-keyring must be
  running) and no build blocker, and a future cleanup that adds `default-features = false`
  would break it silently.
- **`engine.rs` (304)** compiles on Linux with no change, and it is the one portable file that
  calls into a twin module: `crate::hw::detect()` at line 30 and `crate::models::find_model` at
  121, 124 and 180. Both exist on non-Windows, so there is no compile break. `choose_backend`
  (40-42) reads `hw.vulkan_runtime` and `hw.cuda_driver`, which the `hw.rs` stubs report as false
  on every Linux machine, so this file will always select the CPU backend until `hw.rs` has its
  twin. No other portable file calls into `insertion`, `hotkey`, `hw`, `context`, `jobobject` or
  `models`, verified by grep across the tree.
- **`scratch.rs` (55)** calls `show`, `unminimize` and `set_focus` at lines 50-52. Wayland
  compositors commonly refuse focus stealing from a background application. This window is the
  fallback the user sees when insertion failed, so losing the raise would reproduce the original
  complaint the module was written to answer. **Not verified** on a Linux machine.

## Counted totals

All figures are from `wc -l` over `src-tauri/src/**/*.rs`. The four verdict groups are disjoint
and sum to the total.

| Group | Files | Lines | Share |
|---|---|---|---|
| PORTABLE | 17 | 4,414 | 35.7 % |
| MOSTLY PORTABLE | 8 | 4,500 | 36.4 % |
| NEEDS A LINUX TWIN | 6 | 3,438 | 27.8 % |
| WINDOWS ONLY | 0 | 0 | 0 % |
| **Total** | **31** | **12,352** | **100 %** |

Portable files, 4,414 lines: `db.rs` 1014, `audio.rs` 911, `cleanup/deterministic.rs` 380,
`cleanup/questions.rs` 322, `engine.rs` 304, `cleanup/dictionary.rs` 242, `audiofile.rs` 198,
`journal.rs` 186, `logging.rs` 185, `cleanup/divergence.rs` 120, `cleanup/mod.rs` 114,
`learning.rs` 108, `asr/openai_compat.rs` 103, `cleanup/snippets.rs` 90, `asr/mod.rs` 76,
`scratch.rs` 55, `main.rs` 6.

Mostly portable files, 4,500 lines: `commands.rs` 1183, `pipeline.rs` 1183, `settings.rs` 523,
`asr/whisper_server.rs` 452, `app.rs` 429, `paths.rs` 423, `lib.rs` 213, `overlay.rs` 94.

Files needing a Linux twin, 3,438 lines: `insertion.rs` 1194, `hotkey.rs` 927, `models.rs` 644,
`hw.rs` 403, `context.rs` 216, `jobobject.rs` 54.

Windows-only files: none. Every Windows-bound module in this tree implements a capability the
product still needs on Linux, so nothing here can simply be excluded from the build.

Within the twin group, the Windows-bound blocks measure as follows: `insertion.rs` `mod win` is
1,063 lines of its 1,194; `hotkey.rs` `mod win` is 361 of its 927; `hw.rs` has about 194 lines
across five `#[cfg(windows)]` blocks plus the unguarded `user_locale` and `system32`;
`context.rs` `uia` is 48 of its 216; `jobobject.rs` `mod imp` is 43 of its 54. In `pipeline.rs`,
the largest of the mostly-portable files, the directly Windows-bound block is 23 lines of 1,183.

Version audited: `Cargo.toml` reads `version = "0.9.6"`. No git command was run for this
document, so the exact commit is **not verified** here.
