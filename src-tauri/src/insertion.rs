//! Putting text where the cursor is, in whatever application had focus.
//!
//! Layers:
//! 1. Clipboard paste with a delayed-render promise. We publish an empty promise
//!    for CF_UNICODETEXT, press Ctrl+V, and only when the target actually asks
//!    for the data (WM_RENDERFORMAT) do we hand it over. Then we restore the
//!    previous clipboard text. That removes the timing race that plagues
//!    fixed-delay implementations.
//! 2. Unicode typing through SendInput for apps that refuse paste (or when the
//!    user chose "type").
//! 3. Copy only: leave the text on the clipboard and tell the user.
//!
//! All Win32 calls live on one dedicated thread that owns a hidden window.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

/// Windows that accept Ctrl+V without holding text. The desktop treats it as
/// "paste a file" and the taskbar ignores it, and both read the clipboard while
/// doing so, which is the only signal the paste path has. Sending a transcript
/// there looks like a success in every measurement and shows the user nothing.
///
/// Returns a name for the place, for the message the user reads.
///
/// A class is listed here only when it certainly cannot hold text. Absence of a
/// caret is NOT such a signal: Chromium never reports one, and refusing on that
/// basis would break the browsers, editors and chat apps that work today.
pub fn unusable_class(class: &str) -> Option<&'static str> {
    match class {
        "Progman" | "WorkerW" => Some("the desktop"),
        "Shell_TrayWnd" | "Shell_SecondaryTrayWnd" => Some("the taskbar"),
        // The hidden window behind our own tray icon. It holds the focus after
        // a click on the icon; measured 9 September 2026, a 52-word dictation
        // was pasted into it and reported as "the app did not accept it".
        "tray_icon_app" => Some("the tray icon"),
        "" => Some("no window"),
        _ => None,
    }
}

#[cfg(test)]
mod target_tests {
    use super::unusable_class;

    #[test]
    fn the_desktop_and_the_taskbar_are_refused() {
        assert_eq!(unusable_class("Progman"), Some("the desktop"));
        assert_eq!(unusable_class("WorkerW"), Some("the desktop"));
        assert_eq!(unusable_class("Shell_TrayWnd"), Some("the taskbar"));
        assert_eq!(unusable_class("Shell_SecondaryTrayWnd"), Some("the taskbar"));
        assert_eq!(unusable_class("tray_icon_app"), Some("the tray icon"));
        assert_eq!(unusable_class(""), Some("no window"));
    }

    /// The regression this guard must never cause: everything the user actually
    /// dictates into keeps working. Chromium reports no caret, Windows Terminal
    /// is not a normal edit control, and both paste fine today.
    #[test]
    fn real_targets_are_left_alone() {
        for class in [
            "Chrome_WidgetWin_1", // Chrome, Electron, VS Code, Slack, Claude
            "MozillaWindowClass",
            "OpusApp",            // Word
            "CASCADIA_HOSTING_WINDOW_CLASS", // Windows Terminal
            "ConsoleWindowClass",
            "Notepad",
            "Tauri Window",       // our own window
            "SunAwtFrame",        // JetBrains
        ] {
            assert_eq!(unusable_class(class), None, "{class} must still receive pastes");
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InsertOutcome {
    /// Text was pasted and the previous clipboard restored.
    Pasted,
    /// Text was pasted; the previous clipboard could not be restored (still ours).
    PastedNoRestore,
    /// Text was typed key by key.
    Typed,
    /// Text is on the clipboard, nothing was sent to the app.
    CopiedOnly,
    /// The paste keystroke was sent but the app never read the clipboard.
    PasteNotConsumed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertReport {
    pub outcome: InsertOutcome,
    pub method: String,
    pub message: Option<String>,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone)]
pub struct InsertOptions {
    pub restore_clipboard: bool,
    pub settle_ms: u64,
    /// Use Ctrl+Shift+V (terminals) instead of Ctrl+V.
    pub shift_paste: bool,
}

/// Snapshot of the window that should receive the text.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Target {
    pub hwnd: isize,
    pub thread_id: u32,
    pub process_id: u32,
    pub process_name: String,
    pub title: String,
    pub elevated: bool,
    pub is_password_field: bool,
}

#[cfg(windows)]
pub mod win {
    use super::*;
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::{CloseHandle, GetLastError, SetLastError, HANDLE, HGLOBAL, HWND, LPARAM, LRESULT, WIN32_ERROR, WPARAM};
    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData, GetOpenClipboardWindow,
        IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
    };
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::System::Ole::CF_UNICODETEXT;
    use windows::Win32::System::Threading::{
        GetCurrentProcess, OpenProcess, OpenProcessToken, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
        KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_CONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_RMENU, VK_RSHIFT,
        VK_RWIN, VK_SHIFT, VK_V,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowTextLengthW,
        GetWindowTextW, GetWindowThreadProcessId, IsWindow, PostMessageW, RegisterClassW, SetForegroundWindow,
        TranslateMessage, HWND_MESSAGE, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_DESTROYCLIPBOARD,
        WM_RENDERALLFORMATS, WM_RENDERFORMAT, WNDCLASSW,
    };

    const WM_LALIA_PUBLISH: u32 = WM_APP + 1;
    const WM_LALIA_RESTORE: u32 = WM_APP + 2;
    const WM_LALIA_SETTEXT: u32 = WM_APP + 3;

    struct Shared {
        /// Text promised to the clipboard (rendered on demand).
        pending_text: Option<Vec<u16>>,
        /// Everything that was on the clipboard, to put back afterwards.
        saved_formats: Vec<(u32, Vec<u8>)>,
        /// When we pressed Ctrl+V. Renders before that are clipboard managers, not the target.
        keystroke_at: Option<Instant>,
        rendered_after_keystroke: u32,
        /// The process that had focus when Ctrl+V was sent. Only a read by this
        /// process proves the target took the text.
        target_pid: u32,
        /// Reads by that process, after the key. This is the one that decides.
        rendered_by_target: u32,
        /// Everyone else who read the clipboard, by name, for the log. Windows
        /// gives no way to ask "did that application paste", so the only honest
        /// answer comes from watching who asked for the data.
        other_readers: Vec<String>,
        last_render: Option<Instant>,
        publish_result: Option<Result<(), String>>,
    }

    struct State {
        shared: Mutex<Shared>,
        render_tx: Sender<()>,
        done_tx: Sender<Result<(), String>>,
        hwnd: AtomicU32,
        hwnd_hi: AtomicU32,
        ready: AtomicBool,
    }

    static STATE: once_cell::sync::OnceCell<Arc<State>> = once_cell::sync::OnceCell::new();
    static RENDER_RX: once_cell::sync::OnceCell<Mutex<Receiver<()>>> = once_cell::sync::OnceCell::new();
    static DONE_RX: once_cell::sync::OnceCell<Mutex<Receiver<Result<(), String>>>> = once_cell::sync::OnceCell::new();

    fn hwnd() -> HWND {
        let st = STATE.get().expect("clipboard thread");
        let lo = st.hwnd.load(Ordering::SeqCst) as usize;
        let hi = st.hwnd_hi.load(Ordering::SeqCst) as usize;
        HWND(((hi << 32) | lo) as *mut core::ffi::c_void)
    }

    fn to_wide(s: &str) -> Vec<u16> {
        let mut v: Vec<u16> = s.encode_utf16().collect();
        v.push(0);
        v
    }

    fn from_wide(v: &[u16]) -> String {
        let end = v.iter().position(|&c| c == 0).unwrap_or(v.len());
        String::from_utf16_lossy(&v[..end])
    }

    unsafe fn hglobal_from_wide(text: &[u16]) -> Option<HGLOBAL> {
        let bytes = text.len() * 2;
        let h = GlobalAlloc(GMEM_MOVEABLE, bytes).ok()?;
        let p = GlobalLock(h) as *mut u16;
        if p.is_null() {
            return None;
        }
        std::ptr::copy_nonoverlapping(text.as_ptr(), p, text.len());
        let _ = GlobalUnlock(h);
        Some(h)
    }

    unsafe fn hglobal_dword(value: u32) -> Option<HGLOBAL> {
        let h = GlobalAlloc(GMEM_MOVEABLE, 4).ok()?;
        let p = GlobalLock(h) as *mut u32;
        if p.is_null() {
            return None;
        }
        *p = value;
        let _ = GlobalUnlock(h);
        Some(h)
    }

    unsafe fn open_clipboard_retry(owner: HWND) -> bool {
        for _ in 0..12 {
            if OpenClipboard(Some(owner)).is_ok() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(40));
        }
        false
    }

