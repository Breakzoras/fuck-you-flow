//! Microphone capture (WASAPI through cpal), pre-roll ring buffer, level meter,
//! energy-based voice activity gate and WAV encoding.
//!
//! A dedicated thread owns the cpal stream (cpal streams are not Sync). The
//! audio callback converts every incoming buffer to 16 kHz mono f32 and pushes it
//! into a shared recorder state. Recording is just a flag: when it flips on, the
//! pre-roll ring buffer is copied first so the first syllable survives.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;
use serde::Serialize;

pub const TARGET_RATE: u32 = 16_000;

#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StreamStatus {
    Closed,
    Open,
    Error,
}

/// Shared between the audio callback and the rest of the app.
struct Recorder {
    preroll: VecDeque<f32>,
    preroll_capacity: usize,
    recording: bool,
    buffer: Vec<f32>,
    /// Peak level of the last callback (0..1), for the overlay meter.
    level: f32,
    max_samples: usize,
    overflowed: bool,
    last_callback: Option<Instant>,
}

impl Recorder {
    fn push(&mut self, samples: &[f32]) {
        let mut peak = 0f32;
        for &s in samples {
            let a = s.abs();
            if a > peak {
                peak = a;
            }
        }
        self.level = peak;
        self.last_callback = Some(Instant::now());
        if self.recording {
            let remaining = self.max_samples.saturating_sub(self.buffer.len());
            self.buffer.extend_from_slice(&samples[..samples.len().min(remaining)]);
            self.overflowed |= samples.len() > remaining;
        } else if self.preroll_capacity > 0 {
            for &s in samples {
                if self.preroll.len() == self.preroll_capacity {
                    self.preroll.pop_front();
                }
                self.preroll.push_back(s);
            }
        }
    }
}

enum Cmd {
    Open { device_name: Option<String>, reply: Sender<Result<u32, String>> },
    Close,
    Shutdown,
}

/// Handle used by the pipeline. Cheap to clone.
#[derive(Clone)]
pub struct AudioCapture {
    rec: Arc<Mutex<Recorder>>,
    cmd_tx: Sender<Cmd>,
    status: Arc<Mutex<StreamStatus>>,
    open_device: Arc<Mutex<Option<String>>>,
}

impl AudioCapture {
    pub fn new(preroll_ms: u64, max_seconds: u64) -> Self {
        let rec = Arc::new(Mutex::new(Recorder {
            preroll: VecDeque::new(),
            preroll_capacity: (TARGET_RATE as u64 * preroll_ms / 1000) as usize,
            recording: false,
            buffer: Vec::new(),
            level: 0.0,
            max_samples: (TARGET_RATE as u64 * max_seconds) as usize,
            overflowed: false,
            last_callback: None,
        }));
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded::<Cmd>();
        let status = Arc::new(Mutex::new(StreamStatus::Closed));
        let open_device = Arc::new(Mutex::new(None));
        {
            let rec = rec.clone();
            let status = status.clone();
            let open_device = open_device.clone();
            std::thread::Builder::new()
                .name("lalia-audio".into())
                .spawn(move || audio_thread(rec, cmd_rx, status, open_device))
                .expect("spawn audio thread");
        }
        Self { rec, cmd_tx, status, open_device }
    }

    pub fn configure(&self, preroll_ms: u64, max_seconds: u64) {
        let mut r = self.rec.lock();
        r.preroll_capacity = (TARGET_RATE as u64 * preroll_ms / 1000) as usize;
        r.max_samples = (TARGET_RATE as u64 * max_seconds) as usize;
        while r.preroll.len() > r.preroll_capacity {
            r.preroll.pop_front();
        }
    }

    /// Open (or re-open) the input stream. Returns the native sample rate.
    pub fn open(&self, device_name: Option<String>) -> Result<u32, String> {
        let (reply, rx) = crossbeam_channel::bounded(1);
        self.cmd_tx.send(Cmd::Open { device_name, reply }).map_err(|e| e.to_string())?;
        rx.recv_timeout(Duration::from_secs(8)).map_err(|_| "audio thread did not answer".to_string())?
    }

    pub fn close(&self) {
        let _ = self.cmd_tx.send(Cmd::Close);
    }

    pub fn shutdown(&self) {
        let _ = self.cmd_tx.send(Cmd::Shutdown);
    }

    pub fn status(&self) -> StreamStatus {
        self.status.lock().clone()
    }

    pub fn open_device(&self) -> Option<String> {
        self.open_device.lock().clone()
    }

