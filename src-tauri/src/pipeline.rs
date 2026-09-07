//! The dictation state machine. One async task owns it; everything else sends
//! messages. Every step has a timeout or a cancel path so the app never sits in
//! "recording" or "processing" forever.
//!
//! IDLE -> (hotkey down) -> RECORDING -> (hotkey up) -> PROCESSING (VAD, ASR)
//!      -> CLEANING (rules, dictionary, snippets) -> INSERT -> SUCCESS -> IDLE

use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio::sync::mpsc;

use crate::asr::TranscriptionRequest;
use crate::audio::AudioCapture;
use crate::cleanup::dictionary::{build_hint_prompt, DictionaryEngine};
use crate::cleanup::snippets::SnippetEngine;
use crate::cleanup::{run_deterministic, CleanupOptions};
use crate::context::{AppCategory, AppContext};
use crate::db::{Db, HistoryEntry};
use crate::engine::EngineManager;
use crate::hotkey::{ChordId, HotkeyEvent, CAPTURE_ESCAPE};
use crate::insertion::{self, InsertOptions, InsertOutcome};
use crate::overlay::{self, OverlayPayload, OverlayState};
use crate::settings::{InsertionMethod, LanguageMode, Settings};

#[derive(Debug)]
pub enum PipelineMsg {
    Hotkey(HotkeyEvent),
    /// Start or stop from the UI (dashboard test button, tray).
    Toggle,
    Cancel,
    PasteLast,
    /// Re-run insertion of a given history entry (or the last one).
    PasteHistory(String),
    /// Re-transcribe the last recovery audio after a failure.
    Retry,
    SettingsChanged,
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Ptt,
    HandsFree,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Idle,
    Recording,
    Processing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSnapshot {
    pub phase: Phase,
    pub hands_free: bool,
    pub last_error: Option<String>,
    pub last_transcript: Option<String>,
    pub mic_open: bool,
}

pub struct Shared {
    pub settings: Arc<RwLock<Settings>>,
    pub db: Arc<Db>,
    pub audio: AudioCapture,
    pub engine: Arc<EngineManager>,
    pub dict: Arc<RwLock<DictionaryEngine>>,
    pub snippets: Arc<RwLock<SnippetEngine>>,
    pub snapshot: Arc<Mutex<PipelineSnapshot>>,
    pub tx: mpsc::UnboundedSender<PipelineMsg>,
}

struct Session {
    mode: Mode,
    started: Instant,
    ctx: AppContext,
    cancel: bool,
    segments: Arc<Mutex<Segmenter>>,
}

/// Transcription of finished phrases while the user is still speaking. A pause
/// of SEGMENT_PAUSE_MS closes a segment once it holds at least SEGMENT_MIN_MS of
/// audio, and the segment goes to the engine right away. On key release only the
/// last phrase is left, so the wait no longer grows with the length of the
/// dictation. If any segment fails, the whole recording is transcribed in one
/// pass as before: this is an accelerator, never a different result path.
const SEGMENT_PAUSE_MS: usize = 700;
const SEGMENT_MIN_MS: usize = 2500;
/// Without any pause this long, cut anyway at the quietest spot near the end,
/// so the wait after the stop key never grows with a long breathless stretch.
const SEGMENT_MAX_MS: usize = 12_000;

type SegmentJob = tokio::task::JoinHandle<(Result<crate::asr::TranscriptionResult, crate::asr::AsrError>, Option<(f32, f32)>)>;

/// Joins phrases transcribed separately. A phrase that starts with a comma or
/// a semicolon after one that ended in a period ("κάνει." + ", σε pixel") loses
/// that leading mark; the engine only wrote it because it saw the phrase alone.
fn join_phrases(parts: &[String]) -> String {
    let mut out = String::new();
    for p in parts {
        let mut p = p.trim();
        if !out.is_empty() && out.ends_with(['.', ';', '?', '!']) {
            p = p.trim_start_matches([',', ';', ':']).trim_start();
        }
        if p.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(p);
    }
    out
}

/// Adds the question mark a rising voice asked for, when the words alone gave
/// the sentence a period. Marks the engine already wrote are kept.
fn apply_intonation(text: &str, pitch: Option<(f32, f32)>, enabled: bool) -> String {
    let t = text.trim();
    let Some((peak, end)) = pitch else { return t.to_string() };
    if !enabled || t.is_empty() || t.ends_with(';') || t.ends_with('?') || !crate::audio::sounds_like_question(peak, end) {
        return t.to_string();
    }
    // A list item ("Λος Άντζελες, Νέα Υόρκη") rises like a question. A phrase
    // whose last comma-separated part is one or two words is treated as a list.
    let last_chunk = t.rsplit(',').next().unwrap_or(t);
    if t.contains(',') && last_chunk.split_whitespace().count() <= 2 {
        return t.to_string();
    }
    let body = t.trim_end_matches(|c: char| matches!(c, '.' | '!' | '…'));
    let mark = if crate::cleanup::deterministic::looks_greek(body) { ';' } else { '?' };
    tracing::info!("intonation: question mark added (peak {peak:+.1} st, end {end:+.1} st)");
    format!("{body}{mark}")
}

#[derive(Default)]
struct Segmenter {
    /// Sample index where the next segment starts.
    cut: usize,
    jobs: Vec<SegmentJob>,
    /// Text of the last finished segment, given to the next request as context.
    last_text: String,
}

fn join_prompt(hints: Option<String>, previous: &str) -> Option<String> {
    // Whisper reads the prompt as preceding context; keep the tail of the
    // previous phrase so wording and casing stay consistent across segments.
    let prev: String = previous.chars().rev().take(200).collect::<Vec<_>>().into_iter().rev().collect();
    match (hints, prev.trim()) {
        (Some(h), p) if !p.is_empty() => Some(format!("{h} {p}")),
        (Some(h), _) => Some(h),
        (None, p) if !p.is_empty() => Some(p.to_string()),
        _ => None,
    }
}

/// Sends one finished segment to the engine in the background. Silence-only
/// segments are dropped without a request.
fn dispatch_segment(shared: &Arc<Shared>, seg: &Arc<Mutex<Segmenter>>, samples: Vec<f32>, lang: &str, hints: Option<String>, beam: u32, vad: bool, min_speech_ms: u64) {
    let Some(a) = crate::audio::analyze_speech(&samples, min_speech_ms) else { return };
    if a.trimmed.is_empty() {
        return;
    }
    let wav = crate::audio::encode_wav(&a.trimmed);
    let pitch = crate::audio::tail_pitch_features(&a.trimmed);
    let (n, prompt) = {
        let g = seg.lock();
        (g.jobs.len(), join_prompt(hints, &g.last_text))
    };
    let req = TranscriptionRequest { wav, language: lang.to_string(), prompt, beam_size: beam, vad };
    let engine = shared.engine.clone();
    let seg2 = seg.clone();
    let audio_ms = a.trimmed.len() / 16;
    let handle = tokio::spawn(async move {
        let r = engine.transcribe(req).await;
        match &r {
            Ok(t) => {
                tracing::debug!("segment {n}: {audio_ms} ms audio, {} ms inference", t.inference_ms);
                if let Some((peak, end)) = pitch {
                    tracing::info!("prosody: segment {n} tail peak {peak:+.1} st, end {end:+.1} st | {}", crate::logging::redact(t.text.trim()));
                }
                seg2.lock().last_text = t.text.trim().to_string();
            }
            Err(e) => tracing::warn!("segment {n} failed, the recording will be transcribed in one pass: {e}"),
        }
        (r, pitch)
    });
    seg.lock().jobs.push(handle);
}

/// Whisper hallucinations on silence, seen in the wild. If the whole transcript
/// is one of these, treat it as no speech.
const HALLUCINATIONS: &[&str] = &[
    "υπότιτλοι authorwave",
    "υπότιτλοι authorwave.",
    "subtitles by the amara.org community",
];

pub fn spawn(app: tauri::AppHandle, shared: Arc<Shared>, mut rx: mpsc::UnboundedReceiver<PipelineMsg>) {
    tauri::async_runtime::spawn(async move {
        let mut session: Option<Session> = None;
        let mut level_task: Option<tokio::task::JoinHandle<()>> = None;
        let mut idle_timer: Option<tokio::task::JoinHandle<()>> = None;
        let last_recovery: Arc<Mutex<Option<(Vec<f32>, AppContext)>>> = Arc::new(Mutex::new(None));

        while let Some(msg) = rx.recv().await {
            match msg {
                PipelineMsg::Shutdown => break,
                PipelineMsg::SettingsChanged => {
                    let s = shared.settings.read().clone();
                    shared.audio.configure(s.audio.preroll_ms, s.audio.max_recording_seconds);
                    if s.audio.keep_stream_warm && session.is_none() {
                        let audio = shared.audio.clone();
                        let dev = s.audio.device_name.clone();
                        let _ = tokio::task::spawn_blocking(move || audio.open(dev)).await;
                    } else if !s.audio.keep_stream_warm && session.is_none() {
                        shared.audio.close();
                    }
                    shared.snapshot.lock().mic_open = shared.audio.is_open();
                }
                PipelineMsg::Hotkey(ev) => match ev {
                    HotkeyEvent::Pressed(ChordId::PushToTalk) => {
                        if let Some(s) = session.as_ref() {
                            if s.mode == Mode::HandsFree {
                                // pressing the chord again ends hands-free
                                finalize(&app, &shared, session.take().unwrap(), &mut level_task, &last_recovery, &mut idle_timer).await;
                            }
                            continue;
                        }
                        if let Some(s) = begin(&app, &shared, Mode::Ptt, &mut level_task, &mut idle_timer).await {
                            session = Some(s);
                        }
                    }
                    HotkeyEvent::Released(ChordId::PushToTalk) => {
                        let Some(s) = session.as_ref() else { continue };
                        if s.mode != Mode::Ptt {
                            continue;
                        }
                        let settings = shared.settings.read().clone();
                        if settings.hotkeys.tap_toggles_hands_free && s.started.elapsed() < Duration::from_millis(settings.hotkeys.tap_ms) {
                            if let Some(s) = session.as_mut() {
                                s.mode = Mode::HandsFree;
                            }
                            shared.snapshot.lock().hands_free = true;
                            overlay::emit_state(&app, OverlayPayload { state: OverlayState::HandsFree, message: None, preview: None, can_retry: false, seconds: 0.0 });
                            continue;
                        }
                        finalize(&app, &shared, session.take().unwrap(), &mut level_task, &last_recovery, &mut idle_timer).await;
                    }
                    HotkeyEvent::Pressed(ChordId::HandsFree) => {
                        match session.as_mut() {
                            None => {
                                if let Some(s) = begin(&app, &shared, Mode::HandsFree, &mut level_task, &mut idle_timer).await {
                                    session = Some(s);
                                }
                            }
                            Some(s) if s.mode == Mode::Ptt && s.started.elapsed() < Duration::from_millis(1500) => {
                                // Ctrl+Win already recording, Space added: switch to hands-free
                                s.mode = Mode::HandsFree;
                                shared.snapshot.lock().hands_free = true;
                                overlay::emit_state(&app, OverlayPayload { state: OverlayState::HandsFree, message: None, preview: None, can_retry: false, seconds: 0.0 });
                            }
                            Some(_) => {
                                finalize(&app, &shared, session.take().unwrap(), &mut level_task, &last_recovery, &mut idle_timer).await;
                            }
                        }
                    }
                    HotkeyEvent::Released(ChordId::HandsFree) => {}
                    HotkeyEvent::Pressed(ChordId::PasteLast) => {
                        paste_last(&app, &shared, None).await;
                    }
                    HotkeyEvent::Released(ChordId::PasteLast) => {}
                    HotkeyEvent::Escape => {
                        if let Some(s) = session.take() {
                            cancel(&app, &shared, s, &mut level_task, &mut idle_timer).await;
                        }
                    }
                },
                PipelineMsg::Toggle => match session.take() {
                    Some(s) => finalize(&app, &shared, s, &mut level_task, &last_recovery, &mut idle_timer).await,
                    None => {
                        if let Some(s) = begin(&app, &shared, Mode::HandsFree, &mut level_task, &mut idle_timer).await {
                            session = Some(s);
                        }
                    }
                },
                PipelineMsg::Cancel => {
                    if let Some(s) = session.take() {
                        cancel(&app, &shared, s, &mut level_task, &mut idle_timer).await;
                    }
                }
                PipelineMsg::PasteLast => paste_last(&app, &shared, None).await,
                PipelineMsg::PasteHistory(id) => paste_last(&app, &shared, Some(id)).await,
                PipelineMsg::Retry => {
                    let rec = last_recovery.lock().clone();
                    if let Some((samples, ctx)) = rec {
                        process(&app, &shared, samples, ctx, &last_recovery, &mut idle_timer, true, None).await;
                    }
                }
            }
        }
    });
}

fn set_phase(shared: &Shared, phase: Phase, hands_free: bool) {
    let mut s = shared.snapshot.lock();
    s.phase = phase;
    s.hands_free = hands_free;
    s.mic_open = shared.audio.is_open();
}

async fn begin(app: &tauri::AppHandle, shared: &Arc<Shared>, mode: Mode, level_task: &mut Option<tokio::task::JoinHandle<()>>, idle_timer: &mut Option<tokio::task::JoinHandle<()>>) -> Option<Session> {
    let t0 = Instant::now();
    if let Some(t) = idle_timer.take() {
        t.abort();
    }
    let settings = shared.settings.read().clone();

    // 1. Who has focus? Refuse sensitive targets.
    let target = tokio::task::spawn_blocking(insertion::capture_target).await.unwrap_or_default();
    let mut ctx = crate::context::build_context(target);
    if let Ok(styles) = shared.db.list_app_styles() {
        crate::context::apply_style_overrides(&mut ctx, &styles);
    }
    tracing::debug!("begin: target captured in {} ms ({}, hwnd {})", t0.elapsed().as_millis(), ctx.target.process_name, ctx.target.hwnd);
    overlay::show(app, &settings.overlay, ctx.target.hwnd);
    tracing::debug!("begin: overlay shown at {} ms", t0.elapsed().as_millis());
    overlay::emit_state(app, OverlayPayload { state: OverlayState::Starting, message: None, preview: None, can_retry: false, seconds: 0.0 });

    if ctx.category == AppCategory::Sensitive {
        overlay::emit_state(app, OverlayPayload { state: OverlayState::Sensitive, message: Some(ctx.friendly_name.clone()), preview: None, can_retry: false, seconds: 0.0 });
        schedule_idle(app, shared, idle_timer, 1800);
        return None;
    }

    // 2. Microphone.
    if !shared.audio.is_open() || !shared.audio.is_alive() {
        let audio = shared.audio.clone();
        let dev = settings.audio.device_name.clone();
        match tokio::task::spawn_blocking(move || audio.open(dev)).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                tracing::error!("microphone: {e}");
                overlay::emit_state(app, OverlayPayload { state: OverlayState::MicUnavailable, message: Some(e), preview: None, can_retry: false, seconds: 0.0 });
                schedule_idle(app, shared, idle_timer, 2500);
                return None;
            }
            Err(_) => return None,
        }
    }
    shared.audio.start_recording();
    CAPTURE_ESCAPE.store(true, std::sync::atomic::Ordering::Relaxed);

    // 3. Engine warm? Recording proceeds anyway; the wait happens at release.
    let state = if mode == Mode::HandsFree { OverlayState::HandsFree } else { OverlayState::Recording };
    overlay::emit_state(app, OverlayPayload { state, message: None, preview: None, can_retry: false, seconds: 0.0 });
    set_phase(shared, Phase::Recording, mode == Mode::HandsFree);
    tracing::info!("recording started in {} ms (target {}, {})", t0.elapsed().as_millis(), ctx.friendly_name, ctx.target.process_name);
    crate::journal::info(
        "record.start",
        serde_json::json!({ "app": ctx.target.process_name, "friendly": ctx.friendly_name, "ms": t0.elapsed().as_millis() as u64 }),
    );

    // 4. Password field check (UIA) after we already reacted. Chromium and
    // Electron apps can take seconds to answer UI Automation, so the wait is
    // capped: a slow answer must never freeze the pipeline while the user talks.
    #[cfg(windows)]
    {
        let read_ctx = settings.privacy.context_awareness && ctx.category != AppCategory::Sensitive;
        let info = match tokio::time::timeout(Duration::from_millis(400), tokio::task::spawn_blocking(move || crate::context::uia::inspect_focus(read_ctx))).await {
            Ok(Ok(info)) => info,
            Ok(Err(_)) => Default::default(),
            Err(_) => {
                tracing::warn!("UI Automation did not answer within 400 ms for {}; skipping the password-field check", ctx.target.process_name);
                Default::default()
            }
        };
        if info.is_password {
            shared.audio.discard();
            CAPTURE_ESCAPE.store(false, std::sync::atomic::Ordering::Relaxed);
            overlay::emit_state(app, OverlayPayload { state: OverlayState::Sensitive, message: Some("password field".into()), preview: None, can_retry: false, seconds: 0.0 });
            set_phase(shared, Phase::Idle, false);
            schedule_idle(app, shared, idle_timer, 1800);
            return None;
        }
        if read_ctx {
            ctx.nearby_text = info.value_preview;
        }
    }

    // 5. Level meter + safety cap + segment dispatch on pauses.
    let app2 = app.clone();
    let shared2 = shared.clone();
    let max_secs = settings.audio.max_recording_seconds as f32;
    let segments: Arc<Mutex<Segmenter>> = Arc::new(Mutex::new(Segmenter::default()));
    let seg = segments.clone();
    let segment_enabled = settings.asr.segment_while_speaking;
    let lang_code = settings.language.mode.whisper_code().to_string();
    let hints = if settings.asr.hints_from_dictionary {
        let terms = shared.dict.read().hint_terms(settings.asr.max_hint_terms);
        build_hint_prompt(&terms, &lang_code)
    } else {
        None
    };
    let (beam, vad, min_speech_ms) = (settings.asr.beam_size, settings.asr.vad, settings.audio.min_speech_ms);
    *level_task = Some(tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_millis(33));
        let mut n: u32 = 0;
        loop {
            tick.tick().await;
            n = n.wrapping_add(1);
            let secs = shared2.audio.recorded_seconds();
            overlay::emit_level(&app2, shared2.audio.level(), secs);
            if secs >= max_secs {
                let _ = shared2.tx.send(PipelineMsg::Toggle);
                break;
            }
            // Every ~250 ms: has the user paused after saying enough for a segment?
            if segment_enabled && n % 8 == 0 {
                let len = shared2.audio.recorded_len();
                let cut = seg.lock().cut;
                let pause = SEGMENT_PAUSE_MS * 16;
                if len >= cut + SEGMENT_MIN_MS * 16 + pause {
                    let tail = shared2.audio.recorded_range(len - pause, len);
                    if crate::audio::is_silent(&tail) {
                        // cut in the middle of the pause: the segment keeps some
                        // trailing silence, the next one some leading silence
                        let end = len - pause / 2;
                        let samples = shared2.audio.recorded_range(cut, end);
                        seg.lock().cut = end;
                        dispatch_segment(&shared2, &seg, samples, &lang_code, hints.clone(), beam, vad, min_speech_ms);
                    } else if len >= cut + SEGMENT_MAX_MS * 16 {
                        // no pause for a long time: cut at the quietest 200 ms of
                        // the last three seconds, which lands between words
                        let window = shared2.audio.recorded_range(len - 16 * 3000, len);
                        let at = crate::audio::quietest_point(&window, 16 * 200);
                        let end = len - 16 * 3000 + at;
                        let samples = shared2.audio.recorded_range(cut, end);
                        seg.lock().cut = end;
                        dispatch_segment(&shared2, &seg, samples, &lang_code, hints.clone(), beam, vad, min_speech_ms);
                    }
                }
            }
        }
    }));

    Some(Session { mode, started: t0, ctx, cancel: false, segments })
}