    unsafe fn read_clipboard_text() -> Option<Vec<u16>> {
        if !IsClipboardFormatAvailable(CF_UNICODETEXT.0 as u32).is_ok() {
            return None;
        }
        let h = GetClipboardData(CF_UNICODETEXT.0 as u32).ok()?;
        let hg = HGLOBAL(h.0);
        let p = GlobalLock(hg) as *const u16;
        if p.is_null() {
            return None;
        }
        // Read no further than the block Windows actually handed us. The old
        // loop walked forward looking for a zero and gave up only after 50
        // million characters, so a program that puts text on the clipboard
        // without the closing zero (which is allowed, and some do) sent this
        // read straight past the end of the memory. That is an access
        // violation: the process dies on the spot, with no panic to catch, no
        // line in the log and nothing in the Windows event log. Two such
        // deaths happened on 7 September 2026 and left nothing behind.
        let chars = GlobalSize(hg) / 2;
        let mut len = 0usize;
        while len < chars && *p.add(len) != 0 {
            len += 1;
        }
        let mut v = std::slice::from_raw_parts(p, len).to_vec();
        // The callers expect the closing zero, and it may be the one thing the
        // other program left out.
        v.push(0);
        let _ = GlobalUnlock(hg);
        Some(v)
    }

    /// Clipboard formats whose handle is not a memory block. Copying one the
    /// way a memory block is copied reads a bitmap or a palette handle as if it
    /// were a pointer, so they are left out of the snapshot and are the only
    /// things a dictation can still cost the user.
    fn is_handle_format(fmt: u32) -> bool {
        matches!(fmt, 2 | 3 | 9 | 14 | 0x80 | 0x82 | 0x83 | 0x8E)
    }

    /// Everything on the clipboard right now, format by format, as raw bytes.
    ///
    /// Why this exists: the app has to own the clipboard to publish its promise,
    /// and owning it means calling `EmptyClipboard`, which destroys every format
    /// on it. Until 10 September 2026 only the plain text was saved and put
    /// back, so a copied picture, a copied file or copied formatted text was
    /// destroyed by every single dictation, with nothing said and no way back.
    ///
    /// The clipboard must already be open. Asking for a format whose owner
    /// promised it makes that owner render it now, which is exactly what a
    /// clipboard manager does and the only way to hold a real copy.
    unsafe fn snapshot_clipboard() -> Vec<(u32, Vec<u8>)> {
        // A guard against a huge picture: 32 MB is far more than any text or
        // file list and still small enough to hold twice for a moment.
        //
        // Time matters more than size here. Asking for a format its owner only
        // promised makes that owner render it now, and a spreadsheet or a
        // remote desktop can take seconds over it. The caller's wait allows for
        // that, and the duration is logged so a slow one can be seen.
        const BUDGET: usize = 32 * 1024 * 1024;
        let started = Instant::now();
        let mut out: Vec<(u32, Vec<u8>)> = Vec::new();
        let mut used = 0usize;
        let mut fmt = EnumClipboardFormats(0);
        let mut skipped = 0u32;
        while fmt != 0 {
            if !is_handle_format(fmt) {
                if let Ok(h) = GetClipboardData(fmt) {
                    let hg = HGLOBAL(h.0);
                    if !hg.0.is_null() {
                        let size = GlobalSize(hg);
                        if size > 0 && size <= BUDGET && used + size <= BUDGET {
                            let p = GlobalLock(hg) as *const u8;
                            if !p.is_null() {
                                out.push((fmt, std::slice::from_raw_parts(p, size).to_vec()));
                                used += size;
                                let _ = GlobalUnlock(hg);
                            }
                        } else if size > 0 {
                            skipped += 1;
                        }
                    }
                }
            } else {
                skipped += 1;
            }
            fmt = EnumClipboardFormats(fmt);
        }
        let ms = started.elapsed().as_millis();
        if skipped > 0 || ms > 200 {
            tracing::debug!("clipboard snapshot: {} formats kept, {skipped} skipped, {ms} ms", out.len(), );
        }
        out
    }