    pub fn is_open(&self) -> bool {
        *self.status.lock() == StreamStatus::Open
    }

    /// True when the callback delivered audio within the last second.
    pub fn is_alive(&self) -> bool {
        self.rec.lock().last_callback.map(|t| t.elapsed() < Duration::from_secs(1)).unwrap_or(false)
    }

    pub fn level(&self) -> f32 {
        self.rec.lock().level
    }

    pub fn start_recording(&self) {
        let mut r = self.rec.lock();
        r.buffer.clear();
        r.overflowed = false;
        let pre: Vec<f32> = r.preroll.iter().copied().collect();
        r.buffer.extend_from_slice(&pre);
        r.recording = true;
    }

    /// Stops and returns the captured 16 kHz mono samples (including pre-roll).
    pub fn stop_recording(&self) -> Recording {
        let mut r = self.rec.lock();
        r.recording = false;
        let samples = std::mem::take(&mut r.buffer);
        let overflowed = r.overflowed;
        r.overflowed = false;
        r.preroll.clear();
        Recording { samples, overflowed }
    }

    pub fn is_recording(&self) -> bool {
        self.rec.lock().recording
    }

    pub fn recorded_seconds(&self) -> f32 {
        self.rec.lock().buffer.len() as f32 / TARGET_RATE as f32
    }

    /// Samples captured so far in the current recording.
    pub fn recorded_len(&self) -> usize {
        self.rec.lock().buffer.len()
    }

    /// Copy of the samples in `from..to` of the current recording (clamped).
    /// Used to transcribe finished phrases while the recording continues.
    pub fn recorded_range(&self, from: usize, to: usize) -> Vec<f32> {
        let r = self.rec.lock();
        let to = to.min(r.buffer.len());
        if from >= to {
            return Vec::new();
        }
        r.buffer[from..to].to_vec()
    }

    pub fn discard(&self) {
        let mut r = self.rec.lock();
        r.recording = false;
        r.buffer.clear();
        r.preroll.clear();
    }
}

pub struct Recording {
    pub samples: Vec<f32>,
    pub overflowed: bool,
}

pub fn list_devices() -> Vec<DeviceInfo> {
    let host = cpal::default_host();
    let default_name = host.default_input_device().and_then(|d| d.description().ok()).map(|d| d.name().to_string());
    let mut out = Vec::new();
    if let Ok(devs) = host.input_devices() {
        for d in devs {
            if let Ok(desc) = d.description() {
                let name = desc.name().to_string();
                let is_default = default_name.as_deref() == Some(name.as_str());
                out.push(DeviceInfo { name, is_default });
            }
        }
    }
    out
}

fn find_device(name: Option<&str>) -> Option<cpal::Device> {
    let host = cpal::default_host();
    match name {
        Some(n) => {
            if let Ok(devs) = host.input_devices() {
                for d in devs {
                    if d.description().map(|x| x.name() == n).unwrap_or(false) {
                        return Some(d);
                    }
                }
            }
            tracing::warn!("microphone '{n}' not found, falling back to default");
            host.default_input_device()
        }
        None => host.default_input_device(),
    }
}

fn audio_thread(
    rec: Arc<Mutex<Recorder>>,
    cmd_rx: Receiver<Cmd>,
    status: Arc<Mutex<StreamStatus>>,
    open_device: Arc<Mutex<Option<String>>>,
) {
    let mut stream: Option<cpal::Stream> = None;
    loop {
        let Ok(cmd) = cmd_rx.recv() else { break };
        match cmd {
            Cmd::Open { device_name, reply } => {
                stream = None;
                *status.lock() = StreamStatus::Closed;
                match build_stream(device_name.as_deref(), rec.clone(), status.clone()) {
                    Ok((s, rate, resolved)) => {
                        if let Err(e) = s.play() {
                            let _ = reply.send(Err(format!("cannot start stream: {e}")));
                            *status.lock() = StreamStatus::Error;
                            continue;
                        }
                        stream = Some(s);
                        *status.lock() = StreamStatus::Open;
                        *open_device.lock() = Some(resolved);
                        let _ = reply.send(Ok(rate));
                    }
                    Err(e) => {
                        *status.lock() = StreamStatus::Error;
                        let _ = reply.send(Err(e));
                    }
                }
            }
            Cmd::Close => {
                stream = None;
                *status.lock() = StreamStatus::Closed;
                *open_device.lock() = None;
            }
            Cmd::Shutdown => break,
        }
    }
    drop(stream);
}

