//! Global push-to-talk via a low-level keyboard hook (WH_KEYBOARD_LL).
//!
//! Why a hook and not RegisterHotKey: RegisterHotKey only reports key DOWN and
//! swallows the combination for every other app. Push-to-talk needs the key UP
//! too, must ignore auto-repeat, and must not steal chords the user did not ask for.
//!
//! The hook runs on its own thread with a message loop. The callback only touches
//! a small mutex-protected state and returns in microseconds (Windows silently
//! removes hooks that take longer than a few hundred milliseconds).

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChordId {
    PushToTalk,
    HandsFree,
    PasteLast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    Pressed(ChordId),
    Released(ChordId),
    /// The chord was held, but the user was typing rather than dictating.
    ///
    /// A shortcut made only of modifiers cannot tell "hold the right Alt to
    /// speak" apart from AltGr, which is the same physical key and is how a
    /// German keyboard types @, a French one types the euro sign and a Polish
    /// one types every accented letter. Both start with the same key going
    /// down. What separates them is what happens next: a character key while
    /// the modifier is held means the person is writing, so the recording is
    /// thrown away before anything is transcribed.
    Cancelled(ChordId),
    /// Escape pressed while the pipeline asked to capture it.
    Escape,
}

/// One member of a chord. Generic modifiers match either side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeySpec {
    Ctrl,
    Shift,
    Alt,
    Win,
    Vk(u16),
}

// Virtual-key codes (subset). Names follow the Win32 constants.
/// The two side buttons of a mouse. Windows reserves these numbers for them and
/// no keyboard ever sends them, so a chord can hold one without any ambiguity.
pub const VK_XBUTTON1: u16 = 0x05;
pub const VK_XBUTTON2: u16 = 0x06;
pub const VK_BACK: u16 = 0x08;
pub const VK_TAB: u16 = 0x09;
pub const VK_RETURN: u16 = 0x0D;
pub const VK_SHIFT: u16 = 0x10;
pub const VK_CONTROL: u16 = 0x11;
pub const VK_MENU: u16 = 0x12;
pub const VK_PAUSE: u16 = 0x13;
pub const VK_CAPITAL: u16 = 0x14;
pub const VK_ESCAPE: u16 = 0x1B;
pub const VK_SPACE: u16 = 0x20;
pub const VK_LWIN: u16 = 0x5B;
pub const VK_RWIN: u16 = 0x5C;
pub const VK_LSHIFT: u16 = 0xA0;
pub const VK_RSHIFT: u16 = 0xA1;
pub const VK_LCONTROL: u16 = 0xA2;
pub const VK_RCONTROL: u16 = 0xA3;
pub const VK_LMENU: u16 = 0xA4;
pub const VK_RMENU: u16 = 0xA5;
pub const VK_F1: u16 = 0x70;
pub const VK_SCROLL: u16 = 0x91;
pub const VK_INSERT: u16 = 0x2D;
pub const VK_HOME: u16 = 0x24;
pub const VK_END: u16 = 0x23;
pub const VK_PRIOR: u16 = 0x21;
pub const VK_NEXT: u16 = 0x22;
pub const VK_APPS: u16 = 0x5D;
/// Unassigned virtual key used to tell Windows "another key was pressed" so the
/// Start menu does not open when Win is released. Same trick AutoHotkey uses.
pub const VK_MASK: u16 = 0xE8;
/// Marker placed in dwExtraInfo on every key event Lalia itself injects, so the
/// hook can ignore its own paste chords and mask keys while still honouring
/// keys injected by other tools (AutoHotkey, remote desktop, accessibility).
pub const LALIA_INJECT_SIG: usize = 0x4C41_4C49;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord {
    pub keys: Vec<KeySpec>,
}