    /// Puts a snapshot back. The clipboard must be open and already emptied.
    unsafe fn restore_snapshot(saved: &[(u32, Vec<u8>)]) {
        for (fmt, bytes) in saved {
            if let Some(hg) = hglobal_from_bytes(bytes) {
                if SetClipboardData(*fmt, Some(HANDLE(hg.0))).is_err() {
                    // Ownership did not transfer, so this block stays allocated.
                    // It is one failed format on one dictation, and freeing it
                    // needs a Win32 function this build does not expose; a leak
                    // here is cheaper than a double free on a handle Windows may
                    // in fact have taken.
                    tracing::debug!("clipboard restore: format {fmt} refused");
                }
            }
        }
    }

    unsafe fn hglobal_from_bytes(bytes: &[u8]) -> Option<HGLOBAL> {
        let hg = GlobalAlloc(GMEM_MOVEABLE, bytes.len()).ok()?;
        let p = GlobalLock(hg) as *mut u8;
        if p.is_null() {
            return None;
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
        let _ = GlobalUnlock(hg);
        Some(hg)
    }

    /// Tell Windows what may be done with what we just put on the clipboard.
    ///
    /// `recoverable` is for the last-resort copy, the one the user is asked to
    /// paste by hand. Refusing the local clipboard history there means that if
    /// they copy anything else first, the dictation is gone from every place
    /// they would think to look. The cloud is refused either way: a transcript
    /// of someone's voice has no business on another company's servers.
    unsafe fn set_optout_formats_ex(recoverable: bool) {
        let mut names: Vec<PCWSTR> = vec![w!("CanUploadToCloudClipboard")];
        if !recoverable {
            names.push(w!("ExcludeClipboardContentFromMonitorProcessing"));
            names.push(w!("CanIncludeInClipboardHistory"));
        }
        for name in names {
            let fmt = RegisterClipboardFormatW(name);
            if fmt != 0 {
                if let Some(h) = hglobal_dword(0) {
                    let _ = SetClipboardData(fmt, Some(HANDLE(h.0)));
                }
            }
        }
    }

    unsafe fn set_optout_formats() {
        set_optout_formats_ex(false);
    }

    unsafe fn render_pending(st: &State) {
        let text = st.shared.lock().pending_text.clone();
        if let Some(t) = text {
            if let Some(h) = hglobal_from_wide(&t) {
                let _ = SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(h.0)));
            }
        }
        // Who is asking. GetOpenClipboardWindow names the window that holds the
        // clipboard open, which is the one calling GetClipboardData right now.
        // Without this the app counted any read as proof the target had pasted,
        // and on 10 September 2026 that made eight dictations in a row report
        // success while the words never reached the window.
        let mut reader_pid = 0u32;
        if let Ok(owner) = GetOpenClipboardWindow() {
            if !owner.is_invalid() {
                GetWindowThreadProcessId(owner, Some(&mut reader_pid));
            }
        }

