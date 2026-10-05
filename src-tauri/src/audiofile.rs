//! Turning a sound file that already exists into text.
//!
//! The dictation path records from the microphone. This one starts from a file
//! the user already has: a voice message, a meeting recording, a video's audio
//! track. Asked for by the first outside tester on 7 September 2026, whose own
//! words were "or make it record and transcribe the recording afterwards".
//!
//! Everything is decoded in Rust (symphonia), so there is no ffmpeg to install
//! and nothing leaves the machine. The samples are reduced to the one shape the
//! engine accepts, 16 kHz mono, while they are read, so an hour of stereo does
//! not sit in memory at its original size.

use crate::audio::{Resampler, TARGET_RATE};

/// Longer than this and we stop rather than fill the machine's memory. Ninety
/// minutes at 16 kHz is about 350 MB of samples, which is already generous.
const MAX_SECONDS: usize = 90 * 60;

/// The extensions the decoder is built for, for the file picker and the error.
pub const SUPPORTED: &[&str] = &["wav", "mp3", "m4a", "mp4", "aac", "flac", "ogg", "opus", "webm", "mkv", "wma", "aiff", "caf"];

/// Decode any supported sound file to the engine's shape: 16 kHz, mono, f32.
pub fn decode_to_engine_rate(path: &std::path::Path) -> Result<Vec<f32>, String> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
    use symphonia::core::errors::Error as SymError;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = std::fs::File::open(path).map_err(|e| format!("cannot open the file: {e}"))?;
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(&hint, stream, &FormatOptions { enable_gapless: true, ..Default::default() }, &MetadataOptions::default())
        .map_err(|e| format!("this file is not sound the app can read ({e})"))?;
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| "the file has no sound track".to_string())?;
    let track_id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("this sound format is not supported ({e})"))?;

    let mut out: Vec<f32> = Vec::new();
    // Built on the first decoded packet, when the real rate is known: a
    // container's header can disagree with what the decoder actually produces.
    let mut resampler: Option<Resampler> = None;
    let mut source_rate = 0u32;
    let mut source_channels = 0usize;
    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(SymError::ResetRequired) => break,
            Err(e) => return Err(format!("the file stops early ({e})")),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            // A damaged packet in the middle is worth skipping, not dying for.
            Err(SymError::DecodeError(_)) => continue,
            Err(SymError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(format!("the sound could not be decoded ({e})")),
        };
        let spec = *decoded.spec();
        if resampler.is_none() {
            source_rate = spec.rate;
            source_channels = spec.channels.count().max(1);
            resampler = Some(Resampler::new(spec.rate, TARGET_RATE));
        }
        let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        buf.copy_interleaved_ref(decoded);
        let interleaved = buf.samples();
        let channels = spec.channels.count().max(1);
        let mono: Vec<f32> = if channels == 1 {
            interleaved.to_vec()
        } else {
            interleaved.chunks(channels).map(|frame| frame.iter().sum::<f32>() / channels as f32).collect()
        };
        if let Some(r) = resampler.as_mut() {
            out.extend(r.process(&mono));
        }
        if out.len() > MAX_SECONDS * TARGET_RATE as usize {
            return Err(format!("the recording is longer than {} minutes; split it first", MAX_SECONDS / 60));
        }
    }
    if out.is_empty() {
        return Err("no sound was found in the file".into());
    }
    tracing::info!(
        "decoded {}: {} Hz, {} channel(s) -> {:.1} s at {} Hz",
        path.display(),
        source_rate,
        source_channels,
        out.len() as f32 / TARGET_RATE as f32,
        TARGET_RATE
    );
    Ok(out)
}

/// Where to cut a long recording so the engine never gets more than it can
/// hold comfortably. Cuts land on the quietest moment inside the last stretch
/// of each piece, so a sentence is far less likely to be split in half.
///
/// Returns the end index of every piece, in samples.
pub fn cut_points(len: usize, chunk_seconds: usize, samples: &[f32]) -> Vec<usize> {
    let chunk = chunk_seconds * TARGET_RATE as usize;
    if len <= chunk {
        return vec![len];
    }
    // Look for a pause in the last fifth of each piece, never before 60% of it.
    let search = chunk / 5;
    let mut cuts = Vec::new();
    let mut start = 0usize;
    while start + chunk < len {
        let from = start + chunk - search;
        let to = (start + chunk).min(len);
        // quietest_point already returns the centre of the quiet window.
        let window = TARGET_RATE as usize / 20; // 50 ms
        let local = crate::audio::quietest_point(&samples[from..to], window);
        let cut = (from + local).min(len);
        // Never go backwards, and never make a piece shorter than half.
        let cut = if cut <= start + chunk / 2 { start + chunk } else { cut };
        cuts.push(cut);
        start = cut;
    }
    cuts.push(len);
    cuts
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A recording shorter than one piece stays whole.
    #[test]
    fn short_audio_is_one_piece() {
        let samples = vec![0.0f32; TARGET_RATE as usize * 30];
        assert_eq!(cut_points(samples.len(), 120, &samples), vec![samples.len()]);
    }

    /// A long recording is cut, the pieces are in order, and the last cut is
    /// the end of the recording.
    #[test]
    fn long_audio_is_cut_in_order() {
        let len = TARGET_RATE as usize * 300;
        let samples = vec![0.1f32; len];
        let cuts = cut_points(len, 120, &samples);
        assert!(cuts.len() >= 3, "300 s in 120 s pieces needs at least 3 cuts, got {cuts:?}");
        assert_eq!(*cuts.last().unwrap(), len);
        for pair in cuts.windows(2) {
            assert!(pair[1] > pair[0], "cuts must grow: {cuts:?}");
        }
    }

    /// Decoding a file that really exists, for checking a format by hand:
    /// `LALIA_TEST_AUDIO=C:\path\to\file.mp3 cargo test decodes_a_real_file -- --ignored --nocapture`
    /// Ignored by default, since it needs a file this repository does not carry.
    #[test]
    #[ignore]
    fn decodes_a_real_file() {
        let path = std::env::var("LALIA_TEST_AUDIO").expect("set LALIA_TEST_AUDIO to a sound file");
        let samples = decode_to_engine_rate(std::path::Path::new(&path)).expect("the file should decode");
        let seconds = samples.len() as f32 / TARGET_RATE as f32;
        let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        let pieces = cut_points(samples.len(), 120, &samples).len();
        println!("{path}
  {seconds:.2} s at {} Hz, peak {peak:.3}, {pieces} piece(s)", TARGET_RATE);
        assert!(seconds > 0.1, "decoded almost nothing");
        assert!(peak > 0.001, "decoded silence");
    }

    /// The cut prefers a real pause: silence planted before the boundary wins
    /// over the loud audio around it.
    #[test]
    fn cut_lands_on_the_quiet_part() {
        let rate = TARGET_RATE as usize;
        let len = rate * 200;
        let mut samples = vec![0.5f32; len];
        let hush = rate * 110;
        for s in samples.iter_mut().skip(hush).take(rate) {
            *s = 0.0;
        }
        let cuts = cut_points(len, 120, &samples);
        let first = cuts[0];
        assert!(first >= hush && first <= hush + rate, "expected the cut inside the planted pause, got {first}");
    }
}