fn build_stream(
    device_name: Option<&str>,
    rec: Arc<Mutex<Recorder>>,
    status: Arc<Mutex<StreamStatus>>,
) -> Result<(cpal::Stream, u32, String), String> {
    let device = find_device(device_name).ok_or_else(|| "no microphone found".to_string())?;
    let resolved = device.description().map(|d| d.name().to_string()).unwrap_or_else(|_| "?".into());

    // Prefer a native 16 kHz mono config when the driver offers it; otherwise take
    // the default config and resample ourselves.
    let mut chosen: Option<cpal::StreamConfig> = None;
    if let Ok(ranges) = device.supported_input_configs() {
        for r in ranges {
            if r.channels() == 1
                && r.min_sample_rate() <= TARGET_RATE
                && r.max_sample_rate() >= TARGET_RATE
                && r.sample_format() == cpal::SampleFormat::F32
            {
                chosen = Some(r.with_sample_rate(TARGET_RATE).config());
                break;
            }
        }
    }
    let default = device.default_input_config().map_err(|e| format!("no default input config: {e}"))?;
    let sample_format = if chosen.is_some() { cpal::SampleFormat::F32 } else { default.sample_format() };
    let config = chosen.unwrap_or_else(|| default.config());
    let in_rate = config.sample_rate;
    let channels = config.channels as usize;
    tracing::info!("opening microphone '{resolved}' at {in_rate} Hz, {channels} ch, {sample_format:?}");

    static XRUNS: AtomicU32 = AtomicU32::new(0);
    // `None` until the first xrun, so the first one reports at once and the
    // rest are rate-limited from there.
    //
    // This used to hold `Instant::now() - Duration::from_secs(3600)` as a
    // "long ago" sentinel. On Windows an `Instant` counts from boot, so
    // subtracting an hour from it panics with "overflow when subtracting
    // duration from instant" whenever the machine has been up for less than
    // an hour, and the release profile aborts on panic. Measured 12 September
    // 2026: boot at 08:03, five aborts between 08:26 and 08:58, every one of
    // them inside that first hour, each killing the app mid-session without a
    // window or a message. Never build an `Instant` in the past.
    static XRUN_REPORTED: once_cell::sync::Lazy<Mutex<Option<Instant>>> =
        once_cell::sync::Lazy::new(|| Mutex::new(None));

    let err_status = status.clone();
    let err_cb = move |e: cpal::Error| {
        if matches!(e.kind(), cpal::ErrorKind::Xrun) {
            // A buffer over- or underrun while the microphone sits warm and
            // idle. Measured over two days: 2176 of these on 6 September and
            // 686 on 7 September, and not one of them fell inside any of the
            // 109 recordings that could be paired with an end event. They are
            // harmless, and at 40 percent of the file they were burying the
            // lines that matter. Counted here, reported once a minute.
            let n = XRUNS.fetch_add(1, Ordering::Relaxed) + 1;
            let now = std::time::Instant::now();
            let mut last = XRUN_REPORTED.lock();
            if last.map_or(true, |t| now.saturating_duration_since(t) >= Duration::from_secs(60)) {
                *last = Some(now);
                let total = XRUNS.swap(0, Ordering::Relaxed);
                tracing::warn!("audio stream notification: {total} buffer over- or underruns in the last minute ({e})");
            } else {
                let _ = n;
            }
        } else if matches!(e.kind(), cpal::ErrorKind::DeviceChanged | cpal::ErrorKind::RealtimeDenied) {
            // CPAL reports these while the stream remains active. The callback
            // watchdog will reopen it if samples actually stop arriving.
            tracing::warn!("audio stream notification: {e}");
        } else {
            tracing::error!("audio stream error: {e}");
            *err_status.lock() = StreamStatus::Error;
        }
    };

    let mut resampler = Resampler::new(in_rate, TARGET_RATE);
    let stream = match sample_format {
        cpal::SampleFormat::F32 => device
            .build_input_stream(
                config.clone(),
                move |data: &[f32], _| {
                    let mono = to_mono(data, channels);
                    let out = resampler.process(&mono);
                    rec.lock().push(&out);
                },
                err_cb,
                None,
            )
            .map_err(|e| e.to_string())?,
        cpal::SampleFormat::I16 => device
            .build_input_stream(
                config.clone(),
                move |data: &[i16], _| {
                    let f: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                    let mono = to_mono(&f, channels);
                    let out = resampler.process(&mono);
                    rec.lock().push(&out);
                },
                err_cb,
                None,
            )
            .map_err(|e| e.to_string())?,
        cpal::SampleFormat::U16 => device
            .build_input_stream(
                config.clone(),
                move |data: &[u16], _| {
                    let f: Vec<f32> = data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0).collect();
                    let mono = to_mono(&f, channels);
                    let out = resampler.process(&mono);
                    rec.lock().push(&out);
                },
                err_cb,
                None,
            )
            .map_err(|e| e.to_string())?,
        other => return Err(format!("unsupported sample format {other:?}")),
    };
    Ok((stream, in_rate, resolved))
}

