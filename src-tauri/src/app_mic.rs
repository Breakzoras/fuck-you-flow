//! Dictation through the app's own microphone button.
//!
//! Some apps have a dictation button of their own next to the text box: the
//! Claude desktop app ("Press and hold to record") and the ChatGPT desktop app
//! that hosts Codex ("Dictate"). When the user turns this on and presses the
//! dictation shortcut while one of those windows is in front, FU Flow presses
//! that button instead of recording itself, and presses it again to stop. The
//! app hears the voice and writes the text into its own box; FU Flow records
//! nothing, transcribes nothing and pastes nothing.
//!
//! When the button cannot be found (a settings page is open, the app changed
//! its layout) the dictation runs locally as before, so a press is never lost.

/// An app whose microphone button FU Flow knows how to press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppKind {
    Claude,
    /// The ChatGPT desktop app, where Codex lives.
    ChatGpt,
}

impl AppKind {
    pub fn label(self) -> &'static str {
        match self {
            AppKind::Claude => "Claude",
            AppKind::ChatGpt => "Codex",
        }
    }
}

/// Which app a window belongs to, from its process name.
pub fn kind_for_process(process_name: &str) -> Option<AppKind> {
    let name = process_name.trim().to_ascii_lowercase();
    let name = name.strip_suffix(".exe").unwrap_or(&name);
    match name {
        "claude" => Some(AppKind::Claude),
        "chatgpt" => Some(AppKind::ChatGpt),
        _ => None,
    }
}

/// How the shortcut that started the routed dictation is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteMode {
    /// Held: the release stops it.
    Hold,
    /// Pressed once: the next press stops it.
    Toggle,
}

/// The shortcut events the router cares about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    HoldDown,
    HoldUp { tap: bool },
    ToggleDown,
    /// A character typed while the held shortcut was down: AltGr typing.
    Typed,
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    /// Not ours: the normal local dictation handles it.
    Pass,
    /// Swallow it and do nothing.
    Ignore,
    /// Try the app's button; if it is not there, `Pass`.
    Start(RouteMode),
    /// Press the button again. `cancel` drops the words where the app can.
    Stop { cancel: bool },
    /// A quick tap of the hold shortcut: keep listening until the next press,
    /// the same as a local dictation does.
    KeepListening,
}

/// What a key does, given the routed dictation going on (if any) and whether
/// a local one is going on. A local dictation is never interrupted.
pub fn decide(key: Key, routed: Option<RouteMode>, local_busy: bool) -> Act {
    match routed {
        Some(mode) => match key {
            Key::HoldDown => if mode == RouteMode::Toggle { Act::Stop { cancel: false } } else { Act::Ignore },
            Key::HoldUp { tap } => match mode {
                RouteMode::Hold if tap => Act::KeepListening,
                RouteMode::Hold => Act::Stop { cancel: false },
                RouteMode::Toggle => Act::Ignore,
            },
            Key::ToggleDown => Act::Stop { cancel: false },
            Key::Typed => if mode == RouteMode::Hold { Act::Stop { cancel: true } } else { Act::Ignore },
            Key::Escape => Act::Stop { cancel: true },
        },
        None if local_busy => Act::Pass,
        None => match key {
            Key::HoldDown => Act::Start(RouteMode::Hold),
            Key::ToggleDown => Act::Start(RouteMode::Toggle),
            _ => Act::Pass,
        },
    }
}

/// The button names each app shows, in English (the desktop apps' own
/// labels). `start` is the idle button, `stop` ends and keeps the words,
/// `cancel` ends and drops them.
pub struct Names {
    pub start: &'static [&'static str],
    pub stop: &'static [&'static str],
    pub cancel: &'static [&'static str],
}

pub fn names(kind: AppKind) -> Names {
    match kind {
        // One toggle button that changes its name while it listens.
        AppKind::Claude => Names { start: &["Press and hold to record", "Dictate", "Start dictation"], stop: &["Stop dictation", "Stop recording"], cancel: &[] },
        // "Transcribe and send" is never pressed: it would send the message.
        AppKind::ChatGpt => Names { start: &["Dictate", "Start dictation"], stop: &["Stop dictation"], cancel: &["Cancel dictation"] },
    }
}

#[cfg(windows)]
pub use win::{is_listening, start, stop};

#[cfg(not(windows))]
pub fn start(_hwnd: isize, _kind: AppKind, _budget_ms: u64) -> Result<(), String> {
    Err("not supported on this system".into())
}
#[cfg(not(windows))]
pub fn stop(_hwnd: isize, _kind: AppKind, _cancel: bool) -> Result<(), String> {
    Err("not supported on this system".into())
}
#[cfg(not(windows))]
pub fn is_listening(_hwnd: isize, _kind: AppKind) -> Option<bool> {
    None
}

