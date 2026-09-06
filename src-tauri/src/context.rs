//! Which application is the user dictating into, and what style fits it.
//! Privacy first: process name and window title only. Password fields are
//! detected through UI Automation and refused. Nearby text is read only when
//! Context Awareness is switched on, and never from sensitive applications.

use serde::{Deserialize, Serialize};

use crate::insertion::Target;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppCategory {
    Chat,
    Email,
    Document,
    Code,
    AiChat,
    Browser,
    Terminal,
    Sensitive,
    Unknown,
}

impl AppCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppCategory::Chat => "chat",
            AppCategory::Email => "email",
            AppCategory::Document => "document",
            AppCategory::Code => "code",
            AppCategory::AiChat => "ai_chat",
            AppCategory::Browser => "browser",
            AppCategory::Terminal => "terminal",
            AppCategory::Sensitive => "sensitive",
            AppCategory::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppContext {
    pub target: Target,
    pub category: AppCategory,
    pub friendly_name: String,
    /// Style decisions derived from category (overridable by app_styles rows).
    pub trailing_punctuation: bool,
    pub capitalize_first: bool,
    /// Terminals need Ctrl+Shift+V.
    pub shift_paste: bool,
    pub nearby_text: Option<String>,
}

/// Applications where dictation must never read context and where insertion is
/// refused for password-like fields. Matched on the process name (lowercase).
const SENSITIVE_PROCESSES: &[&str] = &[
    "1password.exe", "bitwarden.exe", "keepass.exe", "keepassxc.exe", "lastpass.exe", "dashlane.exe", "nordpass.exe",
    "enpass.exe", "authy.exe", "consent.exe", "logonui.exe", "credentialuihost.exe", "lockapp.exe",
];

fn classify_process(proc_name: &str, title: &str) -> (AppCategory, &'static str) {
    let p = proc_name.to_ascii_lowercase();
    let t = title.to_ascii_lowercase();
    if SENSITIVE_PROCESSES.iter().any(|s| p == *s) {
        return (AppCategory::Sensitive, "Sensitive application");
    }
    match p.as_str() {
        "telegram.exe" | "telegram desktop.exe" => (AppCategory::Chat, "Telegram"),
        "viber.exe" => (AppCategory::Chat, "Viber"),
        "whatsapp.exe" => (AppCategory::Chat, "WhatsApp"),
        "discord.exe" => (AppCategory::Chat, "Discord"),
        "slack.exe" => (AppCategory::Chat, "Slack"),
        "ms-teams.exe" | "teams.exe" => (AppCategory::Chat, "Teams"),
        "signal.exe" => (AppCategory::Chat, "Signal"),
        "messenger.exe" => (AppCategory::Chat, "Messenger"),
        "outlook.exe" | "olk.exe" => (AppCategory::Email, "Outlook"),
        "thunderbird.exe" => (AppCategory::Email, "Thunderbird"),
        "winword.exe" => (AppCategory::Document, "Word"),
        "excel.exe" => (AppCategory::Document, "Excel"),
        "powerpnt.exe" => (AppCategory::Document, "PowerPoint"),
        "onenote.exe" => (AppCategory::Document, "OneNote"),
        "notepad.exe" => (AppCategory::Document, "Notepad"),
        "notepad++.exe" => (AppCategory::Code, "Notepad++"),
        "obsidian.exe" => (AppCategory::Document, "Obsidian"),
        "notion.exe" => (AppCategory::Document, "Notion"),
        "code.exe" => (AppCategory::Code, "VS Code"),
        "cursor.exe" => (AppCategory::Code, "Cursor"),
        "windsurf.exe" => (AppCategory::Code, "Windsurf"),
        "idea64.exe" | "pycharm64.exe" | "webstorm64.exe" | "rider64.exe" | "clion64.exe" => (AppCategory::Code, "JetBrains IDE"),
        "devenv.exe" => (AppCategory::Code, "Visual Studio"),
        "windowsterminal.exe" | "wt.exe" => (AppCategory::Terminal, "Windows Terminal"),
        "cmd.exe" | "powershell.exe" | "pwsh.exe" | "conhost.exe" | "mintty.exe" | "alacritty.exe" | "wezterm-gui.exe" => (AppCategory::Terminal, "Terminal"),
        "claude.exe" => (AppCategory::AiChat, "Claude"),
        "chatgpt.exe" => (AppCategory::AiChat, "ChatGPT"),
        "chrome.exe" | "msedge.exe" | "firefox.exe" | "brave.exe" | "opera.exe" | "vivaldi.exe" | "arc.exe" => classify_browser(&t),
        _ => (AppCategory::Unknown, "Application"),
    }
}