fn to_mono(data: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return data.to_vec();
    }
    data.chunks(channels).map(|frame| frame.iter().sum::<f32>() / channels as f32).collect()
}

/// Windowed-sinc low-pass followed by linear-interpolated decimation. Good enough
/// for speech recognition (the model itself low-passes at 8 kHz).
pub struct Resampler {
    in_rate: u32,
    out_rate: u32,
    taps: Vec<f32>,
    history: Vec<f32>,
    phase: f64,
    pending: Vec<f32>,
}

impl Resampler {
    pub fn new(in_rate: u32, out_rate: u32) -> Self {
        let taps = if in_rate == out_rate { Vec::new() } else { design_lowpass(in_rate, out_rate) };
        Self { in_rate, out_rate, taps, history: Vec::new(), phase: 0.0, pending: Vec::new() }
    }

    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        if self.in_rate == self.out_rate {
            return input.to_vec();
        }
        // filter
        let n = self.taps.len();
        let mut work = std::mem::take(&mut self.history);
        work.extend_from_slice(input);
        let mut filtered = std::mem::take(&mut self.pending);
        for i in (n - 1)..work.len() {
            let mut acc = 0f32;
            for (k, t) in self.taps.iter().enumerate() {
                acc += t * work[i - k];
            }
            filtered.push(acc);
        }
        // keep last n-1 samples as history for the next block
        let keep = work.len().min(n - 1);
        self.history = work[work.len() - keep..].to_vec();
        // decimate with linear interpolation
        let ratio = self.in_rate as f64 / self.out_rate as f64;
        let mut out = Vec::with_capacity((filtered.len() as f64 / ratio) as usize + 2);
        let mut pos = self.phase;
        while pos + 1.0 < filtered.len() as f64 {
            let i = pos as usize;
            let frac = (pos - i as f64) as f32;
            out.push(filtered[i] * (1.0 - frac) + filtered[i + 1] * frac);
            pos += ratio;
        }
        let consumed = (pos.floor() as usize).min(filtered.len());
        self.pending = filtered[consumed..].to_vec();
        self.phase = pos - consumed as f64;
        out
    }
}

fn design_lowpass(in_rate: u32, out_rate: u32) -> Vec<f32> {
    // A bit under the lower of the two Nyquist limits, as a fraction of the
    // input rate.
    //
    // The output rate alone was used here, which is right going down and wrong
    // going up. A microphone running at 8 kHz, which is what a Bluetooth
    // headset gives while it is also being used as a headset, asked for a
    // cutoff of 0.9 of its own sample rate. Nothing above 0.5 exists to filter,
    // so the shape folded back on itself and made speech nearly twice as loud
    // as it should be: measured 10 September 2026, a 1 kHz tone came out at
    // 0.678 where 0.354 went in.
    let limit = in_rate.min(out_rate) as f64;
    let cutoff = 0.45 * limit / in_rate as f64; // fraction of input rate
    let n = 63usize;
    let m = (n - 1) as f64 / 2.0;
    let mut taps = Vec::with_capacity(n);
    let mut sum = 0.0;
    for i in 0..n {
        let x = i as f64 - m;
        let sinc = if x == 0.0 { 2.0 * cutoff } else { (2.0 * std::f64::consts::PI * cutoff * x).sin() / (std::f64::consts::PI * x) };
        let window = 0.54 - 0.46 * (2.0 * std::f64::consts::PI * i as f64 / (n - 1) as f64).cos();
        let t = sinc * window;
        sum += t;
        taps.push(t as f32);
    }
    for t in taps.iter_mut() {
        *t /= sum as f32;
    }
    taps
}

/// Result of the energy gate.
#[derive(Debug, Clone, Serialize)]
pub struct SpeechAnalysis {
    pub total_ms: u64,
    pub speech_ms: u64,
    pub trimmed: Vec<f32>,
    pub peak: f32,
}