impl Chord {
    /// Parse strings like "Ctrl+Win", "RCtrl", "Ctrl+Alt+D", "F13", "CapsLock", "Right Shift".
    pub fn parse(text: &str) -> Result<Chord, String> {
        let mut keys = Vec::new();
        for raw in text.split('+') {
            let part = raw.trim();
            if part.is_empty() {
                continue;
            }
            let norm = part.to_ascii_lowercase().replace(' ', "");
            let spec = match norm.as_str() {
                "ctrl" | "control" | "strg" => KeySpec::Ctrl,
                "shift" => KeySpec::Shift,
                "alt" | "option" => KeySpec::Alt,
                "win" | "super" | "meta" | "windows" | "cmd" => KeySpec::Win,
                "lctrl" | "leftctrl" | "leftcontrol" => KeySpec::Vk(VK_LCONTROL),
                "rctrl" | "rightctrl" | "rightcontrol" => KeySpec::Vk(VK_RCONTROL),
                "lshift" | "leftshift" => KeySpec::Vk(VK_LSHIFT),
                "rshift" | "rightshift" => KeySpec::Vk(VK_RSHIFT),
                "lalt" | "leftalt" => KeySpec::Vk(VK_LMENU),
                "ralt" | "rightalt" | "altgr" => KeySpec::Vk(VK_RMENU),
                "lwin" | "leftwin" => KeySpec::Vk(VK_LWIN),
                "rwin" | "rightwin" => KeySpec::Vk(VK_RWIN),
                "capslock" | "caps" => KeySpec::Vk(VK_CAPITAL),
                "scrolllock" | "scroll" => KeySpec::Vk(VK_SCROLL),
                "pause" | "break" => KeySpec::Vk(VK_PAUSE),
                "space" => KeySpec::Vk(VK_SPACE),
                "tab" => KeySpec::Vk(VK_TAB),
                "enter" | "return" => KeySpec::Vk(VK_RETURN),
                "backspace" => KeySpec::Vk(VK_BACK),
                "insert" | "ins" => KeySpec::Vk(VK_INSERT),
                "home" => KeySpec::Vk(VK_HOME),
                "end" => KeySpec::Vk(VK_END),
                "pageup" | "pgup" => KeySpec::Vk(VK_PRIOR),
                "pagedown" | "pgdn" => KeySpec::Vk(VK_NEXT),
                "menu" | "apps" | "contextmenu" => KeySpec::Vk(VK_APPS),
                "mouse4" | "xbutton1" | "mouseback" => KeySpec::Vk(VK_XBUTTON1),
                "mouse5" | "xbutton2" | "mouseforward" => KeySpec::Vk(VK_XBUTTON2),
                "escape" | "esc" => return Err("Escape is reserved for cancel".into()),
                other => {
                    if let Some(num) = other.strip_prefix('f') {
                        if let Ok(n) = num.parse::<u16>() {
                            if (1..=24).contains(&n) {
                                keys.push(KeySpec::Vk(VK_F1 + n - 1));
                                continue;
                            }
                        }
                    }
                    let chars: Vec<char> = other.chars().collect();
                    if chars.len() == 1 {
                        let c = chars[0].to_ascii_uppercase();
                        if c.is_ascii_alphanumeric() {
                            keys.push(KeySpec::Vk(c as u16));
                            continue;
                        }
                    }
                    return Err(format!("Unknown key: {part}"));
                }
            };
            keys.push(spec);
        }
        if keys.is_empty() {
            return Err("Empty shortcut".into());
        }
        // A chord made only of a generic modifier and nothing else is allowed
        // (for example "Win" alone would be a bad idea, but "RCtrl" is fine).
        Ok(Chord { keys })
    }

    fn is_down(&self, down: &HashSet<u16>) -> bool {
        self.keys.iter().all(|k| match k {
            KeySpec::Ctrl => down.contains(&VK_LCONTROL) || down.contains(&VK_RCONTROL) || down.contains(&VK_CONTROL),
            KeySpec::Shift => down.contains(&VK_LSHIFT) || down.contains(&VK_RSHIFT) || down.contains(&VK_SHIFT),
            KeySpec::Alt => down.contains(&VK_LMENU) || down.contains(&VK_RMENU) || down.contains(&VK_MENU),
            KeySpec::Win => down.contains(&VK_LWIN) || down.contains(&VK_RWIN),
            KeySpec::Vk(vk) => down.contains(vk),
        })
    }

    /// True when the chord is nothing but modifier keys, such as a bare right
    /// Alt. Only these can be confused with typing, because every other chord
    /// needs a character key of its own to fire.
    fn is_all_modifiers(&self) -> bool {
        self.keys.iter().all(|k| match k {
            KeySpec::Vk(vk) => is_modifier_vk(*vk),
            _ => true,
        })
    }

    fn contains_win(&self) -> bool {
        self.keys.iter().any(|k| matches!(k, KeySpec::Win | KeySpec::Vk(VK_LWIN) | KeySpec::Vk(VK_RWIN)))
    }

    /// The non-modifier key of the chord, if any (the one we swallow so the
    /// focused app does not also receive it).
    fn main_key(&self) -> Option<u16> {
        self.keys.iter().find_map(|k| match k {
            KeySpec::Vk(vk) if !is_modifier_vk(*vk) => Some(*vk),
            _ => None,
        })
    }

