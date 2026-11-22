//! The first lines of a problem report: what is wrong, in one place, before
//! the raw logs.
//!
//! A user who writes "it does nothing" sends us a bundle of three hundred
//! events. On 14 November 2026 one such report held the answer twice over and
//! still read as noise: the engine had died at start ("process exited", with
//! the Windows exit code kept only in the engine status, never written out)
//! and not one dictation had started in the whole run. This module turns the
//! facts the app already has into a short verdict, so the first screen of
//! every report says which part failed and what the code means.

/// What a Windows exit code of the engine means, in words a person can act on.
/// The engine is a separate process, so its death shows only as this number.
/// Rust prints such codes as "exit code: 0xc0000135"; the decimal form is
/// matched too, in case a status was formatted another way.
pub fn explain_exit(text: &str) -> Option<&'static str> {
    let t = text.to_ascii_lowercase();
    const CODES: &[(&str, &str, &str)] = &[
        ("0xc0000135", "3221225781", "a file the engine needs (a DLL) is missing on this PC"),
        ("0xc000001d", "3221225501", "the processor lacks an instruction the engine uses (processor too old for this build)"),
        ("0xc0000005", "3221225477", "the engine crashed (memory access violation)"),
        ("0xc000007b", "3221225595", "a DLL of the wrong kind (32-bit and 64-bit mixed) was loaded"),
        ("0xc0000142", "3221225794", "a DLL failed to start, often a missing or broken Visual C++ runtime"),
        ("0xc0000409", "3221226505", "the engine stopped itself on a fatal error"),
        ("0xc0000017", "3221225495", "not enough memory to start the engine"),
        ("0xc00000fd", "3221225725", "the engine ran out of stack"),
    ];
    for (hex, dec, meaning) in CODES {
        if t.contains(hex) || t.contains(dec) {
            return Some(meaning);
        }
    }
    None
}

/// The processor features the engine builds care about. A missing AVX on an
/// old laptop is the first suspect when the engine dies with no message.
pub fn cpu_flags() -> String {
    #[cfg(target_arch = "x86_64")]
    {
        let mut have = Vec::new();
        let mut lack = Vec::new();
        macro_rules! check {
            ($($f:tt),*) => {$(
                if std::arch::is_x86_feature_detected!($f) { have.push($f) } else { lack.push($f) }
            )*};
        }
        check!("sse4.2", "avx", "f16c", "fma", "avx2", "avx512f");
        format!("has {} | lacks {}", join_or_none(&have), join_or_none(&lack))
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        std::env::consts::ARCH.to_string()
    }
}

#[cfg(target_arch = "x86_64")]
fn join_or_none(v: &[&str]) -> String {
    if v.is_empty() { "none".into() } else { v.join(" ") }
}

/// The Windows build number, from the registry (the version API answers
/// with an old number to programs whose manifest does not name Windows 10).
pub fn windows_build() -> String {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let out = std::process::Command::new("reg")
            .args(["query", r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion", "/v", "CurrentBuild"])
            .creation_flags(0x0800_0000)
            .output();
        match out {
            Ok(o) => match String::from_utf8_lossy(&o.stdout).split_whitespace().last() {
                Some(b) if b.chars().all(|c| c.is_ascii_digit()) => format!("build {b}"),
                _ => "unknown".into(),
            },
            Err(_) => "unknown".into(),
        }
    }
    #[cfg(not(windows))]
    {
        std::env::consts::OS.into()
    }
}

/// The engine as the verdict needs it: a status word and its message.
pub struct EngineFacts<'a> {
    /// "Ready", "Starting", "Failed", "Missing" (the Debug form of the status).
    pub status: &'a str,
    pub message: Option<&'a str>,
}