#[cfg(windows)]
mod win {
    use super::{names, AppKind};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use windows::Win32::System::Variant::VARIANT;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
    use windows::Win32::UI::Accessibility::{
        CUIAutomation, IUIAutomation, IUIAutomationCondition, IUIAutomationElement, IUIAutomationInvokePattern, IUIAutomationTogglePattern, ToggleState_On,
        TreeScope_Descendants, UIA_InvokePatternId, UIA_NamePropertyId, UIA_TogglePatternId,
    };

    unsafe fn automation() -> Option<IUIAutomation> {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()
    }

    /// The first element in the window named one of `wanted`.
    unsafe fn find(auto: &IUIAutomation, hwnd: isize, wanted: &[&str]) -> Option<IUIAutomationElement> {
        if wanted.is_empty() {
            return None;
        }
        let root = auto.ElementFromHandle(HWND(hwnd as *mut core::ffi::c_void)).ok()?;
        let mut cond: Option<IUIAutomationCondition> = None;
        for name in wanted {
            let one = auto.CreatePropertyCondition(UIA_NamePropertyId, &VARIANT::from(*name)).ok()?;
            cond = Some(match cond {
                None => one,
                Some(prev) => auto.CreateOrCondition(&prev, &one).ok()?,
            });
        }
        let el = root.FindFirst(TreeScope_Descendants, &cond?).ok()?;
        // Hidden copies (a collapsed panel) do not count.
        if el.CurrentIsOffscreen().map(|b| b.as_bool()).unwrap_or(false) {
            return None;
        }
        Some(el)
    }

    /// Clicks the element: Invoke where it has it, else Toggle.
    unsafe fn press(el: &IUIAutomationElement) -> Result<(), String> {
        if let Ok(p) = el.GetCurrentPatternAs::<IUIAutomationInvokePattern>(UIA_InvokePatternId) {
            return p.Invoke().map_err(|e| e.to_string());
        }
        if let Ok(p) = el.GetCurrentPatternAs::<IUIAutomationTogglePattern>(UIA_TogglePatternId) {
            return p.Toggle().map_err(|e| e.to_string());
        }
        Err("the button cannot be pressed".into())
    }

    unsafe fn toggled_on(el: &IUIAutomationElement) -> Option<bool> {
        let p = el.GetCurrentPatternAs::<IUIAutomationTogglePattern>(UIA_TogglePatternId).ok()?;
        p.CurrentToggleState().ok().map(|s| s == ToggleState_On)
    }