    pub fn display(&self) -> String {
        self.keys
            .iter()
            .map(|k| match k {
                KeySpec::Ctrl => "Ctrl".to_string(),
                KeySpec::Shift => "Shift".to_string(),
                KeySpec::Alt => "Alt".to_string(),
                KeySpec::Win => "Win".to_string(),
                KeySpec::Vk(vk) => vk_name(*vk),
            })
            .collect::<Vec<_>>()
            .join("+")
    }
}

fn is_modifier_vk(vk: u16) -> bool {
    matches!(
        vk,
        VK_SHIFT | VK_CONTROL | VK_MENU | VK_LSHIFT | VK_RSHIFT | VK_LCONTROL | VK_RCONTROL | VK_LMENU | VK_RMENU | VK_LWIN | VK_RWIN
    )
}

pub fn vk_name(vk: u16) -> String {
    match vk {
        VK_LCONTROL => "LCtrl".into(),
        VK_RCONTROL => "RCtrl".into(),
        VK_LSHIFT => "LShift".into(),
        VK_RSHIFT => "RShift".into(),
        VK_LMENU => "LAlt".into(),
        VK_RMENU => "RAlt".into(),
        VK_LWIN => "LWin".into(),
        VK_RWIN => "RWin".into(),
        VK_CAPITAL => "CapsLock".into(),
        VK_SCROLL => "ScrollLock".into(),
        VK_PAUSE => "Pause".into(),
        VK_SPACE => "Space".into(),
        VK_TAB => "Tab".into(),
        VK_RETURN => "Enter".into(),
        VK_BACK => "Backspace".into(),
        VK_INSERT => "Insert".into(),
        VK_HOME => "Home".into(),
        VK_END => "End".into(),
        VK_PRIOR => "PageUp".into(),
        VK_NEXT => "PageDown".into(),
        VK_APPS => "Menu".into(),
        VK_XBUTTON1 => "Mouse4".into(),
        VK_XBUTTON2 => "Mouse5".into(),
        v if (VK_F1..VK_F1 + 24).contains(&v) => format!("F{}", v - VK_F1 + 1),
        v if (0x30..=0x39).contains(&v) || (0x41..=0x5A).contains(&v) => (v as u8 as char).to_string(),
        v => format!("VK{v:02X}"),
    }
}

struct Binding {
    id: ChordId,
    chord: Chord,
    active: bool,
    /// Set once a character key arrived while this all-modifier chord was held,
    /// so the cancel is announced once instead of once per letter typed.
    cancelled: bool,
    /// When the chord went down. Only a key typed in the first moments counts
    /// as typing; see the window below.
    since: Option<std::time::Instant>,
}

/// How soon after the modifier goes down a character key still means "this
/// person is writing, not speaking".
///
/// AltGr and its letter arrive together, as fast as two fingers can move.
/// Somebody who has been holding the key and talking for a second is
/// dictating, and a stray keypress then must never throw their words away.
const TYPING_WINDOW: Duration = Duration::from_millis(800);

struct HookState {
    bindings: Vec<Binding>,
    down: HashSet<u16>,
    /// When each held key was last reported down. Auto-repeat arrives every
    /// ~30 ms; a "repeat" after a long gap means the release was missed.
    last_down: std::collections::HashMap<u16, std::time::Instant>,
    /// The chord that most recently used the Win key and is active; we must
    /// mask the Win release so Start does not open.
    win_mask_pending: bool,
    sender: Option<tokio::sync::mpsc::UnboundedSender<HotkeyEvent>>,
}

static STATE: Lazy<Mutex<HookState>> = Lazy::new(|| {
    Mutex::new(HookState { bindings: Vec::new(), down: HashSet::new(), last_down: std::collections::HashMap::new(), win_mask_pending: false, sender: None })
});

/// When true, the hook swallows Escape and reports it (pipeline is recording).
pub static CAPTURE_ESCAPE: AtomicBool = AtomicBool::new(false);
/// When true, the hook ignores every chord (user is recording a new shortcut in settings).
pub static SUSPENDED: AtomicBool = AtomicBool::new(false);
/// Raw key reporting for the "press a shortcut" recorder in settings.
static RECORD_SENDER: Lazy<Mutex<Option<tokio::sync::mpsc::UnboundedSender<Vec<u16>>>>> = Lazy::new(|| Mutex::new(None));

/// One modifier key event as the hook saw it, for the key check in
/// Diagnostics. `at_ms` is Unix time in milliseconds.
#[derive(Clone, Debug, Serialize)]
pub struct SeenKey {
    pub at_ms: u64,
    pub key: String,
    pub down: bool,
    pub injected: bool,
}