        let mut sh = st.shared.lock();
        let now = Instant::now();
        if let Some(k) = sh.keystroke_at {
            if now >= k {
                sh.rendered_after_keystroke += 1;
                // An unknown reader counts as the target. Guessing the other way
                // would turn every paste the app cannot see into a failure.
                if reader_pid == 0 {
                    // OpenClipboard(NULL) leaves no window to name. Counting it
                    // as the target avoids inventing a failure, and saying so in
                    // the log keeps it from passing as proof. Without this line
                    // "one read by the target, other readers: none" would look
                    // the same whether the window really read it or not.
                    sh.rendered_by_target += 1;
                    let unknown = "unknown".to_string();
                    if !sh.other_readers.contains(&unknown) {
                        sh.other_readers.push(unknown);
                    }
                } else if reader_pid == sh.target_pid {
                    sh.rendered_by_target += 1;
                } else {
                    let name = process_name(reader_pid);
                    if !sh.other_readers.contains(&name) {
                        sh.other_readers.push(name);
                    }
                }
            }
        }
        sh.last_render = Some(now);
        drop(sh);
        let _ = st.render_tx.try_send(());
    }

    unsafe extern "system" fn wndproc(h: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let Some(st) = STATE.get() else { return DefWindowProcW(h, msg, wparam, lparam) };
        match msg {
            WM_RENDERFORMAT => {
                if wparam.0 as u32 == CF_UNICODETEXT.0 as u32 {
                    render_pending(st);
                }
                LRESULT(0)
            }
            WM_RENDERALLFORMATS => {
                if OpenClipboard(Some(h)).is_ok() {
                    render_pending(st);
                    let _ = CloseClipboard();
                }
                LRESULT(0)
            }
            WM_DESTROYCLIPBOARD => LRESULT(0),
            WM_LALIA_PUBLISH => {
                // Save current text, publish the promise plus opt-out formats.
                let result: Result<(), String> = (|| {
                    if !open_clipboard_retry(h) {
                        return Err("clipboard is locked by another application".into());
                    }
                    let saved = snapshot_clipboard();
                    let ok = EmptyClipboard().is_ok();
                    if !ok {
                        let _ = CloseClipboard();
                        return Err("EmptyClipboard failed".into());
                    }
                    // Delayed rendering: SetClipboardData(format, NULL) returns NULL on
                    // success as well, so the crate's Result is meaningless here. Trust
                    // the thread error code instead.
                    SetLastError(WIN32_ERROR(0));
                    let promised = match SetClipboardData(CF_UNICODETEXT.0 as u32, None) {
                        Ok(_) => true,
                        Err(_) => GetLastError() == WIN32_ERROR(0),
                    };
                    set_optout_formats();
                    let _ = CloseClipboard();
                    if !promised {
                        return Err(format!("SetClipboardData promise failed (error {})", GetLastError().0));
                    }
                    let mut sh = st.shared.lock();
                    sh.saved_formats = saved;
                    sh.rendered_after_keystroke = 0;
                    sh.last_render = None;
                    Ok(())
                })();
                let _ = st.done_tx.try_send(result);
                LRESULT(0)
            }
            WM_LALIA_SETTEXT => {
                // Plain copy: put real data on the clipboard immediately (copy-only mode).
                let result: Result<(), String> = (|| {
                    if !open_clipboard_retry(h) {
                        return Err("clipboard is locked by another application".into());
                    }
                    let _ = EmptyClipboard();
                    let text = st.shared.lock().pending_text.clone().unwrap_or_default();
                    let ok = hglobal_from_wide(&text).map(|hg| SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(hg.0))).is_ok()).unwrap_or(false);
                    set_optout_formats_ex(true);
                    let _ = CloseClipboard();
                    if ok {
                        Ok(())
                    } else {
                        Err("SetClipboardData failed".into())
                    }
                })();
                let _ = st.done_tx.try_send(result);
                LRESULT(0)
            }
            WM_LALIA_RESTORE => {
                let result: Result<(), String> = (|| {
                    let saved = std::mem::take(&mut st.shared.lock().saved_formats);
                    if !open_clipboard_retry(h) {
                        return Err("clipboard is locked by another application".into());
                    }
                    let _ = EmptyClipboard();
                    restore_snapshot(&saved);
                    let _ = CloseClipboard();
                    st.shared.lock().pending_text = None;
                    Ok(())
                })();
                let _ = st.done_tx.try_send(result);
                LRESULT(0)
            }
            _ => DefWindowProcW(h, msg, wparam, lparam),
        }
    }

    fn thread_main() {
        unsafe {
            let hinst = GetModuleHandleW(None).unwrap_or_default();
            let class_name = w!("LaliaClipboardOwner");
            let wc = WNDCLASSW { lpfnWndProc: Some(wndproc), hInstance: hinst.into(), lpszClassName: class_name, ..Default::default() };
            RegisterClassW(&wc);
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class_name,
                w!("Lalia clipboard"),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                Some(hinst.into()),
                None,
            );
            let Ok(hwnd) = hwnd else {
                tracing::error!("clipboard owner window creation failed");
                return;
            };
            let st = STATE.get().unwrap();
            let raw = hwnd.0 as usize;
            st.hwnd.store((raw & 0xFFFF_FFFF) as u32, Ordering::SeqCst);
            st.hwnd_hi.store((raw >> 32) as u32, Ordering::SeqCst);
            st.ready.store(true, Ordering::SeqCst);
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    pub fn ensure_started() {
        if STATE.get().is_some() {
            return;
        }
        let (render_tx, render_rx) = crossbeam_channel::bounded::<()>(64);
        let (done_tx, done_rx) = crossbeam_channel::bounded::<Result<(), String>>(8);
        let st = Arc::new(State {
            shared: Mutex::new(Shared { pending_text: None, saved_formats: Vec::new(), keystroke_at: None, rendered_after_keystroke: 0, target_pid: 0, rendered_by_target: 0, other_readers: Vec::new(), last_render: None, publish_result: None }),
            render_tx,
            done_tx,
            hwnd: AtomicU32::new(0),
            hwnd_hi: AtomicU32::new(0),
            ready: AtomicBool::new(false),
        });
        let _ = STATE.set(st);
        let _ = RENDER_RX.set(Mutex::new(render_rx));
        let _ = DONE_RX.set(Mutex::new(done_rx));
        std::thread::Builder::new().name("lalia-clipboard".into()).spawn(thread_main).expect("clipboard thread");
        let start = Instant::now();
        while !STATE.get().unwrap().ready.load(Ordering::SeqCst) && start.elapsed() < Duration::from_secs(3) {
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// Throws away an answer left behind by a call that gave up waiting. The
    /// channel holds eight, so without this the reply to a timed-out request
    /// would be handed to the next request as if it were its own, and a
    /// clipboard step that failed would be reported as done.
    fn drain_done() {
        if let Some(rx) = DONE_RX.get() {
            let rx = rx.lock();
            while rx.try_recv().is_ok() {}
        }
    }

    fn wait_done(timeout: Duration) -> Result<(), String> {
        let rx = DONE_RX.get().unwrap().lock();
        rx.recv_timeout(timeout).map_err(|_| "clipboard thread timeout".to_string())?
    }

    fn key(vk: VIRTUAL_KEY, up: bool) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT { wVk: vk, wScan: 0, dwFlags: if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) }, time: 0, dwExtraInfo: crate::hotkey::LALIA_INJECT_SIG },
            },
        }
    }

    /// Release every modifier the user may still be holding, so the synthesized
    /// paste is a clean Ctrl+V and not AltGr+V or Win+V.
    /// The keys that must be up before Ctrl+V means "paste". Left and right
    /// variants separately: a generic key-up does not clear the right-hand key.
    const MODIFIERS: [(windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY, &str); 8] = [
        (VK_LSHIFT, "LShift"), (VK_RSHIFT, "RShift"), (VK_LMENU, "LAlt"), (VK_RMENU, "RAlt"),
        (VK_LWIN, "LWin"), (VK_RWIN, "RWin"),
        (windows::Win32::UI::Input::KeyboardAndMouse::VK_LCONTROL, "LCtrl"),
        (windows::Win32::UI::Input::KeyboardAndMouse::VK_RCONTROL, "RCtrl"),
    ];

    /// Tick count of the last keyboard or mouse input on the desktop (our own
    /// injected keys count too, so compare against the tick right after sending).
    fn last_input_tick() -> u32 {
        use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
        let mut info = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
        unsafe {
            let _ = GetLastInputInfo(&mut info);
        }
        info.dwTime
    }

    fn held_modifiers() -> Vec<&'static str> {
        unsafe { MODIFIERS.iter().filter(|(vk, _)| (GetAsyncKeyState(vk.0 as i32) as u16 & 0x8000) != 0).map(|(_, n)| *n).collect() }
    }

    /// Waits until no modifier key is physically held, up to `timeout`. Returns
    /// the keys still down when the wait gave up.
    pub fn wait_for_modifier_release(timeout: Duration) -> Vec<&'static str> {
        let start = Instant::now();
        loop {
            let held = held_modifiers();
            if held.is_empty() || start.elapsed() >= timeout {
                return held;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// The class name of a window, empty when it has none.
    fn window_class(h: HWND) -> String {
        use windows::Win32::UI::WindowsAndMessaging::GetClassNameW;
        unsafe {
            let mut b = [0u16; 128];
            let n = GetClassNameW(h, &mut b).max(0) as usize;
            String::from_utf16_lossy(&b[..n])
        }
    }

    /// Some windows swallow Ctrl+V without complaining and without showing
    /// anything: the desktop treats it as "paste a file", the taskbar ignores
    /// it. A paste sent there looks like a success in every measurement we have,
    /// while the user sees nothing. Name those cases so the caller can refuse.
    ///
    /// Only classes that are certainly not text targets belong here. A window
    /// that merely does not report a caret (Chromium does not) must NOT be
    /// listed: that would refuse pastes that work today.
    fn unusable_target(h: HWND) -> Option<&'static str> {
        super::unusable_class(&window_class(h))
    }

    /// Which window has the keyboard focus right now, for the log when a paste
    /// went nowhere.
    fn focus_diagnostics() -> String {
        use windows::Win32::UI::WindowsAndMessaging::{GetGUIThreadInfo, GUITHREADINFO};
        unsafe {
            let fg = GetForegroundWindow();
            let tid = GetWindowThreadProcessId(fg, None);
            let mut gti = GUITHREADINFO::default();
            gti.cbSize = std::mem::size_of::<GUITHREADINFO>() as u32;
            let _ = GetGUIThreadInfo(tid, &mut gti);
            let class = window_class;
            format!(
                "foreground {:?} '{}', focused control {:?} '{}', caret window {:?}",
                fg, class(fg), gti.hwndFocus, class(gti.hwndFocus), gti.hwndCaret
            )
        }
    }

    pub fn release_modifiers() {
        let mods = [VK_LSHIFT, VK_RSHIFT, VK_SHIFT, VK_LMENU, VK_RMENU, VK_MENU, VK_LWIN, VK_RWIN, VK_CONTROL,
            windows::Win32::UI::Input::KeyboardAndMouse::VK_LCONTROL, windows::Win32::UI::Input::KeyboardAndMouse::VK_RCONTROL];
        let mut inputs = Vec::new();
        unsafe {
            for m in mods {
                if (GetAsyncKeyState(m.0 as i32) as u16 & 0x8000) != 0 {
                    inputs.push(key(m, true));
                }
            }
            if !inputs.is_empty() {
                SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                std::thread::sleep(Duration::from_millis(15));
            }
        }
    }

    fn send_paste_chord(shift: bool) {
        unsafe {
            let mut inputs = vec![key(VK_CONTROL, false)];
            if shift {
                inputs.push(key(VK_SHIFT, false));
            }
            inputs.push(key(VK_V, false));
            inputs.push(key(VK_V, true));
            if shift {
                inputs.push(key(VK_SHIFT, true));
            }
            inputs.push(key(VK_CONTROL, true));
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }

    /// Whether the clipboard's text right now is exactly `expected`. `None`
    /// when it could not be opened or holds no text.
    fn clipboard_holds(expected: &[u16]) -> Option<bool> {
        unsafe {
            if OpenClipboard(None).is_err() {
                return None;
            }
            let out = (|| {
                let h = GetClipboardData(CF_UNICODETEXT.0 as u32).ok()?;
                let hg = HGLOBAL(h.0);
                let p = GlobalLock(hg) as *const u16;
                if p.is_null() {
                    return None;
                }
                let mut n = 0usize;
                while *p.add(n) != 0 {
                    n += 1;
                }
                let got = std::slice::from_raw_parts(p, n).to_vec();
                let _ = GlobalUnlock(hg);
                let want: Vec<u16> = expected.iter().copied().take_while(|c| *c != 0).collect();
                Some(got == want)
            })();
            let _ = CloseClipboard();
            out
        }
    }

    /// Puts whatever was on the clipboard back, for the paths that give up
    /// after the app has already taken ownership. Does nothing when there is
    /// nothing saved, so it is safe to call twice.
    fn restore_clipboard_now() {
        let has = STATE.get().map(|st| !st.shared.lock().saved_formats.is_empty()).unwrap_or(false);
        if !has {
            return;
        }
        unsafe {
            drain_done();
            let _ = PostMessageW(Some(hwnd()), WM_LALIA_RESTORE, WPARAM(0), LPARAM(0));
        }
        let _ = wait_done(Duration::from_secs(5));
    }

    /// Layer 1: clipboard paste with delayed render and restore.
    pub fn paste(text: &str, opts: &InsertOptions) -> InsertReport {
        ensure_started();
        let started = Instant::now();
        let st = STATE.get().unwrap();
        {
            let mut sh = st.shared.lock();
            sh.pending_text = Some(to_wide(text));
            sh.keystroke_at = None;
            sh.rendered_after_keystroke = 0;
        }
        unsafe {
            drain_done();
            let _ = PostMessageW(Some(hwnd()), WM_LALIA_PUBLISH, WPARAM(0), LPARAM(0));
        }
        // Ten seconds, not two. Taking a copy of the clipboard can make another
        // application render a format it had only promised, and a spreadsheet
        // with a large selection takes its time. Giving up early here used to
        // leave the clipboard emptied with nothing put back, which destroys
        // exactly the data this snapshot exists to protect.
        if let Err(e) = wait_done(Duration::from_secs(10)) {
            restore_clipboard_now();
            return InsertReport { outcome: InsertOutcome::Failed, method: "paste".into(), message: Some(e), elapsed_ms: started.elapsed().as_millis() as u64 };
        }
        // The stop key is often still physically held here: a toggle press lasts
        // about 100 ms and a short tail transcribes faster than that. AltGr held
        // down turns Ctrl+V into Ctrl+Alt+V, which no editor treats as paste, so
        // wait for the key to come up before synthesising releases.
        let held = wait_for_modifier_release(Duration::from_millis(400));
        if !held.is_empty() {
            tracing::debug!("paste: modifiers still held after 400 ms: {held:?}; releasing them");
        }
        release_modifiers();

        // Refuse to paste into a window that cannot hold text. Ctrl+V there is
        // accepted silently, so every signal we have would report success while
        // the user sees nothing arrive. Leave the text on the clipboard and say
        // so out loud instead.
        let fg_now = unsafe { GetForegroundWindow() };
        if let Some(what) = unusable_target(fg_now) {
            tracing::warn!("paste refused: the target is {what}; {}", focus_diagnostics());
            crate::journal::warn(
                "insert.refused",
                serde_json::json!({
                    "reason": "target cannot hold text",
                    "target": what,
                    "class": window_class(fg_now),
                    "chars": text.chars().count(),
                }),
            );
            unsafe {
                drain_done();
            let _ = PostMessageW(Some(hwnd()), WM_LALIA_SETTEXT, WPARAM(0), LPARAM(0));
            }
            // Same rule as below: only promise the clipboard when it took them.
            let message = match wait_done(Duration::from_secs(2)) {
                // A key, not a sentence. The window that shows this to the user
                // is the only place that knows which language they read, and
                // until 10 September 2026 every one of these came out in English
                // in an otherwise Greek program.
                Ok(()) => "msg_cannot_hold".to_string(),
                Err(e) => {
                    tracing::error!("{what} cannot hold text and the rescue copy failed too, the words are only in History: {e}");
                    "msg_cannot_hold_no_clipboard".to_string()
                }
            };
            return InsertReport {
                outcome: InsertOutcome::PasteNotConsumed,
                method: "paste".into(),
                message: Some(message),
                elapsed_ms: started.elapsed().as_millis() as u64,
            };
        }

        // Empty the notice channel before pressing anything. Windows keeps a
        // clipboard history and reads every new entry the instant it appears,
        // and that read leaves a notice here. Left in place it answers the
        // question below immediately and wrongly, while the target's own read
        // arrives after the app has already given up.
        //
        // Measured on 10 September 2026: two dictations into the same window
        // were reported as "the application did not accept the paste" 126 ms
        // and 180 ms after the key, far sooner than the 700 ms this code
        // believes it waits, because a stale notice was answering for them.
        {
            let rx = RENDER_RX.get().unwrap().lock();
            while rx.try_recv().is_ok() {}
        }
        let mut fg_pid = 0u32;
        unsafe {
            let fg = GetForegroundWindow();
            if !fg.is_invalid() {
                GetWindowThreadProcessId(fg, Some(&mut fg_pid));
            }
        }
        {
            let mut sh = st.shared.lock();
            sh.rendered_after_keystroke = 0;
            sh.rendered_by_target = 0;
            sh.other_readers.clear();
            sh.target_pid = fg_pid;
            sh.keystroke_at = Some(Instant::now());
        }
        // Do not press the key while somebody else holds the clipboard open.
        //
        // Measured 10 September 2026: msrdc.exe, the Remote Desktop client,
        // opens the clipboard 1 to 3 ms after every change and reads the
        // formats it forwards. Ctrl+V used to go out within microseconds of
        // publishing, so the target's own OpenClipboard landed inside that
        // window, failed with "busy", and Chromium dropped the paste without a
        // word. The text then sat on the clipboard while the user searched
        // History for it. A quarter of a second is far more than a read takes;
        // past it the key goes out anyway and the log says so.
        let wait_started = Instant::now();
        let free_by = wait_started + Duration::from_millis(250);
        let mut held_by: Option<String> = None;
        loop {
            let holder = unsafe { GetOpenClipboardWindow().ok().filter(|h| !h.is_invalid()) };
            let Some(h) = holder else { break };
            if held_by.is_none() {
                let mut pid = 0u32;
                unsafe { GetWindowThreadProcessId(h, Some(&mut pid)); }
                held_by = Some(process_name(pid));
            }
            if Instant::now() >= free_by {
                tracing::warn!("clipboard still held by {} 250 ms after publishing; pressing Ctrl+V anyway", held_by.as_deref().unwrap_or("?"));
                break;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        if let Some(who) = held_by {
            tracing::info!("waited {} ms for {} to let go of the clipboard before Ctrl+V", wait_started.elapsed().as_millis(), who);
        }
        send_paste_chord(opts.shift_paste);

        // Wait for the target to read the clipboard, then for traffic to go quiet.
        //
        // The counter, never the channel, decides. A notice means somebody read
        // the clipboard; only the counter says whether that somebody read it
        // after the key was pressed.
        //
        // One attempt, deliberately. A retry loop lived here and was dead code:
        // its condition broke out on the first pass, so it never ran. It stays
        // gone rather than being repaired, because on 5 September 2026 a paste
        // that Chromium did accept produced no render request at all, and six
        // retries pasted the same text six times. Until "did the target take
        // it" can be answered reliably, a second attempt risks doubling the
        // user's words, which is worse than asking them to press Ctrl+V.
        let attempts = 1u32;
        let deadline = Instant::now() + Duration::from_millis(700);
        let consumed = {
            let rx = RENDER_RX.get().unwrap().lock();
            loop {
                if st.shared.lock().rendered_by_target > 0 {
                    // Chromium reads twice; wait until no new read for settle_ms.
                    while rx.recv_timeout(Duration::from_millis(opts.settle_ms.max(60))).is_ok() {}
                    break true;
                }
                let left = deadline.saturating_duration_since(Instant::now());
                if left.is_zero() || rx.recv_timeout(left).is_err() {
                    break false;
                }
            }
        };

        // Somebody else read the clipboard first. Once a promised format has been
        // rendered for them, it is real data, and every later reader gets it
        // without asking us again: the target's own read leaves no trace at all.
        //
        // Measured 10 September 2026: msrdc.exe, the Remote Desktop client,
        // reads every clipboard change 1 to 3 ms later. With it running, this
        // code can never learn whether the window took the words.
        //
        // So it stops guessing. The words stay on the clipboard, the user is
        // told they are there, and nothing claims a failure that was never
        // established. Announcing a failure that did not happen sent the user
        // hunting through History for text that had already arrived.
        let interference = {
            let sh = st.shared.lock();
            !consumed && !sh.other_readers.is_empty()
        };
        if interference {
            match clipboard_holds(&to_wide(text)) {
                Some(true) => tracing::info!("the clipboard still holds our text after the paste"),
                Some(false) => tracing::warn!("the clipboard no longer holds our text: another program replaced it after we published"),
                None => tracing::debug!("could not read the clipboard back to check it"),
            }
        }
        if interference {
            let sh = st.shared.lock();
            tracing::info!("paste unverified: {} read the clipboard before the target could, so nothing here can tell whether it landed; the words stay on the clipboard",
                sh.other_readers.join(", "));
            drop(sh);
            unsafe {
                drain_done();
                let _ = PostMessageW(Some(hwnd()), WM_LALIA_SETTEXT, WPARAM(0), LPARAM(0));
            }
            let _ = wait_done(Duration::from_secs(2));
            return InsertReport {
                outcome: InsertOutcome::PastedNoRestore,
                method: "paste".into(),
                message: Some("msg_also_on_clipboard".into()),
                elapsed_ms: started.elapsed().as_millis() as u64,
            };
        }

        if !consumed {
            {
                let sh = st.shared.lock();
                let others = if sh.other_readers.is_empty() { "nobody".to_string() } else { sh.other_readers.join(", ") };
                tracing::warn!("paste not consumed: {} reads by the target, {} by others ({}), {}",
                    sh.rendered_by_target, sh.rendered_after_keystroke.saturating_sub(sh.rendered_by_target), others, focus_diagnostics());
            }
            crate::journal::warn(
                "insert.not_consumed",
                serde_json::json!({
                    "class": window_class(unsafe { GetForegroundWindow() }),
                    "chars": text.chars().count(),
                    "attempts": attempts,
                }),
            );
            // The app never asked for the data. Leave real text on the clipboard so the
            // user can paste by hand, and do not restore.
            unsafe {
                drain_done();
            let _ = PostMessageW(Some(hwnd()), WM_LALIA_SETTEXT, WPARAM(0), LPARAM(0));
            }
            // Whether the words really got there decides what the user is told.
            // Saying "they are on the clipboard" when the copy failed sends them
            // to press Ctrl+V on nothing, and that is how a dictation is lost
            // without anybody noticing.
            let on_clipboard = wait_done(Duration::from_secs(2));
            let message = match &on_clipboard {
                Ok(()) => "msg_not_taken".to_string(),
                Err(e) => {
                    tracing::error!("the rescue copy failed too, the words are only in History: {e}");
                    "msg_not_taken_no_clipboard".to_string()
                }
            };
            return InsertReport {
                outcome: InsertOutcome::PasteNotConsumed,
                method: "paste".into(),
                message: Some(message),
                elapsed_ms: started.elapsed().as_millis() as u64,
            };
        }

        // How the target consumed the paste. This decides how short settle_ms can
        // safely be: the clipboard must not be restored before the last read.
        {
            let sh = st.shared.lock();
            let last_ms = sh.last_render.zip(sh.keystroke_at).map(|(r, k)| r.saturating_duration_since(k).as_millis() as u64);
            let others = if sh.other_readers.is_empty() { "none".to_string() } else { sh.other_readers.join(", ") };
            tracing::debug!("paste: {} read(s) by the target, {} in total, last read {:?} ms after Ctrl+V, settle {} ms, other readers: {}",
                sh.rendered_by_target, sh.rendered_after_keystroke, last_ms, opts.settle_ms, others);
        }

        if opts.restore_clipboard {
            unsafe {
                drain_done();
            let _ = PostMessageW(Some(hwnd()), WM_LALIA_RESTORE, WPARAM(0), LPARAM(0));
            }
            match wait_done(Duration::from_secs(2)) {
                Ok(()) => InsertReport { outcome: InsertOutcome::Pasted, method: "paste".into(), message: None, elapsed_ms: started.elapsed().as_millis() as u64 },
                Err(e) => InsertReport { outcome: InsertOutcome::PastedNoRestore, method: "paste".into(), message: Some(e), elapsed_ms: started.elapsed().as_millis() as u64 },
            }
        } else {
            InsertReport { outcome: InsertOutcome::PastedNoRestore, method: "paste".into(), message: None, elapsed_ms: started.elapsed().as_millis() as u64 }
        }
    }

    /// Layer 3: only copy to the clipboard.
    pub fn copy_only(text: &str) -> InsertReport {
        ensure_started();
        let started = Instant::now();
        let st = STATE.get().unwrap();
        st.shared.lock().pending_text = Some(to_wide(text));
        unsafe {
            drain_done();
            let _ = PostMessageW(Some(hwnd()), WM_LALIA_SETTEXT, WPARAM(0), LPARAM(0));
        }
        match wait_done(Duration::from_secs(2)) {
            Ok(()) => InsertReport { outcome: InsertOutcome::CopiedOnly, method: "copy".into(), message: None, elapsed_ms: started.elapsed().as_millis() as u64 },
            Err(e) => InsertReport { outcome: InsertOutcome::Failed, method: "copy".into(), message: Some(e), elapsed_ms: started.elapsed().as_millis() as u64 },
        }
    }

    /// Layer 2: type the text as Unicode key events.
    pub fn type_text(text: &str) -> InsertReport {
        let started = Instant::now();
        release_modifiers();
        let units: Vec<u16> = text.encode_utf16().collect();
        let mut inputs: Vec<INPUT> = Vec::with_capacity(units.len() * 2);
        for u in units {
            if u == b'\n' as u16 {
                // Enter as a real key so editors treat it as a newline
                inputs.push(key(windows::Win32::UI::Input::KeyboardAndMouse::VK_RETURN, false));
                inputs.push(key(windows::Win32::UI::Input::KeyboardAndMouse::VK_RETURN, true));
                continue;
            }
            if u == b'\r' as u16 {
                continue;
            }
            for up in [false, true] {
                inputs.push(INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT { wVk: VIRTUAL_KEY(0), wScan: u, dwFlags: if up { KEYEVENTF_UNICODE | KEYEVENTF_KEYUP } else { KEYEVENTF_UNICODE }, time: 0, dwExtraInfo: crate::hotkey::LALIA_INJECT_SIG },
                    },
                });
            }
        }
        let mut sent = 0u32;
        unsafe {
            for chunk in inputs.chunks(64) {
                sent += SendInput(chunk, std::mem::size_of::<INPUT>() as i32);
                std::thread::sleep(Duration::from_millis(4));
            }
        }
        if sent as usize == inputs.len() {
            InsertReport { outcome: InsertOutcome::Typed, method: "type".into(), message: None, elapsed_ms: started.elapsed().as_millis() as u64 }
        } else {
            InsertReport { outcome: InsertOutcome::Failed, method: "type".into(), message: Some(format!("SendInput sent {sent} of {}", inputs.len())), elapsed_ms: started.elapsed().as_millis() as u64 }
        }
    }

    pub fn read_clipboard_string() -> Option<String> {
        ensure_started();
        unsafe {
            if !open_clipboard_retry(hwnd()) {
                return None;
            }
            let t = read_clipboard_text();
            let _ = CloseClipboard();
            t.map(|v| from_wide(&v))
        }
    }

    // ----- focus -----

    pub fn is_process_elevated(pid: u32) -> bool {
        unsafe {
            let Ok(proc_) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else { return false };
            let mut token = HANDLE::default();
            let elevated = if OpenProcessToken(proc_, TOKEN_QUERY, &mut token).is_ok() {
                let mut info = TOKEN_ELEVATION::default();
                let mut ret = 0u32;
                let ok = GetTokenInformation(token, TokenElevation, Some(&mut info as *mut _ as *mut core::ffi::c_void), std::mem::size_of::<TOKEN_ELEVATION>() as u32, &mut ret).is_ok();
                let _ = CloseHandle(token);
                ok && info.TokenIsElevated != 0
            } else {
                false
            };
            let _ = CloseHandle(proc_);
            elevated
        }
    }

    pub fn self_elevated() -> bool {
        unsafe {
            let mut token = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
                return false;
            }
            let mut info = TOKEN_ELEVATION::default();
            let mut ret = 0u32;
            let ok = GetTokenInformation(token, TokenElevation, Some(&mut info as *mut _ as *mut core::ffi::c_void), std::mem::size_of::<TOKEN_ELEVATION>() as u32, &mut ret).is_ok();
            let _ = CloseHandle(token);
            ok && info.TokenIsElevated != 0
        }
    }

    pub fn process_name(pid: u32) -> String {
        unsafe {
            let Ok(proc_) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else { return String::new() };
            let mut buf = vec![0u16; 1024];
            let mut len = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(proc_, PROCESS_NAME_WIN32, windows::core::PWSTR(buf.as_mut_ptr()), &mut len).is_ok();
            let _ = CloseHandle(proc_);
            if !ok {
                return String::new();
            }
            let full = String::from_utf16_lossy(&buf[..len as usize]);
            full.rsplit(['\\', '/']).next().unwrap_or("").to_string()
        }
    }

    pub fn capture_target() -> Target {
        unsafe {
            let h = GetForegroundWindow();
            if h.0.is_null() {
                return Target::default();
            }
            let mut pid = 0u32;
            let tid = GetWindowThreadProcessId(h, Some(&mut pid));
            let len = GetWindowTextLengthW(h);
            let mut title = String::new();
            if len > 0 {
                let mut buf = vec![0u16; len as usize + 1];
                let n = GetWindowTextW(h, &mut buf);
                title = String::from_utf16_lossy(&buf[..n as usize]);
            }
            let elevated = is_process_elevated(pid) && !self_elevated();
            Target { hwnd: h.0 as isize, thread_id: tid, process_id: pid, process_name: process_name(pid), title, elevated, is_password_field: false }
        }
    }

    pub fn foreground_hwnd() -> isize {
        unsafe { GetForegroundWindow().0 as isize }
    }

    pub fn window_alive(hwnd: isize) -> bool {
        unsafe { IsWindow(Some(HWND(hwnd as *mut core::ffi::c_void))).as_bool() }
    }

    /// Bring the target back to the front if focus drifted (for example to our
    /// own overlay). Uses the documented ALT-press trick when a plain
    /// SetForegroundWindow is refused.
    pub fn restore_focus(target: &Target) -> bool {
        unsafe {
            let h = HWND(target.hwnd as *mut core::ffi::c_void);
            if !IsWindow(Some(h)).as_bool() {
                return false;
            }
            if GetForegroundWindow() == h {
                return true;
            }
            if SetForegroundWindow(h).as_bool() && GetForegroundWindow() == h {
                return true;
            }
            let inputs = [key(VK_MENU, false), key(VK_MENU, true)];
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            std::thread::sleep(Duration::from_millis(20));
            let _ = SetForegroundWindow(h);
            std::thread::sleep(Duration::from_millis(30));
            GetForegroundWindow() == h
        }
    }

    #[allow(dead_code)]
    fn _unused(_: PCWSTR) {}
}

#[cfg(windows)]
pub use win::{capture_target, copy_only, ensure_started, foreground_hwnd, paste, read_clipboard_string, restore_focus, type_text, window_alive};

#[cfg(not(windows))]
pub fn capture_target() -> Target {
    Target::default()
}