    /// Presses the app's microphone button. Gives up after `budget_ms`; a
    /// search that finishes later presses nothing, because by then the
    /// dictation is running locally.
    ///
    /// Electron apps build their accessibility tree only when first asked, so
    /// the first search after the app starts can come back empty: it is tried
    /// once more after a short wait.
    pub fn start(hwnd: isize, kind: AppKind, budget_ms: u64) -> Result<(), String> {
        let abandoned = Arc::new(AtomicBool::new(false));
        let flag = abandoned.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || unsafe {
            let result = (|| {
                let auto = automation().ok_or("UI Automation is not available")?;
                let n = names(kind);
                let mut el = find(&auto, hwnd, n.start);
                if el.is_none() {
                    // Already listening (started by hand): leave it on.
                    if find(&auto, hwnd, n.stop).is_some() {
                        return Ok(());
                    }
                    std::thread::sleep(Duration::from_millis(150));
                    el = find(&auto, hwnd, n.start);
                }
                let el = el.ok_or("no microphone button in the window")?;
                if kind == AppKind::Claude && toggled_on(&el) == Some(true) {
                    return Ok(());
                }
                if flag.load(Ordering::SeqCst) {
                    return Err("too slow".to_string());
                }
                press(&el)
            })();
            let _ = tx.send(result);
        });
        match rx.recv_timeout(Duration::from_millis(budget_ms)) {
            Ok(r) => r,
            Err(_) => {
                abandoned.store(true, Ordering::SeqCst);
                Err(format!("no answer from the window within {budget_ms} ms"))
            }
        }
    }

    /// Presses the button again. `cancel` uses the app's cancel button where
    /// it has one.
    pub fn stop(hwnd: isize, kind: AppKind, cancel: bool) -> Result<(), String> {
        run(move || unsafe {
            let auto = automation().ok_or("UI Automation is not available")?;
            let n = names(kind);
            if cancel {
                if let Some(el) = find(&auto, hwnd, n.cancel) {
                    return press(&el);
                }
            }
            if let Some(el) = find(&auto, hwnd, n.stop) {
                return press(&el);
            }
            // Claude's single button: toggled on means listening.
            if kind == AppKind::Claude {
                if let Some(el) = find(&auto, hwnd, n.start) {
                    if toggled_on(&el) == Some(true) {
                        return press(&el);
                    }
                }
            }
            Err("the app is no longer listening".to_string())
        })
        .unwrap_or_else(|| Err("the search stopped".into()))
    }

    /// Whether the app still listens. `None` when the window cannot be read.
    pub fn is_listening(hwnd: isize, kind: AppKind) -> Option<bool> {
        run(move || unsafe {
            let auto = automation()?;
            let n = names(kind);
            if find(&auto, hwnd, n.stop).is_some() {
                return Some(true);
            }
            let el = find(&auto, hwnd, n.start)?;
            Some(kind == AppKind::Claude && toggled_on(&el) == Some(true))
        })
        .flatten()
    }

    /// On a fresh thread each time, so COM state never leaks into the caller.
    fn run<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Option<T> {
        std::thread::spawn(f).join().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_apps_are_known_by_their_process() {
        assert_eq!(kind_for_process("claude.exe"), Some(AppKind::Claude));
        assert_eq!(kind_for_process("Claude.exe"), Some(AppKind::Claude));
        assert_eq!(kind_for_process("ChatGPT.exe"), Some(AppKind::ChatGpt));
        assert_eq!(kind_for_process("chatgpt"), Some(AppKind::ChatGpt));
        // The command-line Codex and other programs keep local dictation.
        assert_eq!(kind_for_process("codex.exe"), None);
        assert_eq!(kind_for_process("chrome.exe"), None);
        assert_eq!(kind_for_process(""), None);
    }

    #[test]
    fn a_press_starts_and_the_next_one_stops() {
        assert_eq!(decide(Key::ToggleDown, None, false), Act::Start(RouteMode::Toggle));
        assert_eq!(decide(Key::ToggleDown, Some(RouteMode::Toggle), false), Act::Stop { cancel: false });
        // The release of the toggle key does nothing.
        assert_eq!(decide(Key::HoldUp { tap: false }, Some(RouteMode::Toggle), false), Act::Ignore);
    }

    #[test]
    fn holding_stops_on_release_and_a_tap_keeps_listening() {
        assert_eq!(decide(Key::HoldDown, None, false), Act::Start(RouteMode::Hold));
        assert_eq!(decide(Key::HoldUp { tap: false }, Some(RouteMode::Hold), false), Act::Stop { cancel: false });
        assert_eq!(decide(Key::HoldUp { tap: true }, Some(RouteMode::Hold), false), Act::KeepListening);
        // After a tap the hold shortcut works as a toggle.
        assert_eq!(decide(Key::HoldDown, Some(RouteMode::Toggle), false), Act::Stop { cancel: false });
    }

    #[test]
    fn escape_and_altgr_typing_cancel() {
        assert_eq!(decide(Key::Escape, Some(RouteMode::Toggle), false), Act::Stop { cancel: true });
        assert_eq!(decide(Key::Typed, Some(RouteMode::Hold), false), Act::Stop { cancel: true });
        // Typing during a toggled dictation is the person writing; it goes on.
        assert_eq!(decide(Key::Typed, Some(RouteMode::Toggle), false), Act::Ignore);
    }

    #[test]
    fn a_local_dictation_is_never_taken_over() {
        assert_eq!(decide(Key::ToggleDown, None, true), Act::Pass);
        assert_eq!(decide(Key::HoldDown, None, true), Act::Pass);
        assert_eq!(decide(Key::Escape, None, false), Act::Pass);
    }

    /// Live check on a real window: FYF_HWND=<window handle> FYF_APP=claude|chatgpt.
    /// Opens the app's microphone for three seconds, then closes it.
    #[test]
    #[ignore]
    fn live_press_and_release() {
        let hwnd: isize = std::env::var("FYF_HWND").expect("FYF_HWND").parse().unwrap();
        let kind = kind_for_process(&std::env::var("FYF_APP").expect("FYF_APP")).unwrap();
        let t = std::time::Instant::now();
        start(hwnd, kind, 700).expect("start");
        println!("started in {} ms", t.elapsed().as_millis());
        std::thread::sleep(std::time::Duration::from_secs(3));
        assert_eq!(is_listening(hwnd, kind), Some(true), "listening after start");
        stop(hwnd, kind, false).expect("stop");
        std::thread::sleep(std::time::Duration::from_millis(1500));
        assert_eq!(is_listening(hwnd, kind), Some(false), "quiet after stop");
    }

    #[test]
    fn codex_is_never_sent_by_the_stop() {
        for name in names(AppKind::ChatGpt).stop.iter().chain(names(AppKind::ChatGpt).cancel) {
            assert!(!name.to_lowercase().contains("send"), "{name}");
        }
    }
}
