# Reliability fixes and optimization check — 2026-09-05

## Changes

- Normal cleanup preserves negation, ambiguous correction words, meaningful phrases such as «να πούμε», and repetitions such as «σιγά σιγά». Greek correction requires an explicit comma-delimited «διόρθωση» or «όχι όχι». Strong mode still removes more content.
- Ordinary short replies («ευχαριστώ», «okay», «μπορείτε να με βοηθήσετε;») are no longer blacklisted as hallucinations. Empty output and high no-speech probability remain filtered.
- Streaming resampling retains the interpolation boundary between microphone callbacks. Tests compare an entire waveform with callbacks of 1, 37, 128, 441 and 1024 samples at 8, 44.1 and 48 kHz.
- The cheap energy gate no longer estimates noise from the same utterance: that rejected continuous speech. It uses RMS 0.003; neural VAD remains responsible for discriminating speech from noise. Noisy real-microphone behavior still needs evaluation, especially if VAD is disabled.
- Recording fills the final partial buffer at the duration limit, allowing the automatic stop to trigger. Zero pre-roll no longer grows an unbounded buffer.
- Tap duration begins before microphone/context initialization, ahead of the UI Automation delay.
- Auto insertion no longer types a second copy after an unconfirmed clipboard read. An unconfirmed paste leaves the text available for manual paste.
- An empty cleaned transcript no longer inserts a space.
- Existing user changes to the default shortcuts were preserved; the stale test expectation now matches RAlt.

- Startup on the Razer Barracuda X reported a CPAL buffer overrun/underrun notification. These recoverable notifications no longer mark an active stream as failed. The idle watchdog now also retries closed/failed warm streams. Disconnect/reconnect still needs a physical test.

## Verification

44 unit tests passed; 1 manual live-inference test ignored. Frontend type-check and production asset build passed. A separate real local Whisper server was exercised against the existing synthetic Greek corpus. With neural VAD enabled, both existing silence/noise samples returned empty text (19/26 ms); see `eval/vad-check-results.json`. This is a two-sample check; general noise robustness remains open.

## Local ASR comparison

Same large-v3-q5_0 model, GPU, no prompt, one pass per configuration. WER is the mean of per-utterance WERs; latency is the arithmetic mean of request times; end-to-end insertion time is measured separately.

| Language | Beam | Mean WER | Mean inference ms |
|---|---:|---:|---:|
| Auto | 5 | 0.222 | 1130 |
| Greek | 5 | 0.229 | 909 |
| Greek | 1 | 0.210 | 699 |

Greek/beam 1 was about 38% faster in this small run. These figures do not establish better accuracy on human speech or English; language and beam defaults were therefore retained. Previous benchmark numbers used dictionary prompts and are not directly comparable. Reproduce from the project root with `python eval/optimize_check.py`. Per-utterance results are in `eval/optimization-results.json`.

## Remaining verification and limitations

Dictation can still make errors. Real human microphone capture and insertion into the user's usual applications still need a manual pass. Escape during transcription is queued behind processing in the current actor and takes effect once processing finishes. Clipboard restoration still only preserves plain text. Application focus restoration still needs manual checking. No new ASR model or AI rewrite stage was added, and no cloud audio requests were made.

## Human acceptance pass

After opening the new release, dictate into Notepad and then the usual chat application:

1. «Θέλω καφέ όχι τσάι.»
2. «Θα έρθω την Τρίτη όχι την Παρασκευή.»
3. «Θέλω να πούμε κάτι και να πάμε σιγά σιγά.»
4. «Ευχαριστώ.»
5. «Ο server τρέχει στο Webdock και η βάση είναι PostgreSQL.»

Check that all words survive, one recording produces one insertion, holding and releasing the configured key ends recording, and a second recording starts normally. Repeat a short phrase softly and without a leading pause. Compare raw versus cleaned history to distinguish recognition errors from cleanup errors. Do not treat the synthetic benchmark as a human acceptance pass.

## Afternoon: latency work, measured on Lu's own dictations

Where the time went before (release of the key to text on screen): recognition of the whole recording plus a fixed 180 ms paste wait. 5 s of speech took 0.95 s, 14 s took 1.45 s; the wait grew with the length of the dictation.

Changes:

1. Segment transcription while speaking (`pipeline.rs`, `Segmenter`). A pause of 700 ms after at least 2.5 s of audio sends the finished phrase to the engine while the key is still down, with the previous phrase (last 200 characters) plus the dictionary hints as prompt context. On stop, only the audio after the last cut is transcribed. If any segment fails, the whole recording is transcribed in one pass. Off switch: `asr.segment_while_speaking`.
2. Paste settle 180 ms to 60 ms. The paste path now logs how many times the target read the clipboard and when; Electron (Claude desktop) reads once, 2 to 3 ms after Ctrl+V.
3. Beam size 5 to 3 (default); Lu's profile also forces Greek, which skips automatic language detection. On the synthetic corpus, detection cost about 150 ms per request; forced Greek kept English test sentences in English (only "um/uh" became Greek fillers, which cleanup removes). Audio-context reduction (`-ac 1024/768`) was slower or less accurate and was rejected. Numbers: `eval/latency_check.py`, `eval/latency-results.json`.
4. Engine request timeout and the pipeline limit now scale with recording length (20 s plus 0.5 s per audio second, never under 60 s), so the 10-minute cap cannot time out.

Measured after the change (Lu, Claude desktop, Greek with English terms):

| Words | Speech | Segments while speaking | Release to text |
|---:|---:|---:|---:|
| 32 | 17.5 s | 1 | 592 ms |
| 44 | 20.9 s | 2 | 578 ms |
| 65 | 24.8 s | 2 | 1176 ms (last phrase 7.3 s) |
| 57 | 25.1 s | 2 | 1389 ms (no pause in the last 13 s) |
| 179 | 76.7 s | 5 | 533 ms |

The wait now depends on the length of the last phrase, and no longer on the length of the whole dictation. A short pause before pressing the key again gives the fastest result.

Not done: the f16 large-v3 model (3.1 GB, likely faster and slightly more accurate on the RTX 3070) awaits Lu's approval of the download; beam 1 was 38 percent faster on the synthetic corpus with equal accuracy and stays optional until human recordings confirm it.