fn classify_browser(title: &str) -> (AppCategory, &'static str) {
    if title.contains("gmail") || title.contains("outlook") || title.contains("mail") {
        (AppCategory::Email, "Webmail")
    } else if title.contains("chatgpt") || title.contains("claude") || title.contains("gemini") || title.contains("copilot") || title.contains("perplexity") {
        (AppCategory::AiChat, "AI chat")
    } else if title.contains("google docs") || title.contains("google docs") || title.contains("notion") || title.contains("- docs") {
        (AppCategory::Document, "Web document")
    } else if title.contains("telegram") || title.contains("whatsapp") || title.contains("slack") || title.contains("discord") || title.contains("messenger") || title.contains("teams") {
        (AppCategory::Chat, "Web chat")
    } else if title.contains("bank") || title.contains("τράπεζα") || title.contains("paypal") || title.contains("stripe") || title.contains("revolut") || title.contains("login") || title.contains("sign in") || title.contains("σύνδεση") {
        (AppCategory::Sensitive, "Sensitive web page")
    } else {
        (AppCategory::Browser, "Browser")
    }
}

pub fn build_context(target: Target) -> AppContext {
    let (category, friendly) = classify_process(&target.process_name, &target.title);
    let (trailing, cap, shift_paste) = match category {
        AppCategory::Chat => (false, true, false),
        AppCategory::Terminal => (false, false, true),
        AppCategory::Code => (false, false, false),
        AppCategory::AiChat => (true, true, false),
        AppCategory::Email | AppCategory::Document => (true, true, false),
        _ => (true, true, false),
    };
    AppContext { target, category, friendly_name: friendly.to_string(), trailing_punctuation: trailing, capitalize_first: cap, shift_paste, nearby_text: None }
}

/// Overrides from the user's app style table (process_match is a case-insensitive
/// substring of the process name, or the category name).
pub fn apply_style_overrides(ctx: &mut AppContext, styles: &[crate::db::AppStyle]) {
    let p = ctx.target.process_name.to_ascii_lowercase();
    for s in styles.iter().filter(|s| s.enabled) {
        let m = s.process_match.to_ascii_lowercase();
        let matches = (!m.is_empty() && p.contains(&m)) || s.category == ctx.category.as_str();
        if matches {
            ctx.trailing_punctuation = s.trailing_punctuation;
            ctx.capitalize_first = s.capitalize_first;
            if !s.name.is_empty() {
                ctx.friendly_name = s.name.clone();
            }
        }
    }
}

#[cfg(windows)]
pub mod uia {
    //! UI Automation: focused element inspection. Used for password detection
    //! and, when the user enabled Context Awareness, a little text near the caret.
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
    use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation, IUIAutomationTextPattern, IUIAutomationValuePattern, UIA_TextPatternId, UIA_ValuePatternId};

    #[derive(Debug, Clone, Default)]
    pub struct FocusInfo {
        pub is_password: bool,
        pub control_type: i32,
        pub name: String,
        pub value_preview: Option<String>,
    }

    /// Runs on a fresh thread each time so COM apartment state never leaks into
    /// the caller. Cheap enough (a few ms).
    pub fn inspect_focus(read_text: bool) -> FocusInfo {
        let handle = std::thread::spawn(move || unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let Ok(automation): windows::core::Result<IUIAutomation> = CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) else { return FocusInfo::default() };
            let Ok(el) = automation.GetFocusedElement() else { return FocusInfo::default() };
            let is_password = el.CurrentIsPassword().map(|b| b.as_bool()).unwrap_or(false);
            let control_type = el.CurrentControlType().map(|c| c.0).unwrap_or(0);
            let name = el.CurrentName().map(|n| n.to_string()).unwrap_or_default();
            let mut value_preview = None;
            if read_text && !is_password {
                if let Ok(p) = el.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId) {
                    if let Ok(range) = p.DocumentRange() {
                        if let Ok(t) = range.GetText(400) {
                            value_preview = Some(t.to_string());
                        }
                    }
                }
                if value_preview.is_none() {
                    if let Ok(p) = el.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId) {
                        if let Ok(v) = p.CurrentValue() {
                            let s = v.to_string();
                            value_preview = Some(s.chars().rev().take(400).collect::<Vec<_>>().into_iter().rev().collect());
                        }
                    }
                }
            }
            FocusInfo { is_password, control_type, name, value_preview }
        });
        handle.join().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_known_apps() {
        assert_eq!(classify_process("Telegram.exe", "Telegram").0, AppCategory::Chat);
        assert_eq!(classify_process("chrome.exe", "Inbox (3) - Gmail - Google Chrome").0, AppCategory::Email);
        assert_eq!(classify_process("chrome.exe", "ChatGPT - Google Chrome").0, AppCategory::AiChat);
        assert_eq!(classify_process("Code.exe", "main.rs - lalia").0, AppCategory::Code);
        assert_eq!(classify_process("1Password.exe", "").0, AppCategory::Sensitive);
        assert_eq!(classify_process("WindowsTerminal.exe", "").0, AppCategory::Terminal);
    }

    #[test]
    fn chat_style_has_no_trailing_period() {
        let ctx = build_context(Target { process_name: "Telegram.exe".into(), ..Default::default() });
        assert!(!ctx.trailing_punctuation);
        assert!(ctx.capitalize_first);
        let ctx = build_context(Target { process_name: "WINWORD.EXE".into(), ..Default::default() });
        assert!(ctx.trailing_punctuation);
    }
}