const SEEN_KEYS_KEPT: usize = 40;

/// The last modifier key events, so a user can see what Windows delivered on
/// a day when "the hotkey does nothing". Modifier keys only, the same set the
/// trace logs: nothing typed is ever kept. On 10 September 2026 the right Alt
/// stopped arriving at Windows altogether while the left one kept coming, and
/// telling the two apart took forty minutes of log reading; this table shows
/// it in one glance.
static SEEN_KEYS: Lazy<Mutex<std::collections::VecDeque<SeenKey>>> =
    Lazy::new(|| Mutex::new(std::collections::VecDeque::with_capacity(SEEN_KEYS_KEPT)));

/// Called from inside the hook, so it never waits for the lock: a contended
/// write is dropped rather than stall a key press.
fn note_seen_key(vk: u16, down: bool, injected: bool) {
    let at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    if let Ok(mut q) = SEEN_KEYS.try_lock() {
        if q.len() >= SEEN_KEYS_KEPT {
            q.pop_front();
        }
        q.push_back(SeenKey { at_ms, key: vk_name(vk), down, injected });
    }
}

/// Newest first.
pub fn recent_keys() -> Vec<SeenKey> {
    SEEN_KEYS.lock().map(|q| q.iter().rev().cloned().collect()).unwrap_or_default()
}

/// One press of the right Alt on a layout that has AltGr, which includes the
/// Greek one, reaches Windows as two keys: a left Ctrl and the right Alt. The
/// shortcut recorder in Settings saw both and offered "Ctrl+RAlt" for a key the
/// user pressed alone, so the shortcut never matched afterwards and the key
/// looked dead. Windows always pairs them, so a genuine Ctrl plus right Alt
/// cannot be told apart and is given up on purpose.
pub fn drop_altgr_companion(mut keys: Vec<u16>) -> Vec<u16> {
    if keys.contains(&VK_RMENU) && keys.contains(&VK_LCONTROL) {
        keys.retain(|k| *k != VK_LCONTROL);
    }
    keys
}

/// How many times the hook has been put back in place. Shown in Diagnostics
/// next to how long the app has been running, so a rate can be read off it.
pub static HOOK_REHOOKS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub fn hook_rehooks() -> u32 {
    HOOK_REHOOKS.load(Ordering::Relaxed)
}

pub fn set_bindings(list: Vec<(ChordId, Chord)>) {
    let mut st = STATE.lock().unwrap();
    st.bindings = list.into_iter().map(|(id, chord)| Binding { id, chord, active: false, cancelled: false, since: None }).collect();
    st.down.clear();
    st.win_mask_pending = false;
}

pub fn start_recording_keys(tx: tokio::sync::mpsc::UnboundedSender<Vec<u16>>) {
    *RECORD_SENDER.lock().unwrap() = Some(tx);
    SUSPENDED.store(true, Ordering::Relaxed);
}

pub fn stop_recording_keys() {
    *RECORD_SENDER.lock().unwrap() = None;
    SUSPENDED.store(false, Ordering::Relaxed);
}

/// Called from the pipeline when it stops recording: forget stale key states so a
/// key released while the app was busy cannot leave a chord stuck.
///
/// Keys the user is still physically holding stay recorded and their chord stays
/// active. Clearing `active` while the key is held made the very next key-up
/// (the synthetic left Ctrl that Windows pairs with AltGr) look like a fresh
/// press, which restarted recording and re-entered hands-free mode.
pub fn reset_pressed_state() {
    let mut st = STATE.lock().unwrap();
    #[cfg(windows)]
    st.down.retain(|&vk| unsafe { (GetAsyncKeyState(vk as i32) as u16 & 0x8000) != 0 });
    let down = st.down.clone();
    st.last_down.retain(|vk, _| down.contains(vk));
    refresh_active(&mut st.bindings, &down);
}

/// A chord is active exactly while every key in it is held.
fn refresh_active(bindings: &mut [Binding], down: &HashSet<u16>) {
    for b in bindings.iter_mut() {
        let was = b.active;
        b.active = b.chord.is_down(down);
        if b.active && !was {
            b.since = Some(std::time::Instant::now());
        }
        if !b.active {
            b.since = None;
            b.cancelled = false;
        }
    }
}

