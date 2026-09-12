# Linux port: build and packaging audit

Audit date: 12 September 2026. Worktree: `C:\Claude Projects\lalia-wt-linux`, version 0.9.6.
Scope: how the app is built, bundled, signed and shipped. No build was run and no file outside this
document was touched.

Summary: at the Cargo level the app is far closer to Linux than the shipping pipeline is. Exactly one
dependency section is target gated (`[target.'cfg(windows)'.dependencies.windows]`, Cargo.toml:52-76),
and the one dependency that looks Windows pinned by its feature name, `keyring`, in fact keeps its
default `v1` feature and therefore already pulls the Secret Service backend on unix. Three unconditional
crates pull system libraries on Linux (`cpal` needs ALSA, the `tray-icon` Tauri feature needs GTK and an
appindicator, the webview needs WebKitGTK). The packaging story is the hard part: the speech engine is a
Vulkan build of whisper.cpp produced by a single PowerShell script with absolute Windows paths that emits
`.exe` and `.dll` files, the bundle target list is `["nsis"]` alone, the runtime locator hardcodes
`whisper-server.exe`, the update manifest carries a single `windows-x86_64` platform key, and there is no
CI at all. A Linux build therefore needs a new engine build path, a new bundle target, a second platform
entry in the manifest, and a Linux host or container to run any of it on.

---

## 1. Cargo dependencies pinned to Windows at the dependency level

**Verdict: exactly one target gated section exists, `[target.'cfg(windows)'.dependencies.windows]`.
`keyring` builds on Linux unchanged and selects Secret Service automatically, despite its feature name.
The real Linux work sits in three unconditional crates that pull system libraries.**

### The one genuine target gate

`src-tauri/Cargo.toml:52-76` declares the `windows` crate (version 0.62) with 22 feature modules:
`Win32_UI_WindowsAndMessaging`, `Win32_UI_Input_KeyboardAndMouse`, `Win32_UI_Accessibility`,
`Win32_System_DataExchange`, `Win32_System_JobObjects`, `Win32_Graphics_Dxgi`, `Win32_System_Performance`,
`Win32_Graphics_Gdi`, `Win32_Globalization` and the rest (Cargo.toml:54-76).

There is no Linux equivalent crate, because there is no single equivalent API surface. The feature
modules map to separate Linux stacks, and each consumer in the code already has a `cfg(not(windows))`
arm:

| Windows feature area | Used for | Linux route |
|---|---|---|
| `Win32_Graphics_Dxgi` | GPU enumeration, `hw.rs:97-124` | `ash` or `vulkano` enumerating Vulkan physical devices, or parsing `/sys/class/drm`. Stub exists at hw.rs:126-129 |
| `Win32_System_Performance` (PDH) | GPU memory counters, `hw.rs:212-288` | `/sys/class/drm/card*/device/mem_info_*` on AMD, `nvidia-smi` on NVIDIA. Stub exists at hw.rs:290-293 |
| `Win32_UI_Input_KeyboardAndMouse` | global hotkey and text injection, hotkey.rs:18, insertion.rs:124 | `evdev` plus `uinput`, or a Wayland portal. Code level work, owned by the code audit |
| `Win32_System_JobObjects` | kill the engine when the app dies, jobobject.rs:5-52 | `prctl(PR_SET_PDEATHSIG)` or a cgroup. Stub exists at jobobject.rs:52 |
| `Win32_UI_WindowsAndMessaging` (`MessageBoxW`) | fatal error dialog, lib.rs:33-64 | already handled: lib.rs:66-70 prints to stderr on non Windows |

Only the section as a whole is Windows gated, so on a Linux target the whole `windows` crate simply is
not built. Nothing in `Cargo.toml` needs to change for that.

### keyring: the correction

`src-tauri/Cargo.toml:39` reads `keyring = { version = "4", features = ["windows-native-keyring-store"] }`.
The feature name reads as a Windows pin. It is not one, for three verified reasons:

1. The line does not set `default-features = false`, so keyring's `default = ["v1"]` stays on
   (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/keyring-4.2.0/Cargo.toml:61-66`).
   The `v1` feature set is `apple-native-keyring-store/keychain`, `windows-native-keyring-store`
   and `zbus-secret-service-keyring-store`.
2. `zbus-secret-service-keyring-store` is declared under
   `cfg(all(unix, not(any(target_os = "macos", target_os = "ios", target_os = "android"))))`
   with `features = ["crypto-rust"]` (keyring-4.2.0/Cargo.toml, unix target section). So it activates
   on Linux automatically.
3. `windows-native-keyring-store` is itself declared under `cfg(target_os = "windows")` inside keyring
   (same file, windows target section), so naming it in features on a Linux target is simply inert.
4. `keyring-4.2.0/src/v1.rs:110-121` selects the store by `cfg`: macOS keychain, Windows credential
   manager, and Secret Service on any other unix, then calls `keyring_core::set_default_store`.

The call sites are `src-tauri/src/asr/openai_compat.rs:25-43` (`keyring::Entry::new(KEYRING_SERVICE,
KEYRING_USER)`), which is the v1 API and needs no change.

The exposure is therefore a runtime one: a Linux box with no Secret Service provider
(gnome-keyring, KWallet, or another D-Bus `org.freedesktop.secrets` implementation) will fail
`Entry::new`. openai_compat.rs:39 already treats that as "no key stored" via `unwrap_or(false)`, so the
app degrades to "the cloud engine has no key" and keeps running. Worth a headless check on server
style distributions. Recommendation: leave Cargo.toml:39 as it is, and add the explicit feature
`zbus-secret-service-keyring-store` only if `default-features = false` is ever introduced.

### Unconditional crates that pull Linux system libraries

These are the ones that actually add work, because they are listed without any `cfg`:

- **`cpal = "0.18"` (Cargo.toml:31).** `src-tauri/Cargo.lock:712-725` shows `cpal 0.18.2` depending on
  `alsa`, and `alsa 0.11.0` in turn depends on `alsa-sys` (Cargo.lock:46-55). Both edges were read in
  full. Build dependency on Linux: `libasound2-dev`
  (Debian family) or `alsa-lib-devel` (Fedora family). No code change, one package in the build image.
  PipeWire and PulseAudio both expose an ALSA compatibility device, so runtime is normally fine.
- **`tauri` with the `tray-icon` feature (Cargo.toml:17).** Cargo.lock contains package entries for
  `gtk` (line 1761), `gtk-sys` (1782), `libappindicator` (2464) and `libappindicator-sys` (2477). Those
  are package presence in the lock file; the dependency edges up to `tauri` were not traced. On Linux the tray needs
  `libayatana-appindicator3` or `libappindicator3` at build and run time. General knowledge, not
  verified in repo: Tauri v2 supports the tray on Linux only through an appindicator, and desktops
  without a StatusNotifierItem host (a stock GNOME session) show no icon at all. The app hides its main
  window on start (`tauri.conf.json:23`, `"visible": false`) and is tray driven, so a missing tray on
  Linux would make it look like it never started. This needs a decision before the port ships.
- **The webview.** Cargo.lock:4870 lists `webkit2gtk`. Linux build needs `libwebkit2gtk-4.1-dev` plus
  `librsvg2-dev` and `libsoup-3.0-dev`. Not verified in repo: exact package names come from Tauri's
  published prerequisites, and no file here lists them.

Everything else in Cargo.toml:16-50 is portable as written: `rusqlite` uses `bundled` (line 30) so no
system SQLite is needed, `reqwest` uses `rustls-tls` with `default-features = false` (line 28) so no
OpenSSL is needed, and `symphonia` (line 50), `hound`, `regex`, `chrono`, `dirs`, `tracing` and the rest
are pure Rust.

**Also verified absent:** there is no `tauri.linux.conf.json` or any other platform override. `ls
src-tauri/*.json` returns `tauri.conf.json` alone. The port will have to add either a platform config
file or new entries under `bundle.targets`.

---

## 2. Tauri plugins in use, and their Linux support

**Verdict: five plugins, all of which build on Linux. Two behave differently enough to need a decision:
`updater` (only the AppImage format can self replace on Linux) and the `tray-icon` core feature
(needs an appindicator). `autostart` and `single-instance` change mechanism silently.**

Registered in `src-tauri/src/lib.rs:95-106`:

| Plugin | Declared | Registered | Notes |
|---|---|---|---|
| `tauri-plugin-single-instance` | Cargo.toml:20 | lib.rs:96-102 | |
| `tauri-plugin-opener` | Cargo.toml:18 | lib.rs:103 | also a frontend dep, package.json:16 |
| `tauri-plugin-dialog` | Cargo.toml:21 | lib.rs:104 | |
| `tauri-plugin-updater` | Cargo.toml:22 | lib.rs:105 | config at tauri.conf.json:96-105 |
| `tauri-plugin-autostart` | Cargo.toml:19 | lib.rs:106 | |

None of the five is declared with any feature flags at all (Cargo.toml:18-22 are bare `"2"` version
strings), and none sits inside a target section. So at the Cargo level every one of them is already
cross platform in this repo.

Permissions are granted in `src-tauri/capabilities/default.json:10-29`: `core:default`, fourteen
`core:window:*` entries, `core:event:default`, `opener:default`, `dialog:default`, `autostart:default`.
The capability file has no `platforms` key, so it applies to every target. Note that the file's
`"windows"` key at line 5 is the Tauri window label list (`main`, `overlay`, `scratch`), unrelated to
the operating system.

Per plugin, Linux behaviour. Everything in this list is general knowledge about Tauri v2, unverified in
this repo, except where a file is cited:

- **autostart**: on Linux it writes a `.desktop` file into `~/.config/autostart/`. The `--autostart`
  argument passed at lib.rs:106 carries over. The Windows specific counterpart in this repo is the
  NSIS hook that rewrites `HKCU\...\CurrentVersion\Run` (`src-tauri/windows/hooks.nsh:46-50`), which has
  no Linux analogue and would be replaced by the plugin's own `.desktop` handling. The
  `MacosLauncher::LaunchAgent` argument at lib.rs:106 is ignored on Linux.
- **single-instance**: on Linux it uses a D-Bus name, where Windows uses a named mutex. Works on any normal
  desktop session, and quietly fails where no session bus exists.
- **updater**: builds on Linux. Self replacement works only for AppImage. See section 5 and 7.
- **dialog** and **opener**: both work on Linux through the portal or the GTK dialogs, plus
  `xdg-open` for opener. No repo specific concern.
- **tray-icon** (a core `tauri` feature, Cargo.toml:17, alongside `image-png` and `devtools`): see
  section 1. This is the one with a real chance of a silent Linux regression.

The three windows in `tauri.conf.json:14-55` also carry two properties that are weaker on Linux:
`"transparent": true` on the overlay (line 34) needs a compositor, and `"visibleOnAllWorkspaces": true`
(line 41) plus `"alwaysOnTop": true` (line 35) are refused outright by Wayland compositors. Not verified
in repo. Window behaviour belongs to the code audit; it is noted here only because it is declared in the
build config file, which is this audit's territory.

---

## 3. How whisper.cpp is built today

**Verdict: it is built from source by one PowerShell script into a Vulkan backend, with absolute
Windows paths hardcoded, entirely outside Cargo. There is no build script and no checked in binary.
The Linux equivalent is the same CMake invocation driven from a shell script, producing
`whisper-server` plus `.so` files.**

- `src-tauri/build.rs:1-3` is two lines, `tauri_build::build()` and nothing else. Cargo does not touch
  whisper.cpp at any point.
- `scripts/build-vulkan.ps1` is the whole build. Line 22-24 is the configure step:
  `cmake -B build-vulkan -DGGML_VULKAN=ON -DBUILD_SHARED_LIBS=ON -DWHISPER_BUILD_TESTS=OFF
  -DWHISPER_BUILD_EXAMPLES=ON -DWHISPER_BUILD_SERVER=ON -DCMAKE_BUILD_TYPE=Release`, then line 27
  `cmake --build build-vulkan --config Release --parallel`.
- **Backend: Vulkan.** The script's own header (build-vulkan.ps1:1-4) states the reason: the upstream
  project ships CPU and CUDA binaries for Windows only, so the Vulkan build is produced here from the
  same tag that is already shipped, "b4938 / v1.9.3". `DEV-SETUP.md:88` records the measurement behind
  the choice: 593 ms on Vulkan against 663 ms on CUDA for the same phrase and model on an RTX 3070,
  with CUDA costing 1.1 GB. So CUDA stays in the developer checkout and the Vulkan build ships.
- **Collected artifacts** (build-vulkan.ps1:32-33): `whisper-server.exe`, `whisper.dll`, `ggml.dll`,
  `ggml-base.dll`, `ggml-vulkan.dll`, plus any `ggml-cpu*.dll`, copied from
  `build-vulkan\bin\Release` into the output folder.
- **Hardcoded absolute paths** (build-vulkan.ps1:10-12):
  `$src = "C:\Claude Projects\lalia\vendor\whisper.cpp-src"`,
  `$out = "C:\Claude Projects\lalia\vendor\whisper-vulkan"`, `$sdk = "C:\VulkanSDK\current"`.
  Line 13 locates `cmake.exe` by globbing under `C:\Program Files*\Microsoft Visual Studio\2022\`.
  The script therefore cannot run from this worktree at all: it would build the main tree's sources and
  write into the main tree's vendor folder.
- **Prerequisites, from the script header** (build-vulkan.ps1:6): Visual Studio 2022 Build Tools, the
  LunarG Vulkan SDK, CMake.

### Where the sources live

`vendor/` does not exist in this worktree. `.gitignore:27` ignores `/vendor/` and `.gitignore:43`
additionally ignores `/vendor/whisper.cpp-src/`. So the whisper.cpp source tree, the checked out tag,
any local patches, and the produced binaries are all outside version control. **How the source tree is
obtained is not verified in repo:** no clone command, submodule, or fetch script exists here. The only
recorded provenance is prose: `docs/PLAN.md:8` and `docs/THIRD-PARTY-NOTICES.md:9` name release b4938,
and `docs/PLAN.md:8` pins the CUDA zip by sha256 `c1b17166e1e31a91cc8e9c1f910d3785e3ce757bb2958bf9dce13fdb4880005f`.

### A documentation inconsistency worth fixing during the port

The build scripts and the docs disagree about which engine ships, and the scripts are the authority:

- `scripts/stage-bundle.py:29` stages the Vulkan set: `whisper-server.exe`, `whisper.dll`, `ggml.dll`,
  `ggml-base.dll`, `ggml-cpu.dll`, `ggml-vulkan.dll`. This is what lands in the installer.
- `docs/ARCHITECTURE.md:11` still describes "release b4938, CUDA 12.4 build" as the engine.
- `docs/DEV-SETUP.md:26` still tells a developer to unzip `whisper-cublas-12.4.0-bin-x64.zip` into
  `vendor/whisper/`.
- `src-tauri/src/models.rs:476-488` `RUNTIME_KEEP` still lists the CUDA DLLs (`ggml-cuda.dll`,
  `cublas64_12.dll`, `cudart64_12.dll`, `nvrtc64_120_0.dll` and the rest), because that constant serves
  the in app downloader path, a separate route from the installer.

### The Linux equivalent

Same CMake flags, different shell and different outputs:

```
cmake -B build-vulkan -DGGML_VULKAN=ON -DBUILD_SHARED_LIBS=ON \
      -DWHISPER_BUILD_TESTS=OFF -DWHISPER_BUILD_EXAMPLES=ON -DWHISPER_BUILD_SERVER=ON \
      -DCMAKE_BUILD_TYPE=Release
cmake --build build-vulkan --config Release --parallel
```

Produces `whisper-server`, `libwhisper.so`, `libggml.so`, `libggml-base.so`, `libggml-cpu.so`,
`libggml-vulkan.so`. Build prerequisites on Linux: `cmake`, a C++ toolchain, `glslc` (from
`glslang-tools` or `shaderc`), and `libvulkan-dev` plus the Vulkan headers. At run time a Vulkan ICD is
needed (`mesa-vulkan-drivers` for AMD and Intel, the proprietary driver for NVIDIA), otherwise the engine
must fall back to the CPU path. Not verified in repo: the exact Debian package names and whether the
b4938 tag builds cleanly with a current glslc.

Two extra Linux items the scripts imply. The `.so` files sit next to the binary, so the Linux build needs
`RPATH=$ORIGIN` (or an `LD_LIBRARY_PATH` wrapper) for the engine to find its own libraries, since
`whisper_server.rs:125-127` only sets `current_dir` to the executable's parent, which Windows treats as a
DLL search path and Linux does not. And the binary must be marked executable in the package, a property
the current staging step has no concept of (stage-bundle.py:42-52 only links or copies).

---

## 4. What the bundle contains besides the executable

**Verdict: the engine (6 files) and three model files, about 1.6 GB, staged into `src-tauri/bundled/`
by a Python script and shipped through `bundle.resources` as a glob. No external binaries or sidecars
are declared anywhere.**

### Declaration

- `src-tauri/tauri.conf.json:91-93`: `"resources": ["bundled/**/*"]`. That is the only resource entry.
- `src-tauri/tauri.conf.json:65-71`: `"icon"` lists `icons/32x32.png`, `icons/128x128.png`,
  `icons/128x128@2x.png`, `icons/icon.icns`, `icons/icon.ico`.
- **There is no `externalBin` key and no sidecar declaration.** Verified by grep over
  `tauri.conf.json`: zero hits for `externalBin` and zero for `sidecar`. The speech engine is a plain
  resource that the Rust code launches itself, with no Tauri managed sidecar involved. This matters for the
  port, because Tauri's sidecar mechanism appends a target triple to the binary name and the resource
  mechanism does not.

### Contents, from the staging script

`scripts/stage-bundle.py` builds `src-tauri/bundled/` (line 24) in two shapes (lines 10-13 of its
docstring):

- Full, `python scripts/stage-bundle.py`: engine plus models, about 1.6 GB.
- Slim, `python scripts/stage-bundle.py --slim`: engine only, about 55 MB (line 62 deletes any models
  left from a previous full staging so the glob cannot pick them up).

What goes in:

- `ENGINE` (stage-bundle.py:29), staged into `bundled/engine-vulkan/`: `whisper-server.exe`,
  `whisper.dll`, `ggml.dll`, `ggml-base.dll`, `ggml-cpu.dll`, `ggml-vulkan.dll`. Source folder
  `vendor/whisper-vulkan` (line 58).
- `MODELS` (stage-bundle.py:30), staged into `bundled/models/`: `ggml-large-v3-q5_0.bin`,
  `ggml-large-v3-turbo-q5_0.bin`, `ggml-silero-v5.1.2.bin`. Source folder `models/` (line 64).
- Files are hard linked (stage-bundle.py:42-52) so the 1.6 GB stays single on
  disk, with a `shutil.copy2` fallback when the link fails.

### How the running app finds them

- `src-tauri/src/app.rs:102-107`: reads `app.path().resource_dir()`, joins `bundled`, and if it exists
  calls `paths::set_bundled_dir`.
- `src-tauri/src/asr/whisper_server.rs:377-402`: `find_runtime_exe_for` builds a candidate list from
  `bundled/engine-vulkan`, `bundled/engine-cuda`, `bundled/engine`, the runtime dir and the dev vendor
  folder, then line 402 does
  `candidates.into_iter().map(|d| d.join("whisper-server.exe")).find(|p| p.exists())`.
  **The `.exe` suffix is hardcoded at that single line**, which is the smallest and most certain code
  change the Linux port needs in this area.
- `src-tauri/src/models.rs:186-190`: `migrate_bundled_models` moves `resources/bundled/models` out of the
  install folder on first run.

### Installer specific assets, Windows only

- `src-tauri/windows/hooks.nsh`, referenced at tauri.conf.json:83 (`installerHooks`). Kills a leftover
  `lalia.exe` (hooks.nsh:19-42), rewrites the `Run` registry entry (hooks.nsh:46-50), and removes it on
  uninstall (hooks.nsh:53-55). None of this has meaning on Linux.
- `icons/nsis-header.bmp` and `icons/nsis-sidebar.bmp` (tauri.conf.json:81-82), both present in
  `src-tauri/icons/`.
- `icons/icon.ico` as `installerIcon` (tauri.conf.json:80).
- `webviewInstallMode: embedBootstrapper` (tauri.conf.json:86-89), a WebView2 concept with no Linux
  counterpart.
- The icon list at tauri.conf.json:65-71 has no `.svg`, `.desktop` or hicolor sized PNG set. Linux
  packages want a 256x256 PNG and normally an SVG. The present `32x32.png`, `128x128.png` and
  `128x128@2x.png` (which is 256x256) do cover the minimum, and `icons/icon.png` also exists.

---

## 5. How the updater works, and what Linux would need

**Verdict: a static JSON manifest on our own domain pointing at GitHub release assets, signed with a
minisign key held only on one machine. The manifest has one platform key, `windows-x86_64`. Adding
Linux means one more platform key, one more built and signed artifact, and a format the updater can
actually replace in place.**

### The pieces

- **Endpoint**: `tauri.conf.json:99-101`, `"endpoints": ["https://fuckyouflow.app/updates/latest.json"]`.
  A single static URL with no `{{target}}` or `{{arch}}` template variables in it.
- **Public key**: `tauri.conf.json:98`, a base64 minisign public key beginning
  `dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDQyQUM1NTY3QUNBNTg3QTM`. Baked into every
  binary. The app refuses any installer not signed by the matching private half
  (`docs/RELEASING.md:24-28`).
- **Private key**: `~/.tauri/fuckyouflow.key`, generated 7 September 2026, outside the repository and
  outside any backup that leaves the machine (`docs/RELEASING.md:24-28`). `.gitignore:46-49` blocks
  `*.key` and `*.key.pub` from ever being committed. Losing it ends updates for every installed copy.
- **Signing at build time**: `TAURI_SIGNING_PRIVATE_KEY` holding the key contents, plus
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (`docs/RELEASING.md:42-43`). RELEASING.md:37-41 records a measured
  gotcha: with only `TAURI_SIGNING_PRIVATE_KEY_PATH` set, the build finishes, says "a public key has been
  found, but no private key", and produces no `.sig`. The fallback is
  `pnpm tauri signer sign -f ~/.tauri/fuckyouflow.key -p "" <setup exe>` (RELEASING.md:49).
- **`createUpdaterArtifacts: true`** at tauri.conf.json:94 is what makes the bundler emit the `.sig`
  alongside the installer.
- **Windows install mode**: `tauri.conf.json:102-104`, `"windows": {"installMode": "passive"}`.

### Manifest format

Written by `scripts/make-update-manifest.py:72-79`:

```json
{
  "version": "...",
  "notes": "...",
  "pub_date": "YYYY-MM-DDTHH:MM:SSZ",
  "platforms": {
    "windows-x86_64": { "signature": "...", "url": "..." }
  }
}
```

`platforms` has exactly one key (make-update-manifest.py:77). The version is read from
`tauri.conf.json` (line 43-44), the tag defaults to `v<version>` (line 45), and the URL is assembled as
`https://github.com/Breakzoras/fuck-you-flow/releases/download/<tag>/<asset>` (line 70), with the asset
name defaulting to `Fuck.You.Flow.Update.<version>.exe` (line 69). The signature is read from the
`.sig` file beside the installer, located by the glob `*-setup.exe.sig` under
`src-tauri/target/release/bundle/nsis` (line 53-61). Output goes to `site/updates/latest.json`
(line 80-83).

### Publishing

`site/deploy.sh:31` ships the `updates` directory as one of the explicitly allowed paths, over SSH to
`occadmin@193.181.216.120` (`site/deploy.sh:13`) into `/srv/fuckyouflow/site/`, then validates and
reloads Caddy in Docker (`site/deploy.sh:65-69`). `docs/RELEASING.md:59-60` and
make-update-manifest.py:93 both insist the manifest goes up last, after the installer is on GitHub.

### Two shapes of artifact

`docs/RELEASING.md:5-8` records the split, and it survives the port as a concept:

| File | Who gets it | Contents | Size |
|---|---|---|---|
| `Fuck.You.Flow.Setup.<version>.exe` | a new user, from the site | program, engine, models | about 1.7 GB |
| `Fuck.You.Flow.Update.<version>.exe` | an installed copy, by itself | program and engine | about 68 MB |

The slim update is safe because the models are moved out of the install folder on first run
(`models::migrate_bundled_models`, RELEASING.md:10-15, models.rs:186). The same trick works on Linux for
a `.deb` or `.rpm`, whose package manager also owns the install folder. It works differently for an
AppImage, which is a single mounted file, so the models must live in the user's data directory from the
first run, outside the image.

### What has to change for Linux

1. A second key under `platforms`: `linux-x86_64`, with its own signature and URL
   (make-update-manifest.py:77 becomes two entries, and the `--asset` default at line 69 needs a per
   platform form).
2. The signature glob at make-update-manifest.py:54 (`*-setup.exe.sig`) must learn about
   `*.AppImage.sig` or `*.deb.sig` and the `nsis` path segment at line 53 must become the right bundle
   subfolder.
3. The artifact naming at line 69 hardcodes `.exe`.
4. The format must be one the updater can replace in place. General knowledge, unverified in repo:
   Tauri v2's Linux updater supports AppImage self replacement; `.deb` and `.rpm` are packaged but the
   updater cannot install them without elevation, so for those the honest behaviour is to open the
   download page and let the user finish the install.
5. Nothing about the key changes. The same minisign key signs every platform, and the public half in
   tauri.conf.json:98 stays as it is.

---

## 6. Scripts that are Windows only

**Verdict: five PowerShell files, plus three shell scripts whose language is portable while their
content is pure Windows, plus two Python scripts that are portable except for `.exe` names. Only
`build-vulkan.ps1` is on the critical path for a Linux build.**

### PowerShell, needs a rewrite

| File | What it does | Linux path |
|---|---|---|
| `scripts/build-vulkan.ps1` | builds the whisper.cpp Vulkan engine (lines 22-27), collects DLLs (lines 32-37) | **Critical.** A `build-vulkan.sh` with the same CMake flags. See section 3 |
| `scripts/tray-sweep.ps1` | clears ghost tray icons after a force kill | No equivalent, and no need. Delete for Linux |
| `scripts/tray-inspect.ps1` | reads the notification area | Same |
| `scripts/tray-count-own.ps1` | counts our own tray icons | Same |
| `scripts/window-shot.ps1` | photographs the dashboard window, used as the check in DEV-SETUP.md:80 | `grim`, `import` from ImageMagick, or `xwd`. Needed if the same visual check is kept in CI |

### Shell scripts that only look portable

Both start with `#!/usr/bin/env bash` and neither runs on Linux:

- `scripts/swap-release.sh`. Line 11 reads `$LOCALAPPDATA/FuckYouFlow/logs/...`. Line 12 targets
  `fuckyouflow.exe`. Line 20 calls `tasklist //FI` and lines 43-48 call `taskkill //F //IM`, in the MSYS
  double slash form. Lines 56-64 copy into `C:\Claude Projects\fyf-run` and register a Windows Scheduled
  Task through PowerShell to escape the MSIX container. Every one of those is a Windows concept. The
  Linux version is a much shorter script: `pkill`, copy, relaunch.
- `scripts/dev-restart.sh`. Lines 4-5 `taskkill //IM`, line 6 `netstat -ano` parsing, line 11 invokes
  `tray-sweep.ps1`, line 16 reads `$LOCALAPPDATA`. Straightforward to port, entirely Windows specific
  as written.
- `site/deploy.sh:14` hardcodes `PY="C:/Python314/python.exe"`. The rest of that script is ordinary
  POSIX (tar, scp, ssh, curl) and would work on Linux by changing that one line to `python3`.

### Python, nearly portable

- `scripts/stage-bundle.py`: uses `os.path.join`, `os.link` and `shutil.copy2` throughout, so it runs
  unchanged on Linux. What needs changing is data alone: the `ENGINE` list at line 29 holds
  `.exe` and `.dll` names. Add an executable bit on the staged `whisper-server` (the script has no
  concept of file modes).
- `scripts/make-update-manifest.py`: portable Python. The Windows assumptions are the `nsis` path
  segment (line 53), the `*-setup.exe.sig` glob (line 54), the `.exe` asset name (line 69), and the
  single `windows-x86_64` platform key (line 77).

### There is no CI

`.github` does not exist in this worktree, and a search for `*.yml` and `*.yaml` returns only
`pnpm-lock.yaml`. The entire release is a sequence of commands typed on one Windows machine
(`docs/RELEASING.md:32-57`). A Linux build therefore has no host to run on today, and there is no
pipeline to add a Linux job to. This is a build blocker in its own right.

---

## 7. Linux package formats in Tauri v2, and which one fits

**Verdict: Tauri v2 bundles `deb`, `rpm` and `appimage` on Linux out of the box. For this app, ship
AppImage as the self updating artifact and `.deb` as the one people install, and keep the 1.6 GB of
models out of both by downloading them on first run.**

Everything in this section is general knowledge about Tauri v2, unverified in this
repo, since `bundle.targets` at `tauri.conf.json:62-64` is `["nsis"]` and no Linux target has ever been
configured here.

| Format | Size behaviour with a 1.6 GB payload | Updater | Fit |
|---|---|---|---|
| `.deb` | fine, the package manager owns the install folder, so the existing "move the models out on first run" trick (models.rs:186) works exactly as on Windows | cannot self replace without elevation | Good for the main download on Debian and Ubuntu |
| `.rpm` | same as `.deb` | same | Same, Fedora side |
| AppImage | a single read only mounted file. 1.6 GB inside it means a 1.6 GB download every update and a slow mount | the only Linux format Tauri can self replace | Good once the models are downloaded on first run |

### The recommendation, given the model files

The models are the deciding factor. `stage-bundle.py:30` bundles three model files and RELEASING.md:5-8
puts the full installer at about 1.7 GB. Two observations:

1. The download path already exists. `docs/DEV-SETUP.md:96` records that the download code in
   `models.rs` is live and is what the Models page in Settings uses to fetch a model that was not
   shipped, with checksum verification (DEV-SETUP.md:29). So "install the app, then fetch the model"
   is a supported flow today, already built.
2. The reason to bundle on Windows was offline first run (DEV-SETUP.md:88) and the reason compression is
   off was that the weights are already compressed (RELEASING.md:74-77). Neither reason changes the
   arithmetic for AppImage, where the payload is re-downloaded on every update.

So the shape that fits:

- **AppImage, engine only, models downloaded on first run.** That puts the artifact near the 68 MB of
  today's slim update, which is a reasonable thing to re-download per version, and it keeps the one
  Linux format the updater can actually replace.
- **`.deb` alongside it**, same contents, for people who prefer their package manager, with the updater
  told to open the download page and let the package manager finish.
- **Skip `.rpm` for the first release** unless there is a known user on Fedora, since it doubles the
  packaging surface for the same contents.
- **Skip Flatpak and Snap for now.** Both are plausible later and both add sandbox work that collides
  with global key capture and clipboard access. Not verified in repo.

Concrete config change: `tauri.conf.json:62-64` becomes something like
`"targets": ["nsis", "appimage", "deb"]`, or better, a platform specific config file so the Windows
build does not try to produce Linux packages. The `bundle.windows` block (tauri.conf.json:72-90) is
already ignored on non Windows targets, so it can stay where it is.

---

## Blockers ranked

Hardest first. Effort notes are for the build and packaging work alone and exclude the Win32 code work
(hotkeys, text injection, GPU detection), which is a separate audit.

### 1. The speech engine has no Linux build path at all
**Effort: 2 to 4 days, with real risk of a longer tail.**
`scripts/build-vulkan.ps1` is the only thing that produces the engine, it hardcodes
`C:\Claude Projects\lalia\vendor\...` and `C:\VulkanSDK\current` (lines 10-12), locates `cmake.exe`
inside a Visual Studio install (line 13), and collects `.exe` and `.dll` files (line 32). None of it
runs on Linux. Worse, the whisper.cpp source tree it builds is gitignored (`.gitignore:27,43`) and
absent from this worktree, and nothing in the repo records how it is obtained, so step one is
reconstructing the provenance of tag b4938 from prose in `docs/PLAN.md:8` and
`docs/THIRD-PARTY-NOTICES.md:9`. Then: a shell rewrite of the CMake invocation, glslc and Vulkan headers
in the build image, `$ORIGIN` RPATH so the `.so` files are found next to the binary, an executable bit on
the output, and a runtime Vulkan ICD check with a CPU fallback for machines that have none. Unknown
until tried: whether the b4938 tag builds cleanly against a current glslc.

### 2. No CI, and the whole release toolchain is Windows bound
**Effort: 2 to 3 days for a working Linux job, more to make it reproducible.**
`.github` does not exist and no workflow file of any kind is present. Every release step in
`docs/RELEASING.md:32-57` is typed by hand on one Windows machine, and the signing key lives only there
(`~/.tauri/fuckyouflow.key`, RELEASING.md:24). A Linux build has nowhere to run. This needs a container
or runner with the Rust toolchain, Node and pnpm, `libasound2-dev`, `libwebkit2gtk-4.1-dev`,
`libayatana-appindicator3-dev`, the Vulkan SDK pieces, plus a decision about how the signing key reaches
it. This ranks second only because it is ordinary work with no unknowns, unlike blocker 1.

### 3. The bundle and updater chain is NSIS shaped end to end
**Effort: 2 to 3 days.**
`tauri.conf.json:62-64` targets `nsis` alone. `make-update-manifest.py` hunts for `*-setup.exe.sig` under
the `nsis` folder (lines 53-54), names the asset `...Update.<version>.exe` (line 69), and writes a
manifest with one platform key, `windows-x86_64` (line 77). Adding Linux means new bundle targets, a per
platform artifact path and glob, a second platform entry with its own signature, and a decision about
which Linux format the updater is allowed to self replace. The minisign key itself needs no change,
which is the one piece of good news here.

### 4. The 1.6 GB of models do not fit the Linux formats the same way
**Effort: 1 to 2 days, mostly decision and testing, with little code.**
`stage-bundle.py:30` bundles three model files and the full installer runs about 1.7 GB
(RELEASING.md:5-8). For `.deb` the existing first run move (models.rs:186) carries over unchanged. For
AppImage it does not, because the image is read only and a bundled model would be re-downloaded on every
update. The fix is to lean on the download path that already exists and is already used by the Models
page (DEV-SETUP.md:96), which costs a first run flow change and no new machinery.

### 5. The tray may be invisible on a stock GNOME session
**Effort: half a day to establish the facts, unknown after that.**
The app starts with its main window hidden (`tauri.conf.json:23`) and is driven from the tray
(`tray-icon` feature, Cargo.toml:17). On Linux that needs an appindicator host, which several default
desktop sessions lack. If the icon does not appear, the app looks like it failed to start. Measure it on
a plain GNOME and a plain KDE before deciding whether a visible window becomes the Linux default.

### 6. Windows only installer assets and hooks have no counterpart
**Effort: half a day.**
`src-tauri/windows/hooks.nsh` handles the old `lalia.exe` process (lines 19-42) and the `Run` registry
entry (lines 46-50); `installerIcon`, `headerImage`, `sidebarImage` (tauri.conf.json:80-82) and
`webviewInstallMode` (lines 86-89) are all Windows concepts. On Linux the autostart plugin writes a
`.desktop` file by itself and the package manager handles upgrades, so most of this is pure deletion.
A `.desktop` entry and a 256x256 icon are the only additions, and the icons directory
already holds a suitable PNG.

### 7. `find_runtime_exe_for` hardcodes the `.exe` suffix
**Effort: under an hour.**
`src-tauri/src/asr/whisper_server.rs:402` joins `"whisper-server.exe"` onto every candidate directory.
One `cfg` or one platform aware constant fixes it. Listed last because it is the smallest and most
certain change in this whole audit, and it is the thing that would otherwise make a correct Linux build
report "engine missing" (whisper_server.rs:90).

### Not a blocker: keyring
Listed here because the task asked and because the feature name invites the wrong conclusion.
`keyring` at Cargo.toml:39 builds on Linux with no change and selects Secret Service automatically
(keyring-4.2.0/src/v1.rs:110-121). The only exposure is a machine with no Secret Service provider, where
`openai_compat.rs:39` already degrades to "no key stored". Zero build effort, worth one headless test.