/// RMS of a 20 ms frame below which the cheap gate counts it as silence.
const SILENCE_RMS: f32 = 0.003;

/// True when no 20 ms frame of `samples` rises above the silence gate. Used to
/// spot a pause in speech while recording.
pub fn is_silent(samples: &[f32]) -> bool {
    let frame = (TARGET_RATE / 50) as usize;
    samples.chunks(frame).all(|c| (c.iter().map(|s| s * s).sum::<f32>() / c.len().max(1) as f32).sqrt() <= SILENCE_RMS)
}

/// Centre of the quietest window of `win` samples inside `samples` (RMS over
/// 20 ms frames). Used to split a long breathless stretch between two words.
pub fn quietest_point(samples: &[f32], win: usize) -> usize {
    let frame = (TARGET_RATE / 50) as usize;
    if samples.len() <= win || win < frame {
        return samples.len() / 2;
    }
    let rms: Vec<f32> = samples.chunks(frame).map(|c| (c.iter().map(|s| s * s).sum::<f32>() / c.len().max(1) as f32).sqrt()).collect();
    let frames_per_win = (win / frame).max(1);
    let mut best = (f32::MAX, 0usize);
    for start in 0..rms.len().saturating_sub(frames_per_win) {
        let e: f32 = rms[start..start + frames_per_win].iter().sum::<f32>() / frames_per_win as f32;
        if e < best.0 {
            best = (e, start);
        }
    }
    ((best.1 * frame) + win / 2).min(samples.len())
}

/// Energy-based voice activity gate. Not a replacement for the neural VAD in
/// whisper-server; it exists to reject empty or accidental recordings quickly
/// (well under 500 ms) and to trim long silences before upload.
pub fn analyze_speech(samples: &[f32], min_speech_ms: u64) -> Option<SpeechAnalysis> {
    let frame = (TARGET_RATE / 50) as usize; // 20 ms
    if samples.len() < frame {
        return None;
    }
    let frames: Vec<f32> = samples
        .chunks(frame)
        .map(|c| (c.iter().map(|s| s * s).sum::<f32>() / c.len() as f32).sqrt())
        .collect();
    // This is only a cheap silence gate. Estimating noise from this recording
    // treats continuous speech as noise and rejects short, pause-free dictation.
    // The neural VAD in the engine handles speech/noise discrimination.
    let threshold = SILENCE_RMS;
    let peak = samples.iter().fold(0f32, |m, s| m.max(s.abs()));
    let speech: Vec<bool> = frames.iter().map(|&r| r > threshold).collect();
    let speech_frames = speech.iter().filter(|&&b| b).count();
    let speech_ms = speech_frames as u64 * 20;
    let total_ms = frames.len() as u64 * 20;
    if speech_ms < min_speech_ms {
        return Some(SpeechAnalysis { total_ms, speech_ms, trimmed: Vec::new(), peak });
    }
    let first = speech.iter().position(|&b| b).unwrap_or(0);
    let last = speech.iter().rposition(|&b| b).unwrap_or(speech.len() - 1);
    let pad = 15; // 300 ms
    let start = first.saturating_sub(pad) * frame;
    let end = ((last + 1 + pad) * frame).min(samples.len());
    Some(SpeechAnalysis { total_ms, speech_ms, trimmed: samples[start..end].to_vec(), peak })
}