async fn cancel(app: &tauri::AppHandle, shared: &Arc<Shared>, s: Session, level_task: &mut Option<tokio::task::JoinHandle<()>>, idle_timer: &mut Option<tokio::task::JoinHandle<()>>) {
    if let Some(t) = level_task.take() {
        t.abort();
    }
    for j in s.segments.lock().jobs.drain(..) {
        j.abort();
    }
    shared.audio.discard();
    CAPTURE_ESCAPE.store(false, std::sync::atomic::Ordering::Relaxed);
    crate::hotkey::reset_pressed_state();
    overlay::emit_state(app, OverlayPayload { state: OverlayState::Cancelled, message: None, preview: None, can_retry: false, seconds: 0.0 });
    set_phase(shared, Phase::Idle, false);
    if !shared.settings.read().audio.keep_stream_warm {
        shared.audio.close();
    }
    schedule_idle(app, shared, idle_timer, 900);
}

async fn finalize(
    app: &tauri::AppHandle,
    shared: &Arc<Shared>,
    s: Session,
    level_task: &mut Option<tokio::task::JoinHandle<()>>,
    last_recovery: &Arc<Mutex<Option<(Vec<f32>, AppContext)>>>,
    idle_timer: &mut Option<tokio::task::JoinHandle<()>>,
) {
    if let Some(t) = level_task.take() {
        t.abort();
    }
    let rec = shared.audio.stop_recording();
    if !shared.settings.read().audio.keep_stream_warm {
        shared.audio.close();
    }
    crate::hotkey::reset_pressed_state();
    if s.cancel {
        return;
    }
    if rec.overflowed {
        tracing::warn!("recording hit the maximum length; transcribing what fits");
    }
    // `mut` only matters to the debug-only fake microphone below.
    #[allow(unused_mut)]
    let mut samples = rec.samples;
    // Test harness (debug builds only): replace the microphone with a WAV file
    // named in %LOCALAPPDATA%\Lalia\fake_mic.txt so the whole hotkey -> ASR ->
    // cleanup -> insertion path can be exercised without a human speaking.
    #[cfg(debug_assertions)]
    if let Ok(p) = std::fs::read_to_string(crate::paths::local_dir().join("fake_mic.txt")) {
        let p = p.trim();
        if !p.is_empty() {
            match crate::audio::decode_wav_file(std::path::Path::new(p)) {
                Ok(fake) => {
                    tracing::warn!("FAKE MIC: using {} ({} samples) instead of the microphone", p, fake.len());
                    samples = fake;
                }
                Err(e) => tracing::warn!("fake mic file unreadable: {e}"),
            }
        }
    }
    let segments = s.segments.clone();
    process(app, shared, samples, s.ctx, last_recovery, idle_timer, false, Some(segments)).await;
}

