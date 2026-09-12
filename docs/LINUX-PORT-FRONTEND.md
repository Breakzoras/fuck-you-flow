# Linux port: the frontend layer

Measured on branch `port/linux`, based on `fix/audit-2026-09-11` at commit b403881 (version 0.9.6).

The web frontend is close to portable. It is 21 files of TypeScript, TSX and CSS under `src/`,
and it reaches the machine only through Tauri: 15 call sites across 8 files, all of them either a
command handled by the Rust side or a Tauri window API. Nothing in it opens a file by absolute
path, spawns a process, or reaches for a Win32 call. Two things do need work: user facing wording that names Windows, and one window
behaviour whose Linux support depends on the display server.

## What was checked

`grep` over `src/**/*.ts` and `src/**/*.tsx` for: `windows`, `win32`, `appdata`, a drive letter,
`.exe`, `msix`, `nsis`. Every hit was read in place.

## Findings

### 1. Wording that names Windows: 11 strings, both languages

`src/i18n.ts` carries 11 lines that say "Windows" to the user. They are real product copy, in the
Greek and the English dictionary alike. Examples:

| Line | Key | Text |
|---|---|---|
| 160 | `theme_system` | "Όπως τα Windows" |
| 163 | `autostart` | "Εκκίνηση με τα Windows" |
| 167 | `keep_warm_note` | the permanent microphone indicator |
| 285 | `key_check_hint` | "Ό,τι φτάνει στα Windows εμφανίζεται εδώ" |
| 306 | `api_key` | "φυλάσσεται στον Windows Credential Manager" |
| 492, 495, 497, 499, 617, 638 | the English twins of the same keys | |

Two of these are more than wording. `api_key` names the Windows Credential Manager, which on Linux
becomes the Secret Service keyring, and `theme_system` and `autostart` describe mechanisms that
differ per desktop. The fix is an operating system aware label, chosen at runtime, so the Greek
keeps reading naturally on both systems. A blind find and replace would break it.

Effort: small, once a decision is made about how the app names the host system in copy.

### 2. Click through overlay: one call, Tauri owned

`src/overlay/main.tsx:180` is the only place the frontend asks the window manager for anything:

```
getCurrentWindow().setIgnoreCursorEvents(!p.can_retry)
```

This is the Tauri API, so it compiles and runs on Linux unchanged. Whether the
pointer actually passes through the transparent area is a display server question and belongs to
the capability audit in `LINUX-PORT-GAPS.md`, section 4. The frontend needs no change here. The
behaviour needs verification on a real Linux desktop.

### 3. Comments that describe Windows behaviour: no code impact

`src/GpuGauge.tsx:11` and `:44`, `src/pages/Diagnostics.tsx:17` and `:31`, and
`src/overlay/main.tsx:176` explain Windows specific behaviour in prose. They are accurate history
and should stay, with a note added when the Linux path lands.

### 4. The command surface is `src/api.ts`, and it is centralised

`src/api.ts:1` imports `invoke` from `@tauri-apps/api/core`, and from line 111 onward every Rust
command is wrapped in a typed one-line method: `get_settings`, `save_settings`,
`get_pipeline_snapshot`, `pipeline_toggle`, `pipeline_cancel`, `pipeline_retry`, `paste_last` and
the rest. Every call carries a type parameter, so the shape is `invoke<Settings>("get_settings")`.

A first pass here counted these with `grep -c "invoke("` and got 0, which was a false reading: the
generic parameter sits between the name and the parenthesis. Counted correctly,
`grep -rn "invoke(\|@tauri-apps/api" src/` returns 15 hits across 8 files: `api.ts`, `App.tsx`,
`GpuGauge.tsx`, `overlay/main.tsx`, `pages/History.tsx`, `pages/Home.tsx`, `pages/SettingsPage.tsx`
and `scratch/main.tsx`.

So the frontend does reach Tauri directly in eight places. All of them are either a command call
into the Rust side or a Tauri window API such as the one in section 2. None of them is an
operating system call of its own, which is why the verdict below stands.

## Verdict

The frontend is PORTABLE. The work it carries is 11 product strings and one behaviour to verify on
a live Linux desktop. No architectural change.
