# Linux Port Gaps

Fuck You Flow is a Windows app in its bones. Roughly eleven capabilities carry the product, and eight of them are reachable on Linux with ordinary effort. The two that define the product, holding a key anywhere in the system and pushing text into somebody else's window, plus a third one that guards the user's passwords, are the whole problem. On X11 all three are solvable with well understood APIs. On Wayland, two capabilities have no portable protocol at all: learning which window has focus, and placing a click-through always-on-top pill at a chosen screen coordinate. The XDG GlobalShortcuts portal cannot express the app's actual shortcut either, because the freedesktop shortcuts specification defines a trigger as modifiers plus a key identifier, with no left/right distinction and no documented modifier-only form, while the app's default hands-free key is a bare right Alt (`settings.rs:79`). The recommendation below is therefore X11 first, on an evdev/uinput foundation that survives the eventual move to Wayland.

One correction to the brief before the detail. The app does not bind "hold Right Alt" as push-to-talk. `settings.rs:78-81` sets `push_to_talk: "Ctrl+Win"`, `hands_free: "RAlt"` and `tap_toggles_hands_free: true`. Right Alt is the hands-free key, which a tap toggles. Both paths still need key-up, because `HotkeyEvent::Released` (`hotkey.rs:30-42`) drives the push-to-talk stop and because `Cancelled` fires when a character key arrives inside an 800 ms window (`hotkey.rs:285`) to tell AltGr typing apart from dictation. The analysis below treats press-and-release as the binding requirement.

## Summary table

| Capability | Windows today | X11 option | Wayland option | Difficulty |
|---|---|---|---|---|
| 1. Global hotkey, held, with release | `WH_KEYBOARD_LL` + `WH_MOUSE_LL` low-level hooks on a dedicated thread, re-installed every second (`hotkey.rs:727`, `738`, `762`) | `XGrabKey` / XRecord, or evdev | evdev + `EVIOCGRAB`; GlobalShortcuts portal has no documented way to express a bare Right Alt | HARD |
| 2. Insert text into the focused window | Clipboard delayed-render promise + synthetic Ctrl+V, fallback Unicode `SendInput` (`insertion.rs:798`, `1033`) | XTEST `XTestFakeKeyEvent` + X11 clipboard | uinput virtual keyboard, or libei via RemoteDesktop portal; `zwp_virtual_keyboard_v1` absent on KWin and Mutter | HARD |
| 3. Read focused window and password field | `GetForegroundWindow` + `QueryFullProcessImageNameW` + UI Automation `CurrentIsPassword` (`insertion.rs:1132`, `context.rs:167`) | `_NET_ACTIVE_WINDOW` + `WM_CLASS` + `_NET_WM_PID`, plus AT-SPI2 for the password state | No portable protocol for focus identity; AT-SPI2 still gives the password state | BLOCKED ON WAYLAND |
| 4. Click-through always-on-top overlay | Tauri window flags plus absolute placement on the target's monitor (`tauri.conf.json:26-42`, `overlay.rs:83-93`) | `_NET_WM_STATE_ABOVE` + `set_position`, both work through tao | `set_position` and `always_on_top` silently no-op in Tauri; needs `wlr-layer-shell`, which Tauri does not expose | BLOCKED ON WAYLAND |
| 5. System tray icon | `TrayIconBuilder` (`app.rs:348`) | StatusNotifierItem via libayatana-appindicator | Same, plus a GNOME extension | MEDIUM |
| 6. Autostart on login | `tauri-plugin-autostart` writing a Run entry (`app.rs:54`, `lib.rs:106`) | XDG autostart `.desktop`, plugin supports Linux | Same | EASY |
| 7. Single instance | `tauri-plugin-single-instance` (`lib.rs:96`) | D-Bus, plugin supports Linux | Same | EASY |
| 8. Secret in the OS keychain | `keyring` v4 with `windows-native-keyring-store` (`Cargo.toml:39`, `asr/openai_compat.rs:25`) | Same crate, Secret Service feature | Same | EASY |
| 9. Microphone capture | cpal on WASAPI (`audio.rs:14`, `218`) | cpal ALSA or PipeWire feature | Same | EASY |
| 10. File locations | `dirs::config_dir` + `dirs::data_local_dir` (`paths.rs:271`, `279`) | Same calls resolve to XDG paths | Same | EASY |
| 11. GPU for whisper.cpp | Downloads a prebuilt CUDA zip (`models.rs:146`) | Prebuilt Linux tarball exists, no GPU asset published; GPU likely needs a source build | Same | MEDIUM |