#[cfg(windows)]
mod win {
    use super::*;
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetTimer, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
        KBDLLHOOKSTRUCT, LLKHF_INJECTED, LLMHF_INJECTED, MSLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP,
        WM_SYSKEYDOWN, WM_SYSKEYUP, WM_TIMER, WM_XBUTTONDOWN, WM_XBUTTONUP,
    };

    pub const SWALLOW: LRESULT = LRESULT(1);

    fn send_mask_key() {
        // Neutral key down+up. Windows sees "another key was pressed while Win
        // was held" and does not open the Start menu on Win release.
        let mk = |flags: KEYBD_EVENT_FLAGS| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT { wVk: VIRTUAL_KEY(VK_MASK), wScan: 0, dwFlags: flags, time: 0, dwExtraInfo: LALIA_INJECT_SIG },
            },
        };
        let inputs = [mk(KEYBD_EVENT_FLAGS(0)), mk(KEYEVENTF_KEYUP)];
        unsafe {
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }

    /// The mouse side buttons, through the hook that carries them.
    ///
    /// This callback runs for every mouse movement on the machine, so the first
    /// thing it does is decide it has nothing to do. Only the two side buttons
    /// go any further, and they take the same road as a key press from there on.
    unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code < 0 {
            return CallNextHookEx(None, code, wparam, lparam);
        }
        let msg = wparam.0 as u32;
        let is_down = msg == WM_XBUTTONDOWN;
        let is_up = msg == WM_XBUTTONUP;
        if !is_down && !is_up {
            return CallNextHookEx(None, code, wparam, lparam);
        }
        let info = &*(lparam.0 as *const MSLLHOOKSTRUCT);
        // Which of the two: the number lives in the high half of mouseData.
        let vk = match (info.mouseData >> 16) as u16 {
            1 => VK_XBUTTON1,
            2 => VK_XBUTTON2,
            _ => return CallNextHookEx(None, code, wparam, lparam),
        };
        if (info.flags & LLMHF_INJECTED) != 0 {
            return CallNextHookEx(None, code, wparam, lparam);
        }
        note_seen_key(vk, is_down, false);
        if dispatch_key(vk, is_down) {
            SWALLOW
        } else {
            CallNextHookEx(None, code, wparam, lparam)
        }
    }

    unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code < 0 {
            return CallNextHookEx(None, code, wparam, lparam);
        }
        let info = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        let own_injection = (info.flags.0 & LLKHF_INJECTED.0) != 0 && info.dwExtraInfo == LALIA_INJECT_SIG;
        let vk = info.vkCode as u16;
        let msg = wparam.0 as u32;
        let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let is_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;
        if own_injection || (!is_down && !is_up) || vk == VK_MASK {
            return CallNextHookEx(None, code, wparam, lparam);
        }
        // Modifier keys only: they carry no typed text, and they are the keys
        // every chord is built from, so this is enough to reconstruct a chord
        // sequence from the log without ever recording what the user types.
        if matches!(vk, VK_LMENU | VK_RMENU | VK_LCONTROL | VK_RCONTROL | VK_LWIN | VK_RWIN) {
            let injected = (info.flags.0 & LLKHF_INJECTED.0) != 0;
            tracing::debug!(
                "key {} {} flags={:#x} injected={}",
                vk_name(vk),
                if is_down { "down" } else { "up" },
                info.flags.0,
                injected
            );
            note_seen_key(vk, is_down, injected);
        }

        if dispatch_key(vk, is_down) {
            SWALLOW
        } else {
            CallNextHookEx(None, code, wparam, lparam)
        }
    }

    /// The part both hooks share: remember what is held, decide which chords
    /// just went down or up, and say whether this press belongs to one of them
    /// and must not reach the rest of Windows.
    ///
    /// The keyboard and the mouse arrive through two different Windows hooks
    /// that have nothing in common, so the mouse buttons used to be invisible
    /// to the whole program. They now meet here, which is why a side button can
    /// hold a shortcut exactly like a key.
    fn dispatch_key(vk: u16, is_down: bool) -> bool {
        let is_up = !is_down;
        // Shortcut recorder in settings: report raw keys, swallow nothing.
        if SUSPENDED.load(Ordering::Relaxed) {
            if let Ok(mut st) = STATE.try_lock() {
                if is_down {
                    st.down.insert(vk);
                } else {
                    st.down.remove(&vk);
                }
                let snapshot: Vec<u16> = st.down.iter().copied().collect();
                if let Some(tx) = RECORD_SENDER.lock().unwrap().as_ref() {
                    if is_down {
                        let _ = tx.send(snapshot);
                    }
                }
            }
            // The settings screen is a web page and XBUTTON1 is the browser's
            // back button, so letting it through while the user is recording a
            // shortcut would navigate the page away under them. Keys are left
            // alone: they carry no such meaning here.
            return matches!(vk, VK_XBUTTON1 | VK_XBUTTON2);
        }

        if vk == VK_ESCAPE && is_down && CAPTURE_ESCAPE.load(Ordering::Relaxed) {
            if let Ok(st) = STATE.try_lock() {
                if let Some(tx) = &st.sender {
                    let _ = tx.send(HotkeyEvent::Escape);
                }
            }
            return true;
        }
        if vk == VK_ESCAPE && is_up && CAPTURE_ESCAPE.load(Ordering::Relaxed) {
            return true;
        }

        let mut swallow = false;
        let lock_wait = std::time::Instant::now();
        // The pipeline takes this lock for microseconds when a recording stops,
        // which is exactly when the stop key is released. Dropping that key-up
        // left the chord "held" and swallowed the next press, so wait briefly.
        let mut st = loop {
            match STATE.try_lock() {
                Ok(g) => break g,
                Err(std::sync::TryLockError::WouldBlock) => {}
                Err(_) => return false,
            }
            if lock_wait.elapsed() > Duration::from_millis(3) {
                return false;
            }
            std::thread::yield_now();
        };

        let mut was_down = st.down.contains(&vk);
        if is_down {
            if was_down {
                let stale = st.last_down.get(&vk).map(|t| t.elapsed() > Duration::from_millis(350)).unwrap_or(true);
                if !stale {
                    // auto-repeat: state unchanged, but keep swallowing a chord's main key
                    st.last_down.insert(vk, std::time::Instant::now());
                    let repeat_main = st.bindings.iter().any(|b| b.active && b.chord.main_key() == Some(vk));
                    drop(st);
                    return repeat_main;
                }
                // A "repeat" after a long silence is a fresh press whose earlier
                // release never reached us. Apply the missed release first.
                st.down.remove(&vk);
                let snapshot = st.down.clone();
                let mut released = Vec::new();
                for b in st.bindings.iter_mut() {
                    if b.active && !b.chord.is_down(&snapshot) {
                        b.active = false;
                        released.push(b.id);
                    }
                }
                if let Some(tx) = &st.sender {
                    for id in released {
                        let _ = tx.send(HotkeyEvent::Released(id));
                    }
                }
                was_down = false;
            }
            st.down.insert(vk);
            st.last_down.insert(vk, std::time::Instant::now());
        } else {
            st.down.remove(&vk);
            st.last_down.remove(&vk);
        }
        let _ = was_down;

        let is_win = vk == VK_LWIN || vk == VK_RWIN;
        // A character key going down. Mouse side buttons count as characters
        // here: clicking one while the dictation modifier is held is not
        // speech either.
        let typed = is_down && !is_modifier_vk(vk) && vk != VK_ESCAPE;
        let mut fired: Vec<HotkeyEvent> = Vec::new();
        let mut any_pressed = false;
        let mut mask_pending = false;
        let down_snapshot = st.down.clone();
        for b in st.bindings.iter_mut() {
            let now = b.chord.is_down(&down_snapshot);
            if now && !b.active {
                b.active = true;
                b.cancelled = false;
                b.since = Some(std::time::Instant::now());
                any_pressed = true;
                fired.push(HotkeyEvent::Pressed(b.id));
                if b.chord.main_key() == Some(vk) {
                    swallow = true;
                }
                if b.chord.contains_win() {
                    mask_pending = true;
                }
            } else if !now && b.active {
                b.active = false;
                b.cancelled = false;
                b.since = None;
                fired.push(HotkeyEvent::Released(b.id));
                if b.chord.main_key() == Some(vk) {
                    swallow = true;
                }
            }
        }
        // Typing, decided only after every shortcut has had its say. A key that
        // completes another shortcut is not typing: holding the right Alt and
        // adding Space is how hands free mode starts, and treating that Space
        // as a letter would throw the recording away at the very moment the
        // user asked for more of it.
        if typed && !any_pressed {
            for b in st.bindings.iter_mut() {
                let fresh = b.since.map(|t| t.elapsed() < TYPING_WINDOW).unwrap_or(false);
                if b.active && fresh && !b.cancelled && b.chord.is_all_modifiers() {
                    b.cancelled = true;
                    fired.push(HotkeyEvent::Cancelled(b.id));
                }
            }
        }
        if mask_pending {
            st.win_mask_pending = true;
        }
        if is_win && is_up && st.win_mask_pending {
            st.win_mask_pending = false;
            send_mask_key();
        }
        if let Some(tx) = &st.sender {
            for ev in fired {
                let _ = tx.send(ev);
            }
        }
        drop(st);
        swallow
    }

    pub fn run_hook_thread() {
        unsafe {
            let hmod = GetModuleHandleW(None).ok().map(|h| windows::Win32::Foundation::HINSTANCE(h.0));
            let hook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), hmod, 0) {
                Ok(h) => h,
                Err(e) => {
                    tracing::error!("SetWindowsHookExW failed: {e}");
                    return;
                }
            };
            tracing::info!("keyboard hook installed ({:?})", hook);
            // The mouse needs its own hook. Without it the side buttons never
            // reach this program at all, which is why they could not hold a
            // shortcut before 10 September 2026.
            let mut mouse = match SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), hmod, 0) {
                Ok(h) => {
                    tracing::info!("mouse hook installed ({:?})", h);
                    Some(h)
                }
                Err(e) => {
                    tracing::warn!("mouse hook failed, the side buttons will not work: {e}");
                    None
                }
            };
            // Windows removes a low-level hook without telling anyone when one
            // callback overruns LowLevelHooksTimeout. Nothing reports it, and
            // there is no way to ask whether a hook is still installed, so the
            // only cure is to keep putting a fresh one in place: install the new
            // one first, then drop the old, so no key press falls in between.
            //
            // This used to run every 30 s, which meant that after a death the
            // user pressed a dead key for up to half a minute. Measured on
            // 10 September 2026 on this machine at 100% CPU with 732 processes:
            // between 10:20:35 and 10:22:12 the keyboard sent eleven key events
            // (confirmed by a raw-input listener that names the device) and the
            // hook saw none of them. One second costs two cheap kernel calls and
            // makes the worst case unnoticeable.
            let mut hook = hook;
            let _ = SetTimer(None, 1, 1_000, None);
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                if msg.message == WM_TIMER {
                    match SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), hmod, 0) {
                        Ok(fresh) => {
                            let _ = UnhookWindowsHookEx(hook);
                            hook = fresh;
                            let n = HOOK_REHOOKS.fetch_add(1, Ordering::Relaxed) + 1;
                            // Once a minute at this cadence, enough to show the
                            // loop is alive without filling the log.
                            if n % 60 == 1 {
                                tracing::debug!("keyboard hook re-registered ({n} times so far)");
                            }
                        }
                        Err(e) => tracing::warn!("keyboard hook re-registration failed: {e}"),
                    }
                    // The mouse hook dies the same silent death as the keyboard
                    // one, so it is replaced on the same beat.
                    match SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), hmod, 0) {
                        Ok(fresh) => {
                            if let Some(old) = mouse.replace(fresh) {
                                let _ = UnhookWindowsHookEx(old);
                            }
                        }
                        Err(e) => tracing::warn!("mouse hook re-registration failed: {e}"),
                    }
                    continue;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