fn schedule_idle(app: &tauri::AppHandle, shared: &Arc<Shared>, idle_timer: &mut Option<tokio::task::JoinHandle<()>>, ms: u64) {
    if let Some(t) = idle_timer.take() {
        t.abort();
    }
    let app = app.clone();
    let hide = shared.settings.read().overlay.hide_when_idle;
    *idle_timer = Some(tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(ms)).await;
        overlay::emit_state(&app, OverlayPayload { state: OverlayState::Idle, message: None, preview: None, can_retry: false, seconds: 0.0 });
        if hide {
            overlay::hide(&app);
        }
    }));
}

fn is_hallucination(text: &str) -> bool {
    let t = text.trim().to_lowercase();
    HALLUCINATIONS.iter().any(|h| t == *h)
}

async fn process(
    app: &tauri::AppHandle,
    shared: &Arc<Shared>,
    samples: Vec<f32>,
    ctx: AppContext,
    last_recovery: &Arc<Mutex<Option<(Vec<f32>, AppContext)>>>,
    idle_timer: &mut Option<tokio::task::JoinHandle<()>>,
    is_retry: bool,
    segments: Option<Arc<Mutex<Segmenter>>>,
) {
    let released_at = Instant::now();
    let settings = shared.settings.read().clone();
    set_phase(shared, Phase::Processing, false);
    overlay::emit_state(app, OverlayPayload { state: OverlayState::Processing, message: None, preview: None, can_retry: false, seconds: samples.len() as f32 / 16000.0 });

    // 1. Empty / accidental press?
    let analysis = crate::audio::analyze_speech(&samples, settings.audio.min_speech_ms);
    let (audio_ms, speech) = match analysis {
        Some(a) if !a.trimmed.is_empty() => (a.total_ms, a.trimmed),
        Some(a) => {
            tracing::info!("no speech: {} ms audio, {} ms speech, peak {:.3}", a.total_ms, a.speech_ms, a.peak);
            crate::journal::info("record.no_speech", serde_json::json!({ "audio_ms": a.total_ms, "speech_ms": a.speech_ms, "peak": a.peak }));
            overlay::emit_state(app, OverlayPayload { state: OverlayState::NoSpeech, message: None, preview: None, can_retry: false, seconds: 0.0 });
            finish_idle(shared, app, idle_timer, 1200);
            return;
        }
        None => {
            overlay::emit_state(app, OverlayPayload { state: OverlayState::NoSpeech, message: None, preview: None, can_retry: false, seconds: 0.0 });
            finish_idle(shared, app, idle_timer, 1200);
            return;
        }
    };

    // 2. Keep a recovery copy until the text is safely inserted.
    *last_recovery.lock() = Some((speech.clone(), ctx.clone()));
    let recovery_file = crate::paths::recovery_dir().join("last.wav");
    let wav = crate::audio::encode_wav(&speech);
    let _ = std::fs::write(&recovery_file, &wav);

    // 3. Engine ready?
    if !shared.engine.is_ready() {
        let info = shared.engine.info();
        let (state, msg) = match info.status {
            crate::asr::EngineStatus::Starting => (OverlayState::ModelUnavailable, Some("the speech model is still loading".to_string())),
            crate::asr::EngineStatus::Missing => (OverlayState::ModelUnavailable, info.message.clone()),
            _ => (OverlayState::Failed, info.message.clone()),
        };
        // wait a little for a starting engine (up to 8 s)
        if info.status == crate::asr::EngineStatus::Starting {
            for _ in 0..40 {
                tokio::time::sleep(Duration::from_millis(200)).await;
                if shared.engine.is_ready() {
                    break;
                }
            }
        }
        if !shared.engine.is_ready() {
            overlay::emit_state(app, OverlayPayload { state, message: msg, preview: None, can_retry: true, seconds: 0.0 });
            shared.snapshot.lock().last_error = Some("engine not ready".into());
            finish_idle(shared, app, idle_timer, 3000);
            return;
        }
    }

    // 4. Transcribe with dictionary hints.
    let lang_code = settings.language.mode.whisper_code().to_string();
    let prompt = if settings.asr.hints_from_dictionary {
        let terms = shared.dict.read().hint_terms(settings.asr.max_hint_terms);
        build_hint_prompt(&terms, &lang_code)
    } else {
        None
    };
    // Segments finished while the user was speaking: collect them in order and
    // transcribe only what came after the last cut. Any failure falls back to
    // one pass over the whole recording.
    let mut head_parts: Vec<String> = Vec::new();
    let mut tail_wav: Option<Vec<u8>> = None;
    let mut tail_prompt: Option<String> = None;
    let mut tail_silent = false;
    if let Some(seg) = segments.as_ref() {
        let (jobs, cut, last_text) = {
            let mut g = seg.lock();
            (std::mem::take(&mut g.jobs), g.cut, g.last_text.clone())
        };
        if !jobs.is_empty() {
            let mut all_ok = true;
            let mut parts_ms = 0u64;
            for j in jobs {
                match j.await {
                    Ok((Ok(t), pitch)) => {
                        parts_ms += t.inference_ms;
                        let text = apply_intonation(t.text.trim(), pitch, settings.cleanup.intonation_questions);
                        if !text.is_empty() && !is_hallucination(&text) {
                            head_parts.push(text);
                        }
                    }
                    _ => {
                        all_ok = false;
                        break;
                    }
                }
            }
            if all_ok {
                let tail = &samples[cut.min(samples.len())..];
                match crate::audio::analyze_speech(tail, settings.audio.min_speech_ms) {
                    Some(a) if !a.trimmed.is_empty() => tail_wav = Some(crate::audio::encode_wav(&a.trimmed)),
                    _ => tail_silent = true,
                }
                tail_prompt = join_prompt(prompt.clone(), &last_text);
                tracing::info!("segments: {} finished while speaking ({parts_ms} ms inference), tail {} ms", head_parts.len(), tail.len() / 16);
            } else {
                head_parts.clear();
                tracing::warn!("a segment failed; transcribing the whole recording in one pass");
            }
        }
    }
    let use_segments = !head_parts.is_empty() || tail_silent;
    let req = if use_segments {
        if tail_silent {
            None
        } else {
            Some(TranscriptionRequest { wav: tail_wav.take().unwrap_or_default(), language: lang_code.clone(), prompt: tail_prompt, beam_size: settings.asr.beam_size, vad: settings.asr.vad })
        }
    } else {
        Some(TranscriptionRequest { wav, language: lang_code.clone(), prompt, beam_size: settings.asr.beam_size, vad: settings.asr.vad })
    };
    // Same rule as the engine client: long recordings need proportionally more
    // time (about 0.09 s per second of audio on the RTX 3070, allow 0.5 s).
    let transcribe_limit = Duration::from_secs((20 + audio_ms / 2000).max(60));
    let result = match req {
        Some(req) => tokio::time::timeout(transcribe_limit, shared.engine.transcribe(req)).await,
        // everything was transcribed while speaking; nothing left after the last pause
        None => Ok(Ok(crate::asr::TranscriptionResult { text: String::new(), detected_language: None, language_probability: None, no_speech_prob: None, engine: String::new(), model: String::new(), inference_ms: 0 })),
    };
    let result = match result {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            tracing::error!("transcription failed: {e}");
            let state = match e {
                crate::asr::AsrError::Offline(_) => OverlayState::Offline,
                _ => OverlayState::Failed,
            };
            overlay::emit_state(app, OverlayPayload { state, message: Some(e.to_string()), preview: None, can_retry: true, seconds: 0.0 });
            shared.snapshot.lock().last_error = Some(e.to_string());
            finish_idle(shared, app, idle_timer, 3500);
            return;
        }
        Err(_) => {
            tracing::error!("transcription timed out; restarting the engine");
            overlay::emit_state(app, OverlayPayload { state: OverlayState::Failed, message: Some("transcription took longer than the limit; restarting the speech engine".into()), preview: None, can_retry: true, seconds: 0.0 });
            let engine = shared.engine.clone();
            let app2 = app.clone();
            let s2 = settings.clone();
            tokio::spawn(async move { engine.apply(&app2, &s2).await });
            finish_idle(shared, app, idle_timer, 3500);
            return;
        }
    };
    let tail_pitch = crate::audio::tail_pitch_features(&speech);
    if let Some((peak, end)) = tail_pitch {
        tracing::info!("prosody: final tail peak {peak:+.1} st, end {end:+.1} st | {}", crate::logging::redact(result.text.trim()));
    }
    let raw = {
        // the intonation applies to the last phrase only; the head parts got theirs above
        let tail_text = apply_intonation(result.text.trim(), tail_pitch, settings.cleanup.intonation_questions);
        let mut parts: Vec<String> = head_parts.clone();
        if !tail_text.is_empty() {
            parts.push(tail_text);
        }
        join_phrases(&parts)
    };
    tracing::info!("raw transcript ({} ms inference): {}", result.inference_ms, crate::logging::redact(&raw));
    if raw.is_empty() || is_hallucination(&raw) || (result.no_speech_prob.unwrap_or(0.0) > 0.85 && raw.split_whitespace().count() <= 2) {
        overlay::emit_state(app, OverlayPayload { state: OverlayState::NoSpeech, message: None, preview: None, can_retry: false, seconds: 0.0 });
        finish_idle(shared, app, idle_timer, 1200);
        return;
    }

    // 5. Cleanup.
    overlay::emit_state(app, OverlayPayload { state: OverlayState::Cleaning, message: None, preview: None, can_retry: false, seconds: 0.0 });
    let effective_lang = match settings.language.mode {
        LanguageMode::Greek => "el".to_string(),
        LanguageMode::English => "en".to_string(),
        _ => result.detected_language.clone().unwrap_or_else(|| if crate::cleanup::deterministic::looks_greek(&raw) { "el".into() } else { "en".into() }),
    };
    let opts = CleanupOptions {
        intensity: settings.cleanup.intensity.clone(),
        remove_fillers: settings.cleanup.remove_fillers,
        resolve_self_corrections: settings.cleanup.resolve_self_corrections,
        auto_punctuate: settings.cleanup.auto_punctuate,
        auto_capitalize: settings.cleanup.auto_capitalize,
        trailing_punctuation: ctx.trailing_punctuation,
        capitalize_first: ctx.capitalize_first,
        language: effective_lang.clone(),
    };
    let outcome = {
        let dict = shared.dict.read();
        let snips = shared.snippets.read();
        run_deterministic(&raw, &opts, &dict, &snips)
    };
    if outcome.cleaned.trim().is_empty() {
        overlay::emit_state(app, OverlayPayload { state: OverlayState::NoSpeech, message: None, preview: None, can_retry: false, seconds: 0.0 });
        finish_idle(shared, app, idle_timer, 1200);
        return;
    }
    let mut final_text = outcome.cleaned.clone();
    if settings.insertion.trailing_space && ctx.category != AppCategory::Code && ctx.category != AppCategory::Terminal && !final_text.ends_with('\n') {
        final_text.push(' ');
    }

    // 6. Insert.
    let target_alive = insertion::window_alive(ctx.target.hwnd);
    let mut insertion_method = "paste".to_string();
    // When the words cannot reach the window the user was in, they are not
    // allowed to vanish into the clipboard unseen: this holds the line that
    // explains where they went, and the notepad below shows them.
    let mut not_landed: Option<&'static str> = None;
    let (status, state, message) = if !target_alive {
        let r = tokio::task::spawn_blocking({
            let t = final_text.clone();
            move || insertion::copy_only(&t)
        })
        .await
        .unwrap_or(insertion::InsertReport { outcome: InsertOutcome::Failed, method: "copy".into(), message: None, elapsed_ms: 0 });
        insertion_method = r.method;
        not_landed = Some("window_closed");
        ("copied".to_string(), OverlayState::TargetChanged, Some("the window closed; the text is on the clipboard".to_string()))
    } else {
        let method = if ctx.target.elevated { InsertionMethod::CopyOnly } else { settings.insertion.method.clone() };
        let opts = InsertOptions { restore_clipboard: settings.insertion.restore_clipboard, settle_ms: settings.insertion.paste_settle_ms, shift_paste: ctx.shift_paste };
        let target = ctx.target.clone();
        let text = final_text.clone();
        let report = tokio::task::spawn_blocking(move || {
            // focus back to the target if it drifted
            if insertion::foreground_hwnd() != target.hwnd && !insertion::restore_focus(&target) {
                return (insertion::copy_only(&text), true);
            }
            let r = match method {
                InsertionMethod::CopyOnly => insertion::copy_only(&text),
                InsertionMethod::Type => insertion::type_text(&text),
                InsertionMethod::Paste => insertion::paste(&text, &opts),
                InsertionMethod::Auto => {
                    insertion::paste(&text, &opts)
                }
            };
            (r, false)
        })
        .await
        .unwrap_or((insertion::InsertReport { outcome: InsertOutcome::Failed, method: "paste".into(), message: Some("insertion task crashed".into()), elapsed_ms: 0 }, false));
        let (r, focus_lost) = report;
        tracing::info!("insertion: {:?} via {} in {} ms{}{}", r.outcome, r.method, r.elapsed_ms, if focus_lost { " (focus lost)" } else { "" }, r.message.as_deref().map(|m| format!(": {m}")).unwrap_or_default());
        let ok = matches!(r.outcome, crate::insertion::InsertOutcome::Pasted | crate::insertion::InsertOutcome::PastedNoRestore | crate::insertion::InsertOutcome::Typed);
        crate::journal::record(
            if ok { "info" } else { "warn" },
            "insert.done",
            serde_json::json!({
                "outcome": format!("{:?}", r.outcome),
                "method": r.method,
                "ms": r.elapsed_ms,
                "focus_lost": focus_lost,
                "message": r.message,
            }),
        );
        insertion_method = r.method.clone();
        match r.outcome {
            InsertOutcome::Pasted | InsertOutcome::PastedNoRestore | InsertOutcome::Typed => ("success".to_string(), OverlayState::Success, r.message),
            InsertOutcome::CopiedOnly if ctx.target.elevated => {
                not_landed = Some("elevated");
                ("copied".to_string(), OverlayState::TargetChanged, Some("the app runs as administrator; the text is on the clipboard (Ctrl+V)".to_string()))
            }
            InsertOutcome::CopiedOnly if focus_lost => {
                not_landed = Some("focus_lost");
                ("copied".to_string(), OverlayState::TargetChanged, Some("the active window changed; the text is on the clipboard (Ctrl+V)".to_string()))
            }
            InsertOutcome::CopiedOnly => ("copied".to_string(), OverlayState::Success, Some("copied to the clipboard".to_string())),
            InsertOutcome::PasteNotConsumed => {
                not_landed = Some("not_taken");
                ("copied".to_string(), OverlayState::TargetChanged, Some("the app did not accept the paste; the text is on the clipboard".to_string()))
            }
            InsertOutcome::Failed => {
                not_landed = Some("insert_failed");
                ("failed".to_string(), OverlayState::Failed, r.message)
            }
        }
    };

    // The words exist and the user cannot see them anywhere. Show them, unless
    // the user has said that a window appearing mid-work costs them more than
    // the loss does; the clipboard still holds the text either way.
    if let Some(reason) = not_landed {
        if settings.insertion.notepad_when_lost {
            crate::scratch::show(app, &final_text, reason, Some(ctx.friendly_name.clone()), "insert");
        } else {
            tracing::info!("notepad suppressed by settings; the text is on the clipboard");
        }
    }

    let latency_ms = released_at.elapsed().as_millis() as u64;
    let preview: String = final_text.trim().chars().take(60).collect();
    overlay::emit_state(app, OverlayPayload { state: state.clone(), message, preview: Some(preview), can_retry: state == OverlayState::Failed, seconds: 0.0 });
    shared.snapshot.lock().last_transcript = Some(final_text.trim().to_string());
    if status == "success" || status == "copied" {
        *last_recovery.lock() = None;
        let _ = std::fs::remove_file(&recovery_file);
    }

    // 7. History and stats.
    let word_count = crate::cleanup::deterministic::word_count(&final_text);
    let audio_path = if settings.privacy.keep_audio && settings.privacy.keep_history {
        let p = crate::paths::local_dir().join("audio").join(format!("{}.wav", crate::db::new_id()));
        let _ = std::fs::create_dir_all(p.parent().unwrap());
        if std::fs::write(&p, crate::audio::encode_wav(&speech)).is_ok() {
            Some(p.display().to_string())
        } else {
            None
        }
    } else {
        None
    };
    if settings.privacy.keep_history {
        let entry = HistoryEntry {
            id: crate::db::new_id(),
            created_at: crate::db::ts_now(),
            raw_text: raw.clone(),
            cleaned_text: outcome.cleaned.clone(),
            final_text: final_text.trim().to_string(),
            language: settings.language.mode.whisper_code().to_string(),
            detected_language: result.detected_language.clone(),
            app_name: Some(ctx.friendly_name.clone()),
            app_process: Some(ctx.target.process_name.clone()),
            app_category: Some(ctx.category.as_str().to_string()),
            cleanup_mode: format!("{:?}", settings.cleanup.intensity).to_lowercase(),
            rules_applied: outcome.applied.clone(),
            audio_ms,
            latency_ms,
            inference_ms: result.inference_ms,
            word_count,
            engine: Some(result.engine.clone()),
            model: Some(result.model.clone()),
            insertion_method: Some(insertion_method),
            status: status.clone(),
            audio_path,
            context_used: ctx.nearby_text.is_some(),
            retried: is_retry,
            undone: false,
            edited_text: None,
        };
        if let Err(e) = shared.db.insert_history(&entry) {
            tracing::error!("history insert failed: {e}");
        }
        let _ = app.emit_to("main", "lalia://history-changed", ());
    }
    if status == "success" || status == "copied" {
        let _ = shared.db.bump_daily(word_count, audio_ms, outcome.rule_ids.len() as u64);
        let _ = shared.db.bump_rule_usage(&outcome.rule_ids);
        let _ = shared.db.bump_snippet_usage(&outcome.snippet_ids);
    }
    if is_retry {
        let _ = shared.db.bump_daily_counter("retries");
    }
    tracing::info!("dictation {status}: {word_count} words, {audio_ms} ms audio, {latency_ms} ms release-to-insert ({} ms inference)", result.inference_ms);
    finish_idle(shared, app, idle_timer, if state == OverlayState::Success { 1400 } else { 4000 });
}