/// One line per finding, most important first. `events` are journal lines
/// (JSON, newest first) as the bundle prints them.
pub fn verdict(engine: &EngineFacts<'_>, events: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let msg = engine.message.unwrap_or("");
    match engine.status {
        "Ready" => out.push("ENGINE: ready".to_string()),
        "Starting" => out.push("ENGINE: still starting when the report was written".to_string()),
        // stop() was called by the app itself, not a crash
        "Stopped" => out.push("ENGINE: stopped by the app (not a crash)".to_string()),
        "Missing" => out.push(format!("ENGINE: not running: {}", if msg.is_empty() { "no model or engine file" } else { msg })),
        _ => match explain_exit(msg) {
            Some(meaning) => out.push(format!("ENGINE: FAILED TO START: {meaning} ({msg})")),
            None if msg.is_empty() => out.push("ENGINE: FAILED, no message".to_string()),
            None => out.push(format!("ENGINE: FAILED: {msg}")),
        },
    }

    let count = |kind: &str| events.iter().filter(|l| l.contains(&format!("\"kind\":\"{kind}\""))).count();
    let starts = count("record.start");
    let not_ready = count("record.engine_not_ready");
    let no_speech = count("record.no_speech");
    let silent = count("record.silent_mic");
    if starts == 0 {
        out.push("DICTATION: no recording started in these events (the hotkey was never caught, or never pressed)".to_string());
    } else {
        out.push(format!("DICTATION: {starts} recording(s) started"));
        if not_ready > 0 {
            out.push(format!("DICTATION: {not_ready} waited for an engine that was not ready"));
        }
        if silent > 0 {
            out.push(format!("MICROPHONE: {silent} recording(s) heard silence"));
        }
        if no_speech > 0 {
            out.push(format!("DICTATION: {no_speech} recording(s) had no speech in them"));
        }
    }
    if count("hotkey.no_permission") > 0 {
        out.push("HOTKEY: the system refused the keyboard hook".to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(kind: &str) -> String {
        format!("{{\"at\":\"2026-01-01T10:00:00+02:00\",\"d\":{{}},\"kind\":\"{kind}\",\"level\":\"info\"}}")
    }

    #[test]
    fn exit_codes_get_words() {
        assert!(explain_exit("whisper-server exited early (exit code: 0xc000001d)").unwrap().contains("processor"));
        assert!(explain_exit("exit code: 0xC0000135").unwrap().contains("DLL"));
        assert!(explain_exit("exit status 3221225477").unwrap().contains("crashed"));
        assert_eq!(explain_exit("exit code: 1"), None);
        assert_eq!(explain_exit(""), None);
    }

    #[test]
    fn a_dead_engine_and_no_dictation_both_show() {
        let engine = EngineFacts { status: "Failed", message: Some("whisper-server exited early (exit code: 0xc000001d)") };
        let events = vec![ev("update.checked"), ev("app.start"), ev("packs.applied")];
        let v = verdict(&engine, &events);
        assert!(v[0].starts_with("ENGINE: FAILED TO START: the processor"), "{v:?}");
        assert!(v[0].contains("0xc000001d"));
        assert!(v[1].contains("no recording started"), "{v:?}");
    }

    #[test]
    fn a_working_run_says_so() {
        let engine = EngineFacts { status: "Ready", message: None };
        let events = vec![ev("record.start"), ev("record.no_speech"), ev("record.start")];
        let v = verdict(&engine, &events);
        assert_eq!(v[0], "ENGINE: ready");
        assert_eq!(v[1], "DICTATION: 2 recording(s) started");
        assert!(v.iter().any(|l| l.contains("1 recording(s) had no speech")));
    }

    #[test]
    fn an_unknown_failure_keeps_its_message() {
        let engine = EngineFacts { status: "Failed", message: Some("whisper-server did not become ready in 120 s") };
        let v = verdict(&engine, &[ev("record.start"), ev("record.engine_not_ready")]);
        assert_eq!(v[0], "ENGINE: FAILED: whisper-server did not become ready in 120 s");
        assert!(v.iter().any(|l| l.contains("1 waited for an engine")));
    }

    /// The engine's real exit status, as Rust prints it, must be what
    /// `explain_exit` looks for: a process that ends with 0xc000001d.
    #[cfg(windows)]
    #[test]
    fn a_real_windows_exit_status_is_recognised() {
        let status = std::process::Command::new("cmd").args(["/c", "exit", "-1073741795"]).status().unwrap();
        let text = format!("whisper-server exited early ({status})");
        assert!(explain_exit(&text).unwrap().contains("processor"), "{text}");
    }

    #[test]
    fn cpu_flags_names_avx() {
        let f = cpu_flags();
        #[cfg(target_arch = "x86_64")]
        assert!(f.contains("avx"), "{f}");
        assert!(!f.is_empty());
    }
}