/// Install the hook on a dedicated thread. Events arrive on `tx`.
pub fn install(tx: tokio::sync::mpsc::UnboundedSender<HotkeyEvent>) {
    STATE.lock().unwrap().sender = Some(tx);
    #[cfg(windows)]
    {
        std::thread::Builder::new()
            .name("lalia-keyboard-hook".into())
            .spawn(win::run_hook_thread)
            .expect("spawn hook thread");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_chords() {
        assert_eq!(Chord::parse("Ctrl+Win").unwrap().keys, vec![KeySpec::Ctrl, KeySpec::Win]);
        assert_eq!(Chord::parse("RCtrl").unwrap().keys, vec![KeySpec::Vk(VK_RCONTROL)]);
        assert_eq!(Chord::parse("Ctrl+Alt+D").unwrap().keys, vec![KeySpec::Ctrl, KeySpec::Alt, KeySpec::Vk(0x44)]);
        assert_eq!(Chord::parse("F13").unwrap().keys, vec![KeySpec::Vk(VK_F1 + 12)]);
        assert!(Chord::parse("Esc").is_err());
        assert!(Chord::parse("").is_err());
    }

    #[test]
    fn chord_activation_needs_all_keys() {
        let c = Chord::parse("Ctrl+Win").unwrap();
        let mut down = HashSet::new();
        down.insert(VK_LCONTROL);
        assert!(!c.is_down(&down));
        down.insert(VK_RWIN);
        assert!(c.is_down(&down));
        assert!(c.contains_win());
        assert_eq!(c.main_key(), None);
        assert_eq!(Chord::parse("Ctrl+Alt+D").unwrap().main_key(), Some(0x44));
    }

    #[test]
    fn reset_keeps_a_held_chord_active() {
        // The pipeline resets hotkey state while the user may still be holding
        // the key. A held chord must stay active, otherwise the next key-up is
        // reported as a fresh press.
        let mut bindings = vec![
            Binding { id: ChordId::PushToTalk, chord: Chord::parse("RAlt").unwrap(), active: false, cancelled: false, since: None },
            Binding { id: ChordId::HandsFree, chord: Chord::parse("RAlt+Space").unwrap(), active: true, cancelled: false, since: None },
        ];
        let mut down = HashSet::new();
        down.insert(VK_RMENU);
        refresh_active(&mut bindings, &down);
        assert!(bindings[0].active, "RAlt is held, so the chord stays active");
        assert!(!bindings[1].active, "Space is up, so RAlt+Space is released");
        down.clear();
        refresh_active(&mut bindings, &down);
        assert!(!bindings[0].active);
    }

    /// AltGr and "hold the right Alt to speak" are the same physical key.
    /// A German keyboard needs it for @, a French one for the euro sign, a
    /// Polish one for every accented letter. Only a bare modifier chord can be
    /// confused this way, so only a bare modifier chord may be cancelled.
    #[test]
    fn typing_while_a_bare_modifier_is_held_is_not_dictation() {
        assert!(Chord::parse("RAlt").unwrap().is_all_modifiers());
        assert!(Chord::parse("Ctrl+Alt").unwrap().is_all_modifiers());
        assert!(Chord::parse("RAlt").unwrap().main_key().is_none());

        // Anything with a character key of its own is safe: nobody types a
        // letter by holding this combination, so it is never cancelled.
        assert!(!Chord::parse("RAlt+Space").unwrap().is_all_modifiers());
        assert!(!Chord::parse("Ctrl+Shift+D").unwrap().is_all_modifiers());
        assert!(!Chord::parse("Mouse4").unwrap().is_all_modifiers());
        assert!(Chord::parse("RAlt+Space").unwrap().main_key().is_some());
    }

    #[test]
    fn display_round_trip() {
        let c = Chord::parse("Ctrl+Shift+F9").unwrap();
        assert_eq!(c.display(), "Ctrl+Shift+F9");
    }

    /// The two side buttons of a mouse must survive the round trip through the
    /// chord parser and the name table, because a shortcut is stored as text.
    #[test]
    fn a_mouse_side_button_can_hold_a_shortcut() {
        for (text, vk) in [("Mouse4", VK_XBUTTON1), ("Mouse5", VK_XBUTTON2)] {
            let chord = Chord::parse(text).expect("the parser must accept a side button");
            assert_eq!(chord.main_key(), Some(vk), "{text} must be the main key");
            assert_eq!(vk_name(vk), text, "the name must come back unchanged");
        }
        // And they combine with modifiers like any other key.
        let chord = Chord::parse("Ctrl+Mouse5").expect("a modifier plus a side button");
        assert_eq!(chord.main_key(), Some(VK_XBUTTON2));
    }

    #[test]
    fn recording_the_right_alt_alone_does_not_become_ctrl_plus_right_alt() {
        // What Windows delivers for one press of the right Alt on a Greek layout.
        assert_eq!(drop_altgr_companion(vec![VK_LCONTROL, VK_RMENU]), vec![VK_RMENU]);
        // A real Ctrl chord with any other key is left alone.
        assert_eq!(drop_altgr_companion(vec![VK_LCONTROL, VK_LMENU]), vec![VK_LCONTROL, VK_LMENU]);
        assert_eq!(drop_altgr_companion(vec![VK_LCONTROL, VK_LWIN]), vec![VK_LCONTROL, VK_LWIN]);
        // Right Ctrl is a key the user really pressed, so it stays.
        assert_eq!(drop_altgr_companion(vec![VK_RCONTROL, VK_RMENU]), vec![VK_RCONTROL, VK_RMENU]);
    }

    /// The key check keeps the newest events first and never grows past its cap.
    #[test]
    fn seen_keys_are_newest_first_and_capped() {
        for i in 0..(SEEN_KEYS_KEPT as u16 + 10) {
            note_seen_key(if i % 2 == 0 { VK_LMENU } else { VK_RMENU }, true, false);
        }
        let seen = recent_keys();
        assert_eq!(seen.len(), SEEN_KEYS_KEPT);
        assert_eq!(seen[0].key, "RAlt", "the last press written (an odd index) comes first");
        assert_eq!(seen[1].key, "LAlt");
        assert!(seen[0].at_ms >= seen[seen.len() - 1].at_ms);
    }
}