fn finish_idle(shared: &Arc<Shared>, app: &tauri::AppHandle, idle_timer: &mut Option<tokio::task::JoinHandle<()>>, ms: u64) {
    CAPTURE_ESCAPE.store(false, std::sync::atomic::Ordering::Relaxed);
    set_phase(shared, Phase::Idle, false);
    schedule_idle(app, shared, idle_timer, ms);
}

async fn paste_last(app: &tauri::AppHandle, shared: &Arc<Shared>, history_id: Option<String>) {
    let text = match history_id {
        Some(id) => shared.db.get_history(&id).ok().flatten().map(|h| h.final_text),
        None => {
            let mem = shared.snapshot.lock().last_transcript.clone();
            mem.or_else(|| shared.db.last_successful_history().ok().flatten().map(|h| h.final_text))
        }
    };
    let Some(text) = text else {
        overlay::emit_state(app, OverlayPayload { state: OverlayState::Failed, message: Some("no previous transcript".into()), preview: None, can_retry: false, seconds: 0.0 });
        return;
    };
    let settings = shared.settings.read().clone();
    let target = tokio::task::spawn_blocking(insertion::capture_target).await.unwrap_or_default();
    overlay::show(app, &settings.overlay, target.hwnd);
    let opts = InsertOptions { restore_clipboard: settings.insertion.restore_clipboard, settle_ms: settings.insertion.paste_settle_ms, shift_paste: false };
    let t = format!("{text} ");
    let r = tokio::task::spawn_blocking(move || insertion::paste(&t, &opts)).await.ok();
    let state = match r.as_ref().map(|r| &r.outcome) {
        Some(InsertOutcome::Pasted) | Some(InsertOutcome::PastedNoRestore) => OverlayState::Success,
        _ => OverlayState::TargetChanged,
    };
    overlay::emit_state(app, OverlayPayload { state, message: r.and_then(|r| r.message), preview: Some(text.chars().take(60).collect()), can_retry: false, seconds: 0.0 });
    let app2 = app.clone();
    let hide = settings.overlay.hide_when_idle;
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(1400)).await;
        overlay::emit_state(&app2, OverlayPayload { state: OverlayState::Idle, message: None, preview: None, can_retry: false, seconds: 0.0 });
        if hide {
            overlay::hide(&app2);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intonation_skips_list_items_and_marked_sentences() {
        let q = Some((11.0, 2.0));
        assert_eq!(apply_intonation("Ας μείνουμε σε αυτά για αρχή και βλέπουμε.", q, true), "Ας μείνουμε σε αυτά για αρχή και βλέπουμε;");
        assert_eq!(apply_intonation("Από Λος Άντζελες, Νέα Υόρκη.", q, true), "Από Λος Άντζελες, Νέα Υόρκη.");
        assert_eq!(apply_intonation("Τι κάνεις;", q, true), "Τι κάνεις;");
        assert_eq!(apply_intonation("Are you sure.", q, true), "Are you sure?");
        assert_eq!(apply_intonation("Πάμε.", Some((2.0, -3.0)), true), "Πάμε.");
        assert_eq!(apply_intonation("Πάμε.", q, false), "Πάμε.");
    }

    #[test]
    fn joining_phrases_drops_a_stray_leading_comma() {
        let parts = vec!["Δεν τη βλέπω να την έχεις κάνει.".to_string(), ", σε pixel.".to_string(), "Ωραία.".to_string()];
        assert_eq!(join_phrases(&parts), "Δεν τη βλέπω να την έχεις κάνει. σε pixel. Ωραία.");
        let parts = vec!["Καλημέρα".to_string(), ", πώς είσαι;".to_string()];
        assert_eq!(join_phrases(&parts), "Καλημέρα , πώς είσαι;");
    }

    #[test]
    fn join_prompt_combines_hints_and_previous_text() {
        assert_eq!(join_prompt(None, ""), None);
        assert_eq!(join_prompt(Some("Luram".into()), ""), Some("Luram".into()));
        assert_eq!(join_prompt(None, "Καλημέρα."), Some("Καλημέρα.".into()));
        assert_eq!(join_prompt(Some("Luram".into()), "Καλημέρα."), Some("Luram Καλημέρα.".into()));
        // only the tail of a long previous segment is kept
        let long: String = "α".repeat(300);
        assert_eq!(join_prompt(None, &long).unwrap().chars().count(), 200);
    }

    #[test]
    fn hallucination_list_is_exact_match_only() {
        assert!(is_hallucination("Υπότιτλοι AUTHORWAVE"));
        for text in ["you", "ευχαριστώ.", "ευχαριστώ πολύ.", "okay", "μπορείτε να με βοηθήσετε;"] {
            assert!(!is_hallucination(text));
        }
        assert!(!is_hallucination("you should call me"));
        assert!(!is_hallucination("ευχαριστώ για το email"));
    }
}