/// Pitch shape of the last voiced stretch of an utterance, in semitones
/// relative to the utterance's median pitch: (peak of the final stretch minus
/// its start, level at the very end minus its start). Greek yes/no questions
/// tend to rise and fall on the last syllable. This is measurement only, a
/// couple of milliseconds per phrase, logged so the shape can be studied on
/// real speech without keeping any audio.
pub fn tail_pitch_features(samples: &[f32]) -> Option<(f32, f32)> {
    let sr = TARGET_RATE as usize;
    let frame = sr * 40 / 1000;
    let hop = sr * 10 / 1000;
    let (lag_min, lag_max) = (sr / 400, sr / 70);
    if samples.len() < frame * 4 {
        return None;
    }
    // analyse at most the last 1.5 s: enough for the final stretch, cheap on long phrases
    let start_at = samples.len().saturating_sub(sr * 3 / 2);
    let mut f0: Vec<f32> = Vec::new();
    let mut pos = start_at;
    while pos + frame <= samples.len() {
        let seg = &samples[pos..pos + frame];
        let mean = seg.iter().sum::<f32>() / frame as f32;
        let energy = (seg.iter().map(|s| (s - mean) * (s - mean)).sum::<f32>() / frame as f32).sqrt();
        if energy < 0.01 {
            f0.push(0.0);
            pos += hop;
            continue;
        }
        let r0: f32 = seg.iter().map(|s| (s - mean) * (s - mean)).sum::<f32>() + 1e-9;
        let mut best_lag = 0;
        let mut best = 0.0f32;
        for lag in lag_min..lag_max.min(frame - 1) {
            let mut acc = 0.0f32;
            for i in 0..frame - lag {
                acc += (seg[i] - mean) * (seg[i + lag] - mean);
            }
            let r = acc / r0;
            if r > best {
                best = r;
                best_lag = lag;
            }
        }
        f0.push(if best > 0.3 && best_lag > 0 { sr as f32 / best_lag as f32 } else { 0.0 });
        pos += hop;
    }
    let voiced: Vec<f32> = f0.iter().copied().filter(|v| *v > 0.0).collect();
    if voiced.len() < 12 {
        return None;
    }
    let mut sorted = voiced.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = sorted[sorted.len() / 2];
    let last = f0.iter().rposition(|v| *v > 0.0)?;
    let tail: Vec<f32> = f0[last.saturating_sub(40)..=last].iter().copied().filter(|v| *v > 0.0).map(|v| 12.0 * (v / median).log2()).collect();
    if tail.len() < 6 {
        return None;
    }
    let q = tail.len() / 3;
    let head_mean = if q > 0 { tail[..q].iter().sum::<f32>() / q as f32 } else { tail[0] };
    let peak = tail[tail.len().saturating_sub(2 * q.max(1))..].iter().copied().fold(f32::MIN, f32::max);
    let end = tail[tail.len() - 3..].iter().sum::<f32>() / 3.0;
    Some((peak - head_mean, end - head_mean))
}

/// Does the pitch shape of a phrase end sound like a yes/no question? Greek
/// questions rise on the last stressed syllable and stay up. Thresholds from 21
/// phrases of the developer's voice on 2026-09-05/06: the four questions
/// without a question word peaked at +9.0, +11.8, +12.4 and +19.5 semitones
/// with the ending at or above the phrase level; one statement out of
/// seventeen looked the same. Expect the occasional wrong mark.
pub fn sounds_like_question(peak: f32, end: f32) -> bool {
    peak >= 8.0 && end >= 0.0
}

/// 16-bit PCM mono WAV at 16 kHz, in memory.
pub fn encode_wav(samples: &[f32]) -> Vec<u8> {
    let spec = hound::WavSpec { channels: 1, sample_rate: TARGET_RATE, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut w = hound::WavWriter::new(&mut cursor, spec).expect("wav writer");
        for &s in samples {
            let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
            w.write_sample(v).expect("wav sample");
        }
        w.finalize().expect("wav finalize");
    }
    cursor.into_inner()
}