---

## 1. Global hotkey capture while another app has focus

### What Windows does today

`hotkey.rs` installs a `WH_KEYBOARD_LL` hook and a `WH_MOUSE_LL` hook on a dedicated thread with its own message loop (`hotkey.rs:727` and `hotkey.rs:738`). The module header explains the choice: `RegisterHotKey` reports key-down only and swallows the combination for every other app, while push-to-talk needs the key-up, must ignore auto-repeat, and must not steal chords the user did not ask for (`hotkey.rs:1-9`).

Three details of that implementation matter for the port:

- A `SetTimer` fires every second and re-installs both hooks (`hotkey.rs:762`, `766`, `781`). This exists only because Windows silently removes a low-level hook whose callback overruns `LowLevelHooksTimeout`, with no notification and no way to ask whether the hook is still alive.
- `LALIA_INJECT_SIG` (`hotkey.rs:93`) is stamped into `dwExtraInfo` on every key the app injects, so the hook can ignore its own paste chords while still honouring keys injected by AutoHotkey or a remote desktop.
- `drop_altgr_companion` (`hotkey.rs:367`) exists because one press of the right Alt on an AltGr layout, including the Greek one, reaches Windows as a synthetic left Ctrl plus the right Alt.

### X11

X11 has no input privilege barrier between clients, so several routes work. `XGrabKey` binds a key combination and delivers both `KeyPress` and `KeyRelease`, and it can bind a bare modifier. The XRecord extension observes the whole key stream passively. `XGrabKey` delivers `KeyPress` and `KeyRelease` to the grabbing client for the duration of the grab ([XGrabKey, X.Org Xlib manual](https://www.x.org/releases/X11R7.7/doc/man/man3/XGrabKey.3.xhtml)), and the XRecord extension exists precisely to let one client observe the core protocol's device events ([Record extension library, X.Org](https://www.x.org/releases/X11R7.7/doc/libXtst/recordlib.html)). Either route reproduces today's behaviour, including release and auto-repeat filtering, without elevated privileges. That ease is also a security downgrade relative to Windows: X11 puts no privilege barrier between clients, so any client can watch every keystroke.

### Wayland

Two routes, and both have a caveat.

The **GlobalShortcuts portal** is the sanctioned one. It does expose both an `Activated` and a `Deactivated` signal, so release is in principle available, alongside `CreateSession`, `BindShortcuts`, `ListShortcuts` and `ConfigureShortcuts` ([portal documentation](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.GlobalShortcuts.html)). Three problems follow.

First, the app does not choose the key. The portal documentation states that binding "will typically result in the portal presenting a dialog showing the shortcuts and allowing users to configure the shortcuts", and the application supplies only an optional `preferred_trigger`. A product whose identity is "hold the right Alt" cannot guarantee that key.

Second, the trigger syntax probably cannot express it. The freedesktop shortcuts specification defines a shortcut as "a set of modifiers (namely CTRL, ALT, SHIFT, NUM and LOGO, as defined as XKB_MOD_NAME_* in xkbcommon-names.h) together with a key identifier, joined by a + sign" ([shortcuts specification](http://specifications.freedesktop.org/shortcuts/latest/)). Every example pairs modifiers with a key. There is no left/right distinction and no documented modifier-only form. Whether any implementation accepts a bare `ALT` in practice is **not verified**.

Third, implementation coverage is uneven and recently unstable. KDE has had an implementation for some time and Hyprland ships one; `xdg-desktop-portal-wlr` ships none, so Sway and Niri fail `BindShortcuts` with error code 5. GNOME gained one through `xdg-desktop-portal-gnome` merge request 208. On GNOME 50 with `xdg-desktop-portal` 1.21 the flow additionally requires a non-sandboxed app to call `org.freedesktop.host.portal.Registry.Register(app_id)` with a reverse-DNS id backed by an installed `.desktop` file, and Chromium's failure to make that call is an open bug ([claude-desktop-debian findings](https://github.com/aaddrick/claude-desktop-debian/blob/main/docs/learnings/wayland-global-shortcuts-portal.md), [GNOME issue 47](https://gitlab.gnome.org/GNOME/xdg-desktop-portal-gnome/-/work_items/47), [Chromium issue 404298968](https://issues.chromium.org/issues/404298968)). Whether any of these implementations actually emits `Deactivated` on physical key release for a held shortcut is **not verified**.

The **evdev route** reads `/dev/input/event*` directly. This sits below the display server, so it is identical on X11 and Wayland. The Rust `evdev` crate exposes both raw reading and `EVIOCGRAB` for an exclusive grab ([evdev crate](https://docs.rs/evdev), [Device](https://docs.rs/evdev/latest/evdev/struct.Device.html)). Two happy consequences for this codebase: raw keycodes arrive before XKB translation, so `KEY_RIGHTALT` is unambiguous and the synthetic-LCtrl pairing that `drop_altgr_companion` was written for disappears; and the one-second re-hook loop has no reason to exist, because nothing silently uninstalls a file descriptor. The cost is device permission, covered in the final section.

**Difficulty: HARD.** Press and release of a bare right Alt is available on both display servers through evdev, but only by taking a device-level permission the app does not need today, and the portal alternative cannot express the shortcut the product is built around.

## 2. Inserting text into the focused window of another application

### What Windows does today

`insertion.rs` layers three strategies (`insertion.rs:1-13`). The primary one publishes an empty delayed-render promise for `CF_UNICODETEXT`, sends Ctrl+V, and hands over the data only when the target asks through `WM_RENDERFORMAT` (`insertion.rs:445`), then restores the previous clipboard. The second types the text as Unicode through `SendInput` with `KEYEVENTF_UNICODE` (`insertion.rs:1033`, `1052`). The third leaves the text on the clipboard and says so.

The subtle part is delivery proof. `Shared::rendered_by_target` counts renders requested by the pid that had focus when Ctrl+V was sent (`insertion.rs:169`, `895`), because Windows offers no way to ask "did that application paste". That is what distinguishes `InsertOutcome::Pasted` from `InsertOutcome::PasteNotConsumed` (`insertion.rs:82-95`).

### X11

Near parity. XTEST `XTestFakeKeyEvent` sends synthetic key events to whichever client has focus, and the X11 selection mechanism is itself a delayed-render protocol: the owner advertises a selection and only materialises the data when a `SelectionRequest` arrives, and that event carries the `requestor` window. So the "did the target actually take it" measurement survives, which is unusual and worth protecting. The terminal case (`context.rs:122`, Ctrl+Shift+V) transfers unchanged, and X11 additionally distinguishes PRIMARY from CLIPBOARD, which is a behaviour question for later.

### Wayland

A client cannot synthesise input into another client by design. Three routes:

- **uinput.** Create a virtual keyboard device in the kernel and type through it. This is what `ydotool` does, and it works on Wayland and on the text console because the events enter below the compositor ([ydotool](https://github.com/ReimuNotMoe/ydotool)). It requires `/dev/uinput`, normally root-only, and a running `ydotoold`-style daemon for the type command.
- **libei through the RemoteDesktop portal.** The modern sanctioned path. libei carries logical input events and has integrated with the RemoteDesktop portal since version 1.17 in mid-2023; session persistence arrived in portal 1.21.0, so a `restore_token` with `persist_mode` 2 avoids re-prompting on every launch ([who-t on libei portal integrations](http://who-t.blogspot.com/2026/07/libei-integrations-in-xdg-remotedesktop.html), [RemoteDesktop portal](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html)). GNOME 45 and Xwayland 23.2 shipped libei support, and KWin exposes a private EIS D-Bus interface ([rustdesk discussion 4515](https://github.com/rustdesk/rustdesk/discussions/4515)).
- **`zwp_virtual_keyboard_v1`.** Available on wlroots compositors. Stock KWin does not expose it, which is why `wtype` fails on KDE Plasma Wayland ([Wayland Explorer protocol page](https://wayland.app/protocols/virtual-keyboard-unstable-v1), [KWtype](https://github.com/Sporif/KWtype)). Mutter does not expose it either. So this route covers Sway and Hyprland and leaves out both major desktops.

Whichever route is taken, the delivery proof degrades. Wayland gives no equivalent of "the target pid requested the render", so `InsertOutcome::PasteNotConsumed` may become unreportable and a failed paste may look identical to a successful one. Another voice-dictation project hit exactly this on GNOME Wayland ([OpenWhispr issue 240](https://github.com/OpenWhispr/openwhispr/issues/240)).

**Difficulty: HARD.** X11 is close to a direct translation, Wayland needs either a kernel device or a portal session, and the delivery-confirmation signal that the current code is carefully built around has no Wayland equivalent.

## 3. Reading the focused window

### What Windows does today

`capture_target` calls `GetForegroundWindow`, then `GetWindowThreadProcessId`, `GetWindowTextW` and `QueryFullProcessImageNameW` to produce a `Target` with hwnd, pid, process name, title and an `elevated` flag (`insertion.rs:1132-1150`, `1085`). `context.rs` classifies that process name into an `AppCategory` and derives punctuation and capitalisation style from it (`context.rs:60-126`). Separately, `context::uia::inspect_focus` spins a fresh COM thread, calls `IUIAutomation::GetFocusedElement` and reads `CurrentIsPassword` (`context.rs:162-167`); insertion is refused when it is true. `window_class` and `unusable_target` (`insertion.rs:678`, `695`) reject the desktop and the taskbar by window class.

Note one small but real leak risk: `SENSITIVE_PROCESSES` (`context.rs:55-58`) is entirely `.exe` names. A port that forgets to add `keepassxc`, `bitwarden`, `gnome-keyring`, `seahorse` and the rest silently loses the password guard.

### X11

Fully solvable. `_NET_ACTIVE_WINDOW` on the root window gives the focused window, `WM_CLASS` gives the application identity that `classify_process` needs, `_NET_WM_NAME` gives the title, and `_NET_WM_PID` gives the pid, from which `/proc/<pid>/comm` and `/proc/<pid>/status` yield the process name and the Uid that replaces the `elevated` check. `WM_CLASS` is the natural analogue of `unusable_class`. The password state comes from AT-SPI2, which is D-Bus based and exposes a password role and state on the focused element ([AT-SPI2, freedesktop](https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/), [atspi-2 API](https://valadoc.org/atspi-2/index.html)).

### Wayland

There is no protocol that lets an unprivileged client learn which window has focus. `wlr-foreign-toplevel-management` gives it on wlroots compositors only, and GNOME closed the D-Bus Eval workaround in GNOME 41 for security reasons ([KDE Discuss thread](https://discuss.kde.org/t/how-to-get-focused-window-title-in-a-python-or-bash-script-on-wayland/21361)). This is a deliberate design decision, so it is unlikely to be granted to ordinary apps.

The password half is better. AT-SPI2 runs over D-Bus, independent of the display server, so `Atspi.Role.PASSWORD_TEXT` and the corresponding state are readable on Wayland as well, with toolkit coverage as the caveat: GTK and Qt implement AT-SPI, Electron and some others implement it partially or not at all. AT-SPI also usually exposes the focused application's name, so one dependency partially covers this capability and helps capability 2. Whether AT-SPI's application name is reliable enough to replace `WM_CLASS` for style classification is **not verified**.

**Difficulty: BLOCKED ON WAYLAND.** Focused-window identity has no portable Wayland protocol and the major desktops have deliberately closed the workarounds, so per-app style and the app-name guard cannot be reproduced there, even though AT-SPI2 still covers password detection.

## 4. Click-through always-on-top overlay indicator

### What Windows does today

There is no `set_ignore_cursor_events` call anywhere in the codebase. The pill is configured purely by window flags in `tauri.conf.json:26-42`: `decorations: false`, `transparent: true`, `alwaysOnTop: true`, `skipTaskbar: true`, `focus: false`, `focusable: false`, `visibleOnAllWorkspaces: true`. So "click-through" today means only that the pill never takes focus; mouse events still land on it, as the module header states (`overlay.rs:1-2`).

Placement is the harder half. `overlay::place` calls `w.set_position` with an absolute `PhysicalPosition` computed from the work area of the monitor that holds the target window, obtained through `MonitorFromWindow` and `GetMonitorInfoW` (`overlay.rs:62-93`).

### X11

Works. `_NET_WM_STATE_ABOVE` provides always-on-top, absolute positioning is normal, and Xinerama or RandR gives per-monitor work areas. True click-through, if it is ever wanted, is the X Shape extension's input region. Nothing here is a research problem.

### Wayland

Both halves fail, and they fail silently, which is the dangerous part. Tauri's `set_position` and `always_on_top` are no-ops on Wayland because Wayland has no global coordinate system and gives clients no way to place their own toplevel ([tauri issue 14913](https://github.com/tauri-apps/tauri/issues/14913), [tao issue 1134](https://github.com/tauri-apps/tao/issues/1134), [tauri issue 3117](https://github.com/tauri-apps/tauri/issues/3117)). The `alwaysOnTop: true` already in `tauri.conf.json` would therefore do nothing, with no error raised. The correct Wayland mechanism is `wlr-layer-shell`, which is designed for panels and overlays, and Tauri does not expose it; it is also absent from Mutter. Running the app under Xwayland restores both behaviours, which is a genuine workaround and worth stating plainly.

**Difficulty: BLOCKED ON WAYLAND.** A surface cannot position itself or claim always-on-top under Wayland, Tauri's calls fail without reporting it, and the one protocol that would work is unavailable both in Tauri and in GNOME.

## 5. System tray icon

### What Windows does today

`build_tray` in `app.rs:348` constructs a single `TrayIconBuilder::with_id("main")` with a menu and a left-click handler, deliberately in code, because declaring it in `tauri.conf.json` as well makes Tauri create a second icon (`app.rs:345-347`). `drop_tray` removes it on every exit path to avoid a ghost icon (`app.rs:339`).

### X11 and Wayland

The same answer on both, because the tray is a D-Bus protocol, independent of the display server. Tauri uses StatusNotifierItem through `libayatana-appindicator`, so `libayatana-appindicator3-dev` on Debian-family and `libayatana-appindicator` on Arch become build and runtime dependencies ([tray-icon README](https://github.com/tauri-apps/tray-icon/blob/dev/README.md)). Two caveats: `libayatana-appindicator` is deprecated in favour of `libayatana-appindicator-glib` ([tray-icon issue 260](https://github.com/tauri-apps/tray-icon/issues/260)), and there is an open report of the icon never registering with `StatusNotifierWatcher` on KDE Plasma 6 Wayland ([tray-icon issue 336](https://github.com/tauri-apps/tray-icon/issues/336)). Stock GNOME has no tray and needs the AppIndicator extension.

**Difficulty: MEDIUM.** The code carries over unchanged, but it adds a native dependency, depends on a deprecated library, and on stock GNOME needs a user-installed extension before the icon appears at all.

## 6. Autostart on login

### What Windows does today

`lib.rs:106` registers `tauri-plugin-autostart`. `app.rs:54` wraps it with a rule that only the installed copy may claim the entry, because a build-folder copy once claimed it and then booted an unupdatable build (`app.rs:46-52`). `set_autostart` returns whether the entry now matches, so a switch that did nothing never shows as on.

### X11 and Wayland

Identical. The plugin lists Linux as supported ([autostart plugin README](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/autostart/README.md)). On Linux this is an XDG autostart `.desktop` file in `~/.config/autostart`, though the plugin README does not state the mechanism, so the exact file it writes is **not verified**. The `is_installed_copy` guard needs rewriting for Linux install layouts, where the notion of "the installed copy" differs between a `.deb`, an AppImage and a Flatpak.

**Difficulty: EASY.** The plugin already supports Linux and the only work is redefining what counts as the installed copy.

## 7. Single-instance enforcement

### What Windows does today

`lib.rs:96` registers `tauri-plugin-single-instance` with a callback that shows, unminimises and focuses the main window when a second launch is attempted.

### X11 and Wayland

Identical. The plugin lists Linux as supported and uses D-Bus there; the README notes a `DBUS_ID` builder option for Flatpak and Snap when the package id differs from the Tauri identifier ([single-instance plugin README](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/single-instance/README.md)). The current identifier is `gr.luram.lalia` (`tauri.conf.json:6`), which is already reverse-DNS and therefore fine as a D-Bus name. The `w.set_focus()` in the callback may be ignored under Wayland, where focus stealing is restricted, so the second launch might raise no window; that specific behaviour is **not verified**.

**Difficulty: EASY.** Supported by the existing plugin with no code change.

## 8. Storing the API key in the OS keychain

### What Windows does today

`asr/openai_compat.rs:25-45` uses `keyring::Entry::new("Lalia", "openai_compatible_api_key")` for store, delete, presence check and read. `Cargo.toml:39` pins `keyring = { version = "4", features = ["windows-native-keyring-store"] }`.

### X11 and Wayland

Identical, and it is a one-line change. keyring 4.2.0 offers `dbus-secret-service-keyring-store` and `zbus-secret-service-keyring-store` for the freedesktop Secret Service, plus `linux-keyutils-keyring-store` ([keyring on docs.rs](https://docs.rs/keyring/latest/keyring/)). Secret Service is the right default, backed by gnome-keyring or KWallet. Two operational notes: on a headless or minimal session no Secret Service provider may be running, so the code must report a missing provider as its own distinct condition, separate from a missing key; and the service name `Lalia` (`asr/openai_compat.rs:9`) is a leftover from the old product name that will be visible in Seahorse and KWalletManager.

**Difficulty: EASY.** A feature-flag swap, with an error path to add for a session that has no Secret Service provider.

## 9. Microphone capture

### What Windows does today

`audio.rs` uses cpal over WASAPI (`audio.rs:1`, `14`). It enumerates with `host.input_devices()` (`audio.rs:218-232`), prefers an F32 configuration at the target rate (`audio.rs:305-318`), and handles `ErrorKind::Xrun`, `DeviceChanged` and `RealtimeDenied` (`audio.rs:341-358`). A watchdog re-opens the warm stream when it stops delivering (`app.rs:307-315`).

### X11 and Wayland

Identical, since audio is unrelated to the display server. cpal 0.18 supports Linux and needs ALSA development headers at build time (`libasound2-dev` on Debian, `alsa-lib-devel` on Fedora) even when the runtime is PipeWire ([cpal README](https://github.com/RustAudio/cpal/blob/master/README.md)). One real trap: when PipeWire or PulseAudio is running it holds the ALSA default device exclusively, and a second stream opening `default` through the ALSA host fails with `DeviceBusy`. cpal 0.18 offers native `pipewire` and `pulseaudio` features, which are the better choice on a modern desktop ([cpal on docs.rs](https://docs.rs/cpal)). Given that this app keeps a stream warm while idle (`app.rs:311`), the exclusive-default problem is likely to bite in practice. Under Flatpak the microphone also needs the Device portal, which is one more reason Flatpak is a poor fit here.

**Difficulty: EASY.** cpal already abstracts the host; the work is picking the PipeWire feature and adapting the error mapping.

## 10. Standard file locations

### What Windows does today

`paths.rs:271` and `paths.rs:279` call `dirs::config_dir()` and `dirs::data_local_dir()`, giving `%APPDATA%\FuckYouFlow` for `settings.json` and `fuckyouflow.db`, and `%LOCALAPPDATA%\FuckYouFlow` for models, the whisper runtime, logs, temp audio and recovery (`paths.rs:3-4`, `334-360`). There is also a folder-rename migration from the older "Lalia" and "Fuck You Flow" names (`paths.rs:27`, `76-95`).

### X11 and Wayland

Identical, and mostly free, because the code already goes through `dirs`. On Linux `config_dir()` resolves to `$XDG_CONFIG_HOME` or `$HOME/.config`, and `data_local_dir()` to `$XDG_DATA_HOME` or `$HOME/.local/share` ([dirs README](https://github.com/dirs-dev/dirs-rs)). So the two roots stay distinct and the roaming/local split survives as `~/.config/FuckYouFlow` and `~/.local/share/FuckYouFlow`.

Three adjustments. `dirs::data_local_dir()` and `dirs::data_dir()` return the same path on Linux while they differ on Windows, so any code that relies on them differing needs checking. Logs and temp audio belong in `$XDG_STATE_HOME` (`~/.local/state`) and `$XDG_CACHE_HOME` (`~/.cache`) by XDG convention, so those two paths move. And the "Lalia" rename migration is Windows-only history that a fresh Linux build should skip.

**Difficulty: EASY.** The `dirs` crate already does the mapping; only the log and cache placement is a judgement call.

## 11. GPU acceleration for whisper.cpp

### What Windows does today

`models.rs:144-149` downloads `whisper-cublas-12.4.0-bin-x64.zip` from the whisper.cpp release b4938, verifies it and extracts a fixed list of files including `whisper-server.exe`, `ggml-cuda.dll`, `cublas64_12.dll` and friends (`models.rs:476-489`), using the `tar.exe` that ships with Windows 10 (`models.rs:529`). `cuda_driver_present()` tests for `nvcuda.dll` in System32 (`models.rs:560`). `hw.rs:98` enumerates adapters and VRAM through DXGI `EnumAdapters1`, and `preferred_installed_model` picks large-v3 only above 3000 MB of VRAM (`models.rs:546`). The engine runs `whisper-server` as a managed child with `--flash-attn` and `--no-gpu` when the GPU is off (`asr/whisper_server.rs:101-118`).

### X11 and Wayland

Identical, since this is compute work, independent of the display server. Checked directly against the GitHub releases API on 12 September 2026, release b5130 does publish `whisper-bin-ubuntu-x64.tar.gz` and `whisper-bin-ubuntu-arm64.tar.gz` alongside the Windows zips ([whisper.cpp releases](https://github.com/ggml-org/whisper.cpp/releases)). There is no `whisper-cublas-*-linux` or Vulkan Linux asset in that list, so the prebuilt Linux tarball appears to be CPU-only. Whether it carries any GPU backend is **not verified** from the asset name alone.

That means the whole `install_runtime` strategy changes shape on Linux. Either build whisper.cpp from source with `GGML_CUDA` or `GGML_VULKAN` at packaging time and ship the result, or ship CPU-only and treat GPU as an opt-in local build. Vulkan is the more portable choice because it covers NVIDIA, AMD and Intel from one build, which matters more on Linux than on Windows. The two detection helpers need Linux replacements: `nvcuda.dll` becomes a check for `libcuda.so.1`, and DXGI VRAM enumeration becomes sysfs or `vulkaninfo`. The `CREATE_NO_WINDOW` creation flag (`asr/whisper_server.rs:131`) is Windows-only and simply drops away.

**Difficulty: MEDIUM.** No blocker anywhere, but the download-a-prebuilt-runtime design has no Linux equivalent for GPU builds, so packaging has to grow a compile step or accept CPU-only.

---

## Recommended first target

**Target X11 first, specifically Ubuntu LTS or Linux Mint on an X11 session, and build the input layer on evdev and uinput from day one.**

### Why X11 comes first

The deciding argument is capability 3, and it settles the question before the hotkey debate even begins. This app reads the focused application's identity for two purposes: choosing punctuation and capitalisation style per app (`context.rs:60-126`), and refusing to type into a password field or a password manager (`context.rs:55-58`). Wayland offers no portable protocol for focused-window identity, and GNOME closed its D-Bus workaround deliberately. So a Wayland-portable build would have to ship without per-app style and with a weaker password guard, on a product whose value is that it behaves correctly in whatever window you are in. Capability 4 adds a second, independent blocker: the overlay cannot position itself or stay on top, and Tauri fails at both without raising an error.

X11 gives all eleven capabilities with known APIs. It is also still the default or a one-click login option on the distributions most likely to host a paid desktop tool of this kind. The honest caveat is that this is a shrinking target: Ubuntu, Fedora and RHEL have all moved to Wayland by default, so X11 is a first target, with Wayland as the eventual destination.

### Why the foundation should be evdev and uinput

Choose the foundation by what survives the move to Wayland.

An `XGrabKey` plus XTEST layer is the shortest path to a working X11 build, and every line of it is thrown away when Wayland becomes mandatory. An evdev reader plus a uinput writer works identically under X11, under Wayland and on a bare text console, because it sits below the display server in the kernel. Writing it once means the hardest two capabilities carry over unchanged, and the remaining Wayland work reduces to capabilities 3 and 4, which are blocked regardless of what the input layer is made of.

The evdev foundation also deletes code. Raw keycodes arrive before XKB translation, so `KEY_RIGHTALT` is unambiguous and the AltGr synthetic-left-Ctrl workaround (`hotkey.rs:367`) and probably the 800 ms typing window (`hotkey.rs:285`) can shrink or disappear. A uinput device the app owns lets it filter its own injected events by device identity, which replaces the `LALIA_INJECT_SIG` `dwExtraInfo` marker (`hotkey.rs:93`) with something cleaner. And the one-second re-hook loop (`hotkey.rs:762`) has no reason to exist at all.

The portals stay on the roadmap as the second input backend, behind a trait, for the eventual Wayland build: GlobalShortcuts for capture where the user is willing to assign a key that the specification can express, and libei through RemoteDesktop with `persist_mode` 2 and a stored `restore_token` for injection. Building the input layer behind a trait from the start costs little and makes that swap a contained backend change.

### What this implies for permissions and packaging

- **Device access.** Reading `/dev/input/event*` and writing `/dev/uinput` both need permission beyond a normal user. Ship a udev rule of the shape `KERNEL=="uinput", SUBSYSTEM=="misc", TAG+="uaccess", OPTIONS+="static_node=uinput"` plus `input` group membership for the user, installed by a `.deb` or `.rpm` postinst script or documented as a one-time setup step ([ydotool](https://github.com/ReimuNotMoe/ydotool), [ydotool fork with udev notes](https://github.com/SteveCharleston/ydotool)). Group membership takes effect at next login, so the installer has to say so or the first run looks broken.
- **Distribute as .deb and .rpm, plus an AppImage.** These can run a postinst and can drop a udev rule.
- **Flatpak is effectively ruled out for this design.** Granting `/dev/uinput` inside a Flatpak needs `--device=all`, which discards the sandbox and is rejected by Flathub reviewers for good reason. The microphone would additionally go through the Device portal. A Flatpak build would have to use the portal input backend, which brings back every limitation in capability 1. State that in the packaging plan, so it surfaces long before submission.
- **Security posture worsens relative to Windows.** Today the app benefits from Windows UIPI, and it refuses to type into elevated windows (`insertion.rs:1085`, `1103`). On X11 there is no input privilege barrier at all, and an evdev grab means the process can read every keystroke on the machine. That deserves a line in `THREAT-MODEL.md` and probably a visible statement to users.
- **Native build dependencies to add:** `libasound2-dev` for cpal, `libayatana-appindicator3-dev` for the tray, plus the usual Tauri Linux set (`libwebkit2gtk`, `libgtk-3-dev`).
- **Rewrite the sensitive-application list.** `SENSITIVE_PROCESSES` (`context.rs:55-58`) is all `.exe` names and will match nothing on Linux, silently removing the password guard.
