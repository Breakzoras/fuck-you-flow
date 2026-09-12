# Linux port: the map

Branch `port/linux`, cut from `fix/audit-2026-09-11` at commit b403881, version 0.9.6.
Written 2026-09-12. This branch holds analysis only. No source file was changed.

## What this is

A survey of what stands between Fuck You Flow and a Linux build, measured against the code as it
exists today. It exists so the port can be prepared in small steps over time, with every decision
made against counted facts.

## The four documents

| Document | Covers |
|---|---|
| [LINUX-PORT-CORE.md](LINUX-PORT-CORE.md) | Every Rust file, classified by portability |
| [LINUX-PORT-BUILD.md](LINUX-PORT-BUILD.md) | Cargo, Tauri config, whisper.cpp, bundling, updater, CI |
| [LINUX-PORT-GAPS.md](LINUX-PORT-GAPS.md) | Each operating system capability, with X11 and Wayland answers |
| [LINUX-PORT-FRONTEND.md](LINUX-PORT-FRONTEND.md) | The TypeScript layer |

## The shape of the problem, in counted lines

31 Rust files, 12,352 lines. Verified with `find src-tauri/src -name "*.rs" -exec wc -l {} +`.

| Group | Files | Lines | Share |
|---|---|---|---|
| Portable as they stand | 17 | 4,414 | 36% |
| Mostly portable, small guarded bits | 8 | 4,500 | 36% |
| A Linux twin must be written | 6 | 3,438 | 28% |
| Windows only, drop on Linux | 0 | 0 | 0% |

The six files that need a Linux twin are `insertion.rs` (1,194), `hotkey.rs` (927),
`models.rs` (644), `hw.rs` (403), `context.rs` (216) and `jobobject.rs` (54).

Nothing in the app exists purely as a Windows artifact, so no module gets deleted. Roughly seven
lines in ten already work anywhere. The work ahead is a port with one hard edge, and the edge is
small enough to name file by file.

## The three walls

### 1. Putting the text into the other window

`insertion.rs` is 1,194 lines, of which 1,063 sit inside `mod win`. The paste path is a nine step
Win32 sequence built on a delayed clipboard promise plus `SendInput`, and it confirms success by
watching which process comes to collect the clipboard data. That confirmation signal has no
counterpart on X11 or on Wayland. Today only 1 of its 9 public functions has a non-Windows arm, so
12 call sites fail to compile the moment the target changes.

### 2. Catching the key while another app has focus

`hotkey.rs` installs a `WH_KEYBOARD_LL` hook that swallows the chord, and a watchdog re-installs a
fresh hook every second because Windows silently drops it. On Linux `install()` is already gated
down to a silent no-op. No Linux mechanism offers global capture, key-up delivery and suppression
together while leaving AltGr typing intact, which is why the input layer has to be rebuilt from
the device level.

Correction to an assumption worth recording: hold-to-talk is `Ctrl+Win`, and Right Alt is the
hands-free toggle with a 280 ms tap (`settings.rs:78-81`).

### 3. Two capabilities the modern Linux desktop refuses

Reading which window has focus and whether the focused field is a password box, and letting the
pointer pass through the transparent part of the overlay, are both available on X11 and both
denied to an ordinary client on Wayland. These are design limits of the display server.

`context.rs` carries a safety consequence that goes beyond inconvenience. `SENSITIVE_PROCESSES`
(line 55) matches process names by exact equality against strings such as `keepassxc.exe`, and the
comparison at line 63 is `p == *s`. On Linux the process is `keepassxc`, so the match never fires,
every application falls through to `Unknown`, and the refusal to dictate into a password manager
quietly stops working. It fails open. Any Linux build has to close this before it ships.

## Build and packaging

The speech engine has no Linux build path at all. `scripts/build-vulkan.ps1` is the only producer
and it hardcodes `C:\Claude Projects\lalia\vendor\...` and `C:\VulkanSDK\current` (lines 10-12),
locates `cmake.exe` inside Visual Studio (line 13), and gathers `.exe` and `.dll` files (line 32).
The whisper.cpp tree is gitignored and absent, with no clone script in the repo.

There is also no CI. `.github` does not exist, and every release step in `docs/RELEASING.md:32-57`
is typed by hand on one Windows machine that holds the only signing key. The bundle targets `nsis`
alone (`tauri.conf.json:62-64`) and the update manifest writes a single platform key,
`windows-x86_64` (`make-update-manifest.py:77`).

One assumption was checked and proved wrong: `keyring` (`Cargo.toml:39`) carries no Windows pin. It
keeps its default `v1` feature, which already activates the Secret Service backend on unix. That
line of work is zero.

## Recommended first target

X11 on an Ubuntu LTS or Mint session, with the input layer built on evdev and uinput from the first
day.

The reasoning is that uinput behaves the same under both display servers, so the two hardest
capabilities carry over to Wayland unchanged, whereas an XGrabKey and XTEST layer would be written
once and thrown away. The cost is a udev rule and membership of the `input` group, installed by a
`.deb` or `.rpm` postinst step. Flatpak is effectively ruled out, because reaching `/dev/uinput`
demands `--device=all`.

## A staged order of work

Each stage stands on its own and leaves the Windows build untouched.

1. **Make it compile.** Put the unguarded Win32 calls behind `cfg`. The known hard break is
   `hw.rs:357` calling `GetUserDefaultLocaleName` with no guard, reached from `app.rs:139`. Three
   enabled Win32 Cargo features have no user at all and can go.
2. **Give the engine a Linux build.** A clone and build script for whisper.cpp with a Vulkan
   backend, replacing the PowerShell producer.
3. **Close the safety hole.** Make the sensitive-application check work without `.exe` suffixes and
   without the Windows accessibility API, so it fails closed on Linux.
4. **Build the input layer.** evdev capture and uinput injection, behind the same internal
   interface the Windows code already presents.
5. **Package.** A `.deb` first, with the udev rule, then teach the update manifest a second
   platform key.
6. **Decide what the overlay does on Wayland.** The click-through behaviour has no answer there, so
   the app needs a deliberate fallback.

## Open questions for Lu

- Which Linux desktop will be the first one tested on a real machine.
- Whether the first Linux release goes out through the same updater or as a plain download.
- What the app should do on Wayland, where it can neither see the focused window nor pass the
  pointer through the overlay.