/// Decode a 16-bit or float WAV file into 16 kHz mono samples (used by the
/// evaluation harness and the benchmark).
pub fn decode_wav_file(path: &std::path::Path) -> anyhow::Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let channels = spec.channels as usize;
    let raw: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>().map(|s| s.map(|v| v as f32 / max)).collect::<Result<_, _>>()?
        }
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<_, _>>()?,
    };
    let mono = to_mono(&raw, channels);
    let mut rs = Resampler::new(spec.sample_rate, TARGET_RATE);
    Ok(rs.process(&mono))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No source file may build an `Instant` that sits in the past.
    ///
    /// On Windows an `Instant` counts from boot, so `Instant::now() - d`
    /// panics whenever the machine has been up for less than `d`, and the
    /// release profile aborts on panic. One such sentinel in this file killed
    /// the app five times inside one hour on 12 September 2026, each time
    /// silently and mid-session. A "long ago" marker is `Option::None`, never
    /// arithmetic on a clock. This guards every module, not just this one.
    #[test]
    fn no_instant_is_ever_built_in_the_past() {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut offenders = Vec::new();
        let mut stack = vec![src.clone()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).expect("read src") {
                let path = entry.expect("entry").path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }
                let text = std::fs::read_to_string(&path).expect("read file");
                for (i, line) in text.lines().enumerate() {
                    let code = line.trim_start();
                    // Skip comments: the explanation of this very bug quotes
                    // the pattern it forbids.
                    if code.starts_with("//") || code.starts_with("*") {
                        continue;
                    }
                    if let Some(rest) = code.split_once("Instant::now()").map(|(_, r)| r.trim_start()) {
                        if rest.starts_with('-') && !rest.starts_with("->") {
                            offenders.push(format!("{}:{}: {}", path.display(), i + 1, code));
                        }
                    }
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "subtracting from Instant::now() aborts the app when uptime is shorter              than the amount subtracted; use Option<Instant> instead:
{}",
            offenders.join("
")
        );
    }

    #[test]
    fn tail_pitch_sees_a_final_rise() {
        // 1.2 s of a 150 Hz tone whose last 0.3 s glides up to 260 Hz
        let sr = TARGET_RATE as f32;
        let mut phase = 0.0f32;
        let mut rising = Vec::new();
        let mut flat = Vec::new();
        for i in 0..(sr * 1.2) as usize {
            let t = i as f32 / sr;
            let f = if t > 0.9 { 150.0 + (t - 0.9) / 0.3 * 110.0 } else { 150.0 };
            phase += 2.0 * std::f32::consts::PI * f / sr;
            rising.push(0.3 * phase.sin());
            flat.push(0.3 * (2.0 * std::f32::consts::PI * 150.0 * t).sin());
        }
        let (rise_peak, _) = tail_pitch_features(&rising).expect("voiced");
        let (flat_peak, _) = tail_pitch_features(&flat).expect("voiced");
        assert!(rise_peak > 4.0, "rise {rise_peak}");
        assert!(flat_peak.abs() < 1.0, "flat {flat_peak}");
        assert!(tail_pitch_features(&vec![0.0f32; 16_000]).is_none());
    }

    #[test]
    fn quietest_point_finds_the_gap() {
        // 3 s of noise with a 200 ms dip at 2.0 s
        let mut x: Vec<f32> = (0..48_000).map(|i| if (i / 7) % 2 == 0 { 0.2 } else { -0.2 }).collect();
        for s in x[32_000..35_200].iter_mut() {
            *s *= 0.05;
        }
        let at = quietest_point(&x, 3_200);
        assert!((32_000..=35_200).contains(&at), "at {at}");
    }

    #[test]
    fn silence_gate_tells_pause_from_speech() {
        let quiet = vec![0.001f32; 16_000];
        assert!(is_silent(&quiet));
        let mut loud = quiet.clone();
        for (i, s) in loud.iter_mut().enumerate() {
            *s = if (i / 20) % 2 == 0 { 0.05 } else { -0.05 };
        }
        assert!(!is_silent(&loud));
        // one loud frame inside silence is enough to count as speech
        let mut one = vec![0.0f32; 16_000];
        for s in one[8_000..8_320].iter_mut() {
            *s = 0.1;
        }
        assert!(!is_silent(&one));
    }

    #[test]
    fn recorded_range_reads_the_live_buffer() {
        let cap = AudioCapture::new(0, 60);
        cap.start_recording();
        cap.rec.lock().buffer.extend((0..1000).map(|i| i as f32));
        assert_eq!(cap.recorded_len(), 1000);
        assert_eq!(cap.recorded_range(10, 13), vec![10.0, 11.0, 12.0]);
        assert_eq!(cap.recorded_range(990, 5000).len(), 10);
        assert!(cap.recorded_range(500, 400).is_empty());
        cap.discard();
    }

    #[test]
    fn silence_is_rejected() {
        let s = vec![0.0f32; TARGET_RATE as usize];
        let a = analyze_speech(&s, 200).unwrap();
        assert_eq!(a.speech_ms, 0);
        assert!(a.trimmed.is_empty());
    }

    #[test]
    fn continuous_quiet_audio_is_not_mistaken_for_its_own_noise_floor() {
        let samples: Vec<f32> = (0..16_000).map(|i| (i as f32 * 0.1).sin() * 0.01).collect();
        assert!(!analyze_speech(&samples, 200).unwrap().trimmed.is_empty());
    }

    #[test]
    fn tone_burst_is_kept_and_trimmed() {
        let mut s = vec![0.0f32; TARGET_RATE as usize * 2];
        for i in 16_000..24_000usize {
            s[i] = (i as f32 * 0.05).sin() * 0.4;
        }
        let a = analyze_speech(&s, 200).unwrap();
        assert!(a.speech_ms >= 400, "speech_ms={}", a.speech_ms);
        assert!(!a.trimmed.is_empty());
        assert!(a.trimmed.len() < s.len());
    }

    #[test]
    fn resampler_48k_to_16k_keeps_duration() {
        let input: Vec<f32> = (0..48_000).map(|i| (i as f32 * 0.01).sin()).collect();
        let mut r = Resampler::new(48_000, 16_000);
        let out = r.process(&input);
        assert!((out.len() as i64 - 16_000).abs() < 200, "len={}", out.len());
    }

    /// A microphone that runs slower than the engine must still be heard.
    ///
    /// The filter's cutoff was worked out from the output rate alone. Going up,
    /// from an 8 or 11 kHz microphone to the engine's 16 kHz, that put the
    /// cutoff above half the input rate, where the maths that builds the filter
    /// folds back on itself and the sound comes out quiet and distorted.
    ///
    /// Loudness and pitch are checked rather than the waveform itself, because
    /// any honest filter delays the sound a little and the shift alone would
    /// fail a sample by sample comparison.
    #[test]
    fn a_slow_microphone_still_comes_through() {
        for rate in [8_000u32, 11_025, 16_000, 22_050, 44_100, 48_000] {
            // 300 Hz and 1 kHz both sit in the middle of a speaking voice.
            for tone in [300.0f32, 1000.0] {
                let n = rate / 2;
                let input: Vec<f32> = (0..n)
                    .map(|i| (2.0 * std::f32::consts::PI * tone * i as f32 / rate as f32).sin() * 0.5)
                    .collect();
                let out = Resampler::new(rate, TARGET_RATE).process(&input);
                assert!(out.len() > 1000, "rate={rate} produced only {} samples", out.len());

                // Past the filter's warm up.
                let body = &out[200..out.len() - 200];

                let rms = (body.iter().map(|a| a * a).sum::<f32>() / body.len() as f32).sqrt();
                let want_rms = 0.5 / 2f32.sqrt();
                assert!(
                    rms < want_rms * 1.05,
                    "rate={rate} tone={tone}: came back louder than it went in ({rms:.3} against {want_rms:.3})"
                );
                assert!(
                    rms > want_rms * 0.8,
                    "rate={rate} tone={tone}: came back too quiet ({rms:.3} against {want_rms:.3})"
                );

                // The pitch is unchanged, whatever the rate it arrived at.
                let crossings = body.windows(2).filter(|w| (w[0] < 0.0) != (w[1] < 0.0)).count();
                let want_crossings = 2.0 * tone * body.len() as f32 / TARGET_RATE as f32;
                assert!(
                    (crossings as f32 - want_crossings).abs() < want_crossings * 0.1,
                    "rate={rate} tone={tone}: the pitch changed ({crossings} crossings, expected about {want_crossings:.0})"
                );
            }
        }
    }

    #[test]
    fn resampling_does_not_depend_on_callback_size() {
        for rate in [8_000, 44_100, 48_000] {
            let input: Vec<f32> = (0..rate).map(|i| (i as f32 * 0.13).sin() * 0.4).collect();
            let expected = Resampler::new(rate, TARGET_RATE).process(&input);
            for chunk_size in [1, 37, 128, 441, 1024] {
                let mut r = Resampler::new(rate, TARGET_RATE);
                let actual: Vec<f32> = input.chunks(chunk_size).flat_map(|c| r.process(c)).collect();
                assert_eq!(actual.len(), expected.len(), "rate={rate}, chunk={chunk_size}");
                assert!(actual.iter().zip(&expected).all(|(a,b)| (a-b).abs() < 0.0001));
            }
        }
    }

    #[test]
    fn recorder_fills_cap_and_zero_preroll_stays_empty() {
        let mut r = Recorder { preroll: VecDeque::new(), preroll_capacity: 0,
            recording: false, buffer: vec![], level: 0.0, max_samples: 5,
            overflowed: false, last_callback: None };
        r.push(&[0.1; 8]);
        assert!(r.preroll.is_empty());
        r.recording = true;
        r.push(&[0.1; 3]);
        r.push(&[0.2; 3]);
        assert_eq!(r.buffer.len(), 5);
        assert!(r.overflowed);
    }

    #[test]
    fn wav_round_trip() {
        let input: Vec<f32> = (0..1600).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let bytes = encode_wav(&input);
        assert!(bytes.len() > 44);
        let mut reader = hound::WavReader::new(std::io::Cursor::new(bytes)).unwrap();
        let n = reader.samples::<i16>().count();
        assert_eq!(n, 1600);
    }
}
