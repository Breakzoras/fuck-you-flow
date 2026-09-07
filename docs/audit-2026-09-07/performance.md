# Lalia speech-to-text latency audit

Date: 2026-09-07
Scope: everything in the latency path except the GPU VRAM eviction incident, which is already root-caused and being fixed separately.
Method: read-only. Nothing was built, run or benchmarked. Every number below comes either from source lines cited as `file:line`, from the app's own logs, or from benchmark result files the project had already produced and committed under `eval/`.

Evidence base:

- `C:\Users\luram\AppData\Local\Lalia\logs\lalia.log.2026-09-06` and `...2026-09-07`, spanning 2026-09-06T05:52:51Z to 2026-09-07T14:24:47Z. 95 completed dictations (89 `success`, 6 `copied`, 0 `failed`), 355 segment transcriptions, 89 paste operations, 22 engine starts.
- Live configuration read from `C:\Users\luram\AppData\Roaming\Lalia\settings.json`: `language.mode = "greek"`, `asr.beam_size = 3`, `asr.threads = 12`, `asr.vad = true`, `asr.segment_while_speaking = true`, `asr.model_id = "large-v3-q5_0"`, `insertion.paste_settle_ms = 60`.
- Machine, from the log's own probe: `NVIDIA GeForce RTX 3070 (8017 MB, nvidia); 16 logical cores; 32693 MB RAM; vulkan true`.
- Committed benchmark results: `eval/latency-results.json`, `eval/optimization-results.json`, `eval/vulkan-results.json`, `eval/bench-summary.md`.
- Vendored engine source for parameter semantics: `vendor/whisper.cpp-src/examples/server/server.cpp` and `vendor/whisper.cpp-src/src/whisper.cpp`.

Six of the 89 successful dictations are the VRAM incident (release-to-insert 2010 ms to 37137 ms, all between 2026-09-06T14:09Z and 2026-09-07T11:13Z). They are excluded from the budget table below and analysed separately in Finding 4.

---

## 1. Where the milliseconds actually go

`release-to-insert` is the whole user-perceived tail: `Instant::now()` at `pipeline.rs:548` to `released_at.elapsed()` at `pipeline.rs:807`. The stage boundaries below were reconstructed from the log timestamps of `segments:` (`pipeline.rs:647`), `raw transcript` (`pipeline.rs:709`), `insertion:` (`pipeline.rs:783`) and `dictation success` (`pipeline.rs:870`).

Healthy runs only, n = 83, all values in milliseconds.

| # | Stage | Code | Median | p90 | Max | Share of median |
|---|---|---|---|---|---|---|
| A | Release to tail request sent: whole-recording energy scan, recovery WAV encode plus synchronous disk write, waiting for any still-running segment job, tail WAV encode | `pipeline.rs:554`, `572-575`, `617-653` | 0 | 155 | 746 | 0% |
| B | Tail whisper request (HTTP plus inference) | `pipeline.rs:668`, `whisper_server.rs:271-329` | 600 | 1095 | 1921 | 83% |
| C | Cleanup rules, dictionary, snippets, focus check | `pipeline.rs:696-749` | 3 | 11 | 73 | 0.4% |
| D | Insertion: clipboard publish, Ctrl+V, settle wait, clipboard restore | `insertion.rs:559-711` | 70 | 77 | 160 | 10% |
| E | History write and success log | `pipeline.rs:809-870` | 6 | 9 | 12 | 1% |
| | **Total release-to-insert** | | **722** | **1209** | **2010** | |

Across all 89 successes including the VRAM runs: median 735, p90 1358, max 37137.

Representative healthy lines:

```
2026-09-07T07:50:32.393799Z DEBUG lalia_lib::insertion::win: paste: 1 clipboard read(s), last read Some(1) ms after Ctrl+V, settle 60 ms
2026-09-07T07:50:32.394089Z  INFO lalia_lib::pipeline: insertion: Pasted via paste in 70 ms
dictation success: 279 words, 132900 ms audio, 564 ms release-to-insert (481 ms inference)
dictation success: 32 words, 13800 ms audio, 656 ms release-to-insert (579 ms inference)
```

The 132900 ms line is the segmentation working as designed: 133 seconds of speech, 564 ms of wait.

### The cost of one whisper request

Least-squares fit over the 377 non-overlapping `segment N: X ms audio, Y ms inference` lines outside the VRAM window (segments between 0.5 s and 13 s of audio):

```
inference_ms  =  355  +  0.079 x audio_ms
```

Fixed cost 355 ms per request; 79 ms per second of audio. Residual median -6 ms, p90 194 ms. Cross-check: the median tail is 3230 ms of audio, the model predicts 610 ms, and the measured median tail inference is 613 ms.

The split of that 355 ms across HTTP transport, WAV decode, mel spectrogram, the Silero VAD pass and the fixed-size encoder pass is **not verified**; it would need whisper-server's own timing output. What matters for planning is the shape: **most of the tail is a per-request constant, so shortening the tail audio has a modest ceiling, and cheapening the request itself has a large one.**

Tail audio length distribution (from the `tail N ms` field of `pipeline.rs:647`, n = 58): min 420, p25 2240, median 3230, p75 5210, p90 7290, max 10300.

---

## 2. Findings, ordered by milliseconds saved

### Finding 1. Beam size 3 costs 130 ms on every request. Measured.

- **File:line**: `src-tauri/src/settings.rs:144` (`beam_size: 3`), passed at `pipeline.rs:659` and `pipeline.rs:662` for the tail and at `pipeline.rs:175` for every background segment, sent to the engine at `asr/whisper_server.rs:289`. The engine turns `beam_size > 1` into beam search at `vendor/whisper.cpp-src/examples/server/server.cpp:918` and runs `beam_size` decoders at temperature 0 (`vendor/whisper.cpp-src/src/whisper.cpp:7088`).
- **Current behaviour**: beam search with 3 decoders.
- **Evidence**: `eval/latency-results.json`, `server: ctx-full`, `language: el`, same 12 clips per arm.

  | beam | mean ms | mean WER |
  |---|---|---|
  | 1 | 532.4 | 0.2099 |
  | 3 | 662.6 | 0.2085 |
  | 5 | 707.0 | 0.2288 |

  `eval/optimization-results.json` reproduces the direction on a separate run: beam 5 to beam 1 saves 210.6 ms mean (22.0%) with WER moving 0.2288 to 0.2099.
- **Proposed change**: `beam_size: 1` as the default in `settings.rs:144`.
- **Expected saving**: **130 ms median** off release-to-insert, which is 18% of the 722 ms median. The saving grows with tail length, because beam multiplies per-token decode work.
- **Also**: `dispatch_segment` passes the same beam (`pipeline.rs:175`), so every background segment gets cheaper too, which shrinks the window in which a tail request can queue behind a running segment. This links directly to Finding 3.
- **Risk**: WER moved 0.2085 to 0.2099 on 12 clips, which is well inside the noise of that sample size. The honest statement is that no accuracy loss was detectable at this corpus size. The comment already in the code at `settings.rs:143` records the earlier 5-to-3 move as "same accuracy on the corpus, about 6% faster", so the same reasoning applied one step further is consistent with the project's own precedent. Before shipping, re-run `eval/latency_check.py` on a larger corpus, ideally with the dictionary-term accuracy metric from `eval/bench-summary.md`, because beam search protects rare proper nouns more than it protects common words.

### Finding 2. The tail always carries at least 2.5 seconds of speech, by construction.

- **File:line**: `src-tauri/src/pipeline.rs:92-96`.

  ```rust
  const SEGMENT_PAUSE_MS: usize = 700;
  const SEGMENT_MIN_MS: usize = 2500;
  const SEGMENT_MAX_MS: usize = 12_000;
  ```

- **Current behaviour**: a segment closes only when at least 2500 ms of new audio has accumulated since the previous cut *and* the last 700 ms are silent by the energy gate (`pipeline.rs:428-436`, `audio.rs:475`). The check runs every eighth tick of a 33 ms interval, so roughly every 264 ms (`pipeline.rs:412`, `pipeline.rs:424`). Whatever follows the last cut becomes the tail. Measured median tail: 3230 ms of audio, which the cost model prices at 610 ms.
- **Proposed change**: lower `SEGMENT_MIN_MS` towards 1200 and `SEGMENT_PAUSE_MS` towards 450, so that a pause the user actually takes near the end of the utterance is more likely to become a cut. Raise the check frequency from every eighth tick to every fourth (roughly 132 ms) so a detected pause is acted on sooner, which removes up to 264 ms of stale audio from the tail.
- **Expected saving**: **around 105 ms median. Modelled; no measurement backs it yet.** Shifting the median tail from 3230 ms to roughly 1800 ms costs `0.079 x 1430 = 113 ms` less inference. Verifying this properly needs a replay of recorded audio through the segmenter with both threshold sets, which no current eval script does.
- **Risk**: three separate costs. First, whisper accuracy degrades on short chunks because there is less acoustic context; the existing mitigation is the previous segment's text fed back as a prompt (`pipeline.rs:150-160`), which helps wording consistency and does nothing for acoustics. Second, each extra segment adds a 355 ms fixed-cost request to the background queue, so a heavily segmented dictation does more total GPU work; this is free while the user is still speaking and stops being free the moment the GPU is slow (see Finding 4). Third, more cuts mean more phrase joins, and `join_phrases` (`pipeline.rs:103-119`) already has to repair punctuation artefacts at every boundary.
- **Note on the ceiling**: because 355 ms of each request is fixed, driving the tail to zero audio would still leave 355 ms. The only path below that is the tail-is-silent case, where no request is made at all (`pipeline.rs:656-658`, `670`). That fired 8 times in 89 dictations and produced the fastest runs in the whole log: `dictation success: 148 words, 79880 ms audio, 784 ms release-to-insert (0 ms inference)`, and one at 95 ms.

### Finding 3. Stage A blocks the tail request behind work that does not need to precede it. A p90 fix.

- **File:line**: `src-tauri/src/pipeline.rs:554` (energy scan of the whole recording), `572-575` (clone the whole speech buffer, encode it to WAV, write it to disk synchronously), `625-639` (await every segment job to completion), `641-645` (only then build the tail WAV), `668` (only then send the tail request).
- **Current behaviour**: the tail request stays unconstructed until every outstanding segment job has returned, even though the tail bytes depend only on `cut`, which is known the instant the key is released.
- **Measured**: stage A is 0 ms at the median because most segment jobs have long finished. It exceeds 100 ms in **14 of 89 dictations (16%)**, ranging 154 ms to 746 ms in healthy runs. Every one of those 14 had segments. Examples:

  ```
  2026-09-06 09:30:18   A=424 ms   1 segment   audio 12820 ms   segments: 1 finished while speaking (1027 ms inference), tail 1200 ms
  2026-09-07 02:38:53   A=699 ms  14 segments  audio 79880 ms
  2026-09-06 16:57:05   A=555 ms   1 segment   audio 11040 ms
  ```

- **Proposed change**: two independent edits.
  1. Build and send the tail request first, then await the head jobs while it is in flight. whisper-server serialises on one mutex (`vendor/whisper.cpp-src/examples/server/server.cpp:638`, `818`), so a queued tail starts the instant the running segment finishes, with no Rust-side await and no fresh HTTP round trip in between.
  2. Move the recovery WAV encode plus disk write (`pipeline.rs:572-575`) into a spawned task. It exists so a crash cannot lose the audio, and it does not need to complete before the request goes out. For a 78-second recording it writes about 2.5 MB.
- **Expected saving**: **0 ms at the median, 150 ms at p90, up to 750 ms on the worst healthy run.** Roughly one dictation in six.
- **Risk**: low for edit 1, since the tail bytes are already fully determined by `cut`. Edit 2 narrows the window in which a crash loses audio from "before the request" to "before the spawned write lands", a few milliseconds wider. If that matters, keep the write synchronous and only move the `encode_wav` allocation off the path.

### Finding 4. Segment concurrency is unbounded, which converts a slow GPU into an unbounded pile-up.

- **File:line**: `src-tauri/src/pipeline.rs:179` (`tokio::spawn` per segment), `193` (`jobs.push`), with no semaphore, no queue depth limit and no in-flight check anywhere in `dispatch_segment` or `EngineManager::transcribe` (`engine.rs:168-181`).
- **Server side**: whisper-server holds one global mutex around inference (`vendor/whisper.cpp-src/examples/server/server.cpp:638` and `818`), so concurrent POSTs queue. The client measures `inference_ms` from before the POST to after the response (`whisper_server.rs:272`, `327`), which means queue wait is counted as inference.
- **Measured**: over 355 segments in 63 segmented dictations, exactly **7 segments in 3 dictations** overlapped in time. One of those three was harmless:

  ```
  2026-09-06 12:15:56   rel=672 ms   7 segments   1 overlapping pair   per-segment inference 495..1188 ms
  ```

  The other two are inside the VRAM window and are catastrophic:

  ```
  2026-09-07 10:48:09   rel=37137 ms   3 segments   per-segment inference 19134, 28280, 37092 ms
  2026-09-07 10:54:41   rel=32838 ms   2 segments   per-segment inference 19764, 25875 ms
  ```

  Overlapped segments cost a median of 2013 ms of inference per second of audio against 146 ms for non-overlapped segments, a 14x ratio.
- **Diagnosis**: at healthy speed a segment finishes in about 750 ms while the next pause is at least 3.2 s away, so the queue drains between cuts and concurrency almost never happens. When per-segment cost rises to 19 s, cuts keep firing every few seconds while nothing completes, the queue grows without bound, and stage A ends up waiting 37 seconds for jobs that were dispatched before the user stopped speaking. Oversubscription of the single GPU was **not** the cause of the slowdown; the unbounded queue was the amplifier that turned it into a 37-second wait.
- **Proposed change**: a `tokio::sync::Semaphore` with one permit around the whole `engine.transcribe` call, plus a back-pressure rule in the level task: when a permit cannot be acquired immediately, skip the cut and let the audio fall through to the next boundary or to the tail. Optionally, abort outstanding segment jobs at release when the elapsed dispatch age already exceeds the recording length, and fall back to a single pass.
- **Expected saving**: **0 ms in normal operation.** It caps the worst case. On the two logged incidents it would have bounded the wait at roughly one segment's inference. Three queued ones is what actually happened.
- **Risk**: skipping a cut makes the tail longer for that dictation, which costs the 79 ms per second from the cost model. That is the correct trade when the engine is already saturated.

### Finding 5. The 60 ms paste settle is 50 ms of measured dead time.

- **File:line**: `src-tauri/src/insertion.rs:638`.

  ```rust
  match rx.recv_timeout(Duration::from_millis(opts.settle_ms.max(60))) {
      Ok(()) => continue,
      Err(_) => break,
  }
  ```

  Value from `settings.rs:205` (`paste_settle_ms: 60`), threaded through `pipeline.rs:762`.
- **Current behaviour**: the wait is condition-based in the sense that it extends when another clipboard read arrives, and the exit is always a timeout, so **the last `settle_ms` is always paid in full**. There is no early-exit path.
- **Measured, all 89 paste operations in the two logs**: every one reported exactly `1 clipboard read(s)`. Last-read latency after Ctrl+V: 0 ms x16, 1 ms x39, 2 ms x23, 3 ms x5, 4 ms x3, 5 ms x1, 9 ms x1, 10 ms x1. **Maximum observed 10 ms.** Total insertion is 70 ms median, of which 60 ms is this timeout.
- **Proposed change**: lower `paste_settle_ms` to 25 ms and remove the `.max(60)` floor at `insertion.rs:638` so the setting can actually take effect. 25 ms is 2.5x the worst read ever observed here.
- **Expected saving**: **35 ms median**, on every single dictation. About 5% of the 722 ms median.
- **Risk**: the settle exists so the clipboard survives until after the target's last read. An application that reads the clipboard lazily, more than 25 ms after the keystroke, would receive the restored clipboard content and the dictated text would be lost from view. The evidence covers Electron and Chromium targets only; the comment at `settings.rs:203-205` records that the previous cut from 180 ms to 60 ms was made on the same kind of evidence. Safer variant: keep 60 ms, and make the loop exit early when `rendered_after_keystroke >= 1` and no second read arrives within 15 ms. That recovers most of the 35 ms with no change in worst-case behaviour.

### Finding 6. Language auto-detect costs 220 ms per request. Fresh installs only.

**This machine escapes this cost.** `settings.json` has `"language": {"mode": "greek"}`, so `whisper_code()` returns `"el"` (`settings.rs:19-23`) and no auto-detect happens. The 722 ms budget above already excludes this cost.

- **File:line**: `src-tauri/src/settings.rs:319-322`.

  ```rust
  impl Default for LanguageModeSetting {
      fn default() -> Self {
          Self { mode: LanguageMode::Multi }
      }
  }
  ```

  `LanguageMode::Multi` maps to `"auto"` at `settings.rs:22`. That value is sent on every request at `whisper_server.rs:287` and on every background segment at `pipeline.rs:403` and `436`.
- **What "auto" costs in the engine**: `vendor/whisper.cpp-src/src/whisper.cpp:6849` triggers `whisper_lang_auto_detect_with_state`, which runs a **full extra encoder pass** (`whisper.cpp:4064`) plus one decoder step (`whisper.cpp:4071`) before the real transcription begins.
- **Evidence**: `eval/optimization-results.json`, same 12 clips, same beam 5, only the language field differing: mean 1129.5 ms at `auto` against 909.2 ms at `el`. **220.3 ms mean saving, 19.7%.** Mean WER 0.2218 at `auto` against 0.2288 at `el` on that corpus, so accuracy is unchanged within noise.
- **Proposed change**: make the first-run setup ask for the dictation language and write a concrete code, and change the default in `settings.rs:321` from `Multi` to a resolved language. Keep `Multi` available as an explicit user choice for genuinely bilingual dictation.
- **Expected saving**: **220 ms per request for every user who never opens the language setting**, on the tail and on every segment. Since the task's goal is ordinary consumer machines, this is the largest single item in the report for the population that matters, even though it is worth zero on the machine that produced the logs.
- **Risk**: a user who dictates in a language other than the configured one gets that language forced. The `Multi` mode also exists to feed a bilingual prompt, so the setup question has to be a real question, never a silent guess.

### Finding 7. Temperature fallback is enabled and can multiply a request by up to six.

- **File:line**: `src-tauri/src/asr/whisper_server.rs:283-284`.

  ```rust
  .text("temperature", "0.0")
  .text("temperature_inc", "0.2")
  ```

- **What it does**: `vendor/whisper.cpp-src/src/whisper.cpp:6891-6893` builds the temperature ladder `[0.0, 0.2, 0.4, 0.6, 0.8, 1.0]` from `temperature_inc`. When a decode fails the entropy or logprob check (`whisper.cpp:7583-7607`), the whole window is decoded again at the next temperature, with `best_of` decoders (server default 2, `server.cpp:81`). A pathological clip can therefore cost six decode passes. With `temperature_inc <= 0` the ladder collapses to a single entry (`whisper.cpp:6895-6896`).
- **How often it fires here: unverified.** The app logs only the total wall time of a request, and whisper-server logs the fallback at `WHISPER_LOG_DEBUG`, which this build does not capture. What can be said from the logs is that 28 of 378 segments (7.4%) cost more than twice the median 144 ms per second of audio, and that most of those are short segments where the 355 ms fixed cost naturally dominates. One genuine outlier stands out:

  ```
  2026-09-06 07:27:56  segment: 8640 ms audio, 5151 ms inference   (596 ms/s against a 144 ms/s median)
  ```

  That is consistent with a fallback but does not prove one.
- **Proposed change**: send `temperature_inc = 0.0`. Then verify by running whisper-server with debug logging enabled and counting `failed to decode with temperature` lines over a real dictation session, before deciding to keep it off.
- **Expected saving**: **0 ms at the median, and it removes a multi-second tail risk.** If the 7.4% figure were entirely fallback, which it is not, the ceiling would be about 40 ms of median saving.
- **Risk**: fallback exists to rescue a garbled decode. Turning it off means a bad window stays bad. The app already has downstream defences: the hallucination list (`pipeline.rs:198-202`), the `no_speech_prob > 0.85` guard (`pipeline.rs:710`) and the deterministic cleanup pass. A middle option is `temperature_inc = 0.4`, which halves the ladder to three rungs.

### Finding 8. audio_ctx is left at full, and lowering it is a small win with a real hazard.

- **File:line**: the app never sends `audio_ctx`. The server default is 0, meaning the full 1500-frame (30 second) encoder context (`vendor/whisper.cpp-src/examples/server/server.cpp:83`, applied at `server.cpp:935`).
- **Evidence**: `eval/latency-results.json`, `language: el`, `beam: 1`, same 12 clips: `ctx-full` 532.4 ms at WER 0.2099, `ctx-1024` 512.5 ms at WER 0.2035, `ctx-768` 477.2 ms at WER 0.2041.
- **Expected saving**: **55 ms**, incremental on top of Finding 1. Combining Finding 1 and this gives 662.6 to 477.2, but only 55 ms of that belongs to `audio_ctx`.
- **Hazard, which is why this is ranked low**: with `no_timestamps = true` (sent at `whisper_server.rs:285`), no timestamp tokens are decoded, so `seek_delta` stays at its initial `100 * WHISPER_CHUNK_SIZE`, that is 30 seconds (`vendor/whisper.cpp-src/src/whisper.cpp:7109`). The seek therefore advances a full 30 seconds per window while the encoder only sees `audio_ctx x 0.02` seconds of it. At `audio_ctx = 768` that is 15.36 seconds, so **audio between 15.36 s and 30 s of every window is silently dropped with no error**. The 12 eval clips are all shorter than 15 seconds, so the corpus cannot see this failure at all.
- **Where it would bite**: the whole-recording path at `pipeline.rs:662`, which fires when segmentation is off or when a segment failed, and which can carry up to `max_recording_seconds = 600` seconds of audio.
- **Proposed change, if taken at all**: set `audio_ctx` per request from the actual clip length, something like `min(1500, ceil(audio_secs * 50) + 64)`, and pass 0 unconditionally on the whole-recording fallback path. Never a fixed global value.
- **Risk**: beyond the truncation hazard, the encoder's positional embeddings were trained at 1500 frames; a 12-clip corpus showing slightly better WER at 768 falls well short of evidence that the model is unharmed.

### Finding 9. The turbo model is 223 ms faster and measurably less accurate. Offer it as an explicit user choice.

- **Evidence**: `eval/vulkan-results.json`, same 12 clips on the Vulkan backend: `large-v3-q5_0` mean 592.8 ms at WER 0.2346, `large-v3-turbo-q5_0` mean 369.8 ms at WER 0.2560. `eval/bench-summary.md` adds the metric that matters for dictation: dictionary-term accuracy falls from 0.833 to 0.5, and P50 falls from 783 ms to 304 ms.
- **Status**: already implemented as a setting (`settings.rs:118`, catalogue at `models.rs:58-69`), and the turbo model is already bundled (`src-tauri/target/release/bundled/models/ggml-large-v3-turbo-q5_0.bin`, 574 MB). `models.rs:346-354` already prefers turbo on machines with under about 2.6 GB of usable VRAM.
- **Proposed change**: no code change. Surface it in the interface as a speed setting with the term-accuracy trade named honestly.
- **Expected saving**: up to 223 ms, at the cost of losing one dictionary term in three.

---

## 3. Direct answers to the five questions

### Segmentation strategy

Boundaries come from a **fixed RMS energy gate. Silero plays no part in the cut decision.** `audio.rs:471` sets `SILENCE_RMS = 0.003`; `audio.rs:475-477` calls a 700 ms window silent when no 20 ms frame in it exceeds that. Silero runs only inside whisper-server, once per request, on audio that has already been cut (`engine.rs:114`, `whisper_server.rs:120-124`, `288`).

Three boundary triggers exist:

1. **Pause** (`pipeline.rs:428-436`): at least `SEGMENT_MIN_MS` 2500 of new audio plus a silent 700 ms tail. The cut lands mid-pause, at `len - pause / 2`, so both sides keep some silence.
2. **Length escape** (`pipeline.rs:437-445`): no pause for `SEGMENT_MAX_MS` 12000, so cut at the quietest 200 ms window inside the last 3 seconds (`audio.rs:482-495`).
3. **Release**: whatever follows the last cut becomes the tail.

The check runs every eighth tick of a 33 ms interval, roughly every 264 ms (`pipeline.rs:412`, `424`).

**Transcription is concurrent by construction and serial in practice.** Each cut spawns an independent task (`pipeline.rs:179`) with no limit of any kind. whisper-server then serialises them behind one mutex (`server.cpp:638`, `818`). Over 355 segments, only 7 overlapped, in 3 dictations. The single healthy overlap cost nothing (2026-09-06 12:15:56, release-to-insert 672 ms). The two harmful ones are the VRAM incident, where per-segment inference of 19 to 37 seconds made the queue grow faster than it drained.

Total GPU work against audio duration, for dictations that produced segments: median 4133 ms of total inference for 28620 ms of audio, that is 124 ms of inference per second of audio. Almost all of it overlaps with the user still speaking, which is exactly the design intent. The longest single case:

```
2026-09-07T05:27:02.326465Z  INFO lalia_lib::pipeline: segments: 20 finished while speaking (18295 ms inference), tail 1700 ms
```

18.3 seconds of GPU work, none of it in the user's tail.

### The tail

Work that lands entirely in the tail and is therefore never overlapped with speech:

| Work | Line | Cost |
|---|---|---|
| Energy scan of the whole recording | `pipeline.rs:554` | milliseconds, grows with length |
| Clone the whole speech buffer | `pipeline.rs:572` | milliseconds |
| Encode whole recording to WAV plus synchronous disk write | `pipeline.rs:574-575` | about 2.5 MB for 78 s of audio |
| Await every outstanding segment job | `pipeline.rs:625-639` | 0 ms median, 155 ms p90, 746 ms max |
| Analyse and encode the tail WAV | `pipeline.rs:641-645` | milliseconds |
| Tail whisper request | `pipeline.rs:668` | **600 ms median** |
| Pitch features over the whole speech buffer | `pipeline.rs:696` | milliseconds, inside stage C |
| Deterministic cleanup | `pipeline.rs:733-737` | 3 ms median |
| Insertion | `pipeline.rs:765-781` | 70 ms median |
| History row and audio file | `pipeline.rs:818-861` | after `latency_ms` is taken at `pipeline.rs:807`, so outside the measured tail |

**There is no end-of-utterance re-decode of the whole utterance.** This was the hypothesis and the code contradicts it. When segments succeed, the request built at `pipeline.rs:659` carries only `samples[cut..]` (`pipeline.rs:641`), and the head segments are never sent again. The `raw transcript (N ms inference)` line at `pipeline.rs:709` prints `result.inference_ms`, which is the **tail request only**, so a line like `raw transcript (1132 ms inference)` after segments have already finished is the tail alone. No re-decode happens.

`(0 ms inference)` has a single cause: `tail_silent` is true, so no request is made at all and a synthetic empty result with `inference_ms: 0` is substituted at `pipeline.rs:670`. That happened 8 times in 89 dictations, and produced the fastest runs in the log (95 ms, 176 ms, 303 ms).

**Redundant re-transcription does exist, on one path only.** If any segment job fails or is cancelled, `all_ok` goes false at `pipeline.rs:634-637`, `head_parts` is cleared, and the entire recording is transcribed from scratch at `pipeline.rs:662`. In two days of logs that fired **once**:

```
segment N failed, the recording will be transcribed in one pass
a segment failed; transcribing the whole recording in one pass
```

Rare, but on a 10-minute recording it would mean a full re-decode in the user's tail. Worth a cheaper fallback: keep the segments that did succeed and re-transcribe only the failed span.

### whisper-server parameters

Command line, `asr/whisper_server.rs:101-124`. Per-request form fields, `asr/whisper_server.rs:278-292`.

| Parameter | Where | Actual value | Latency cost | Faster setting | Accuracy trade |
|---|---|---|---|---|---|
| threads | `whisper_server.rs:109`, from `settings.rs:142` and `hw.rs:36-38` (`cores * 3/4`, clamped 4 to 16) | **12** on this machine, default 8 | Affects the CPU parts only on a GPU backend. The comment at `hw.rs:35` records 12 beating 8 and 16 by about 15% on this Ryzen | leave as is | none |
| beam size | `whisper_server.rs:289`, from `settings.rs:144` | **3** | **130 ms**, Finding 1 | 1 | flat within noise on 12 clips |
| greedy / beam search | chosen by the engine at `server.cpp:918` from `beam_size > 1` | **beam search** | see above | greedy at beam 1 | see above |
| best_of | never sent, server default `server.cpp:81` | **2** | none at temperature 0; beam search uses `beam_size` there (`whisper.cpp:7088`). Only reached during temperature fallback | no change needed | none |
| temperature | `whisper_server.rs:283` | **0.0** | correct already | no change | none |
| temperature fallback | `whisper_server.rs:284` | **`temperature_inc = 0.2`, so fallback is ON**, ladder of 6 (`whisper.cpp:6891`) | 0 ms typical, up to 6x on a bad window | `0.0` | a garbled window stays garbled, Finding 7 |
| no_context | not a per-request field; server default `server.cpp:109` and applied at `server.cpp:955` | **true, so no context carry-over between 30 s windows** | already the fast setting | no change | none |
| language | `whisper_server.rs:287`, from `settings.rs:19-23` | **`el` on this machine. `auto` on a fresh install**, because `LanguageModeSetting::default()` is `Multi` (`settings.rs:321`) | **220 ms when `auto`**, one extra full encoder pass (`whisper.cpp:6849`, `4064`) | explicit code | forces the language, Finding 6 |
| translate | never sent, server default `server.cpp:93` | **false** | none | no change | none |
| single-segment mode | never sent, no such request field in this server build | **off** | none | not applicable | not applicable |
| max_len | never sent; server turns 0 into 60 at `server.cpp:933` | **60 characters** | none measurable. Segment wrapping needs token timestamps, and `token_timestamps` is forced false by `no_timestamps = true` (`server.cpp:554-558`) | no change | none |
| no_timestamps | `whisper_server.rs:285` and CLI `whisper_server.rs:112` | **true** | already the fast setting. It is also what makes the `audio_ctx` hazard in Finding 8 real | no change | none |
| VAD | `whisper_server.rs:288`, model at `engine.rs:114`, threshold 0.5 at `whisper_server.rs:122` | **on** | Silero pass per request. `eval/vad-check-results.json` shows a silence-only request completing in 19 ms and a noise-only one in 26 ms, so the pass is cheap and it removes internal pauses from the encoder's work | keep on | keeping it is the accurate choice |
| flash attention | CLI `whisper_server.rs:114`; server default is already true (`server.cpp:106`) | **on** | already the fast setting | no change | none |
| suppress non-speech | CLI `whisper_server.rs:113` and per-request `whisper_server.rs:286` | **on** | negligible | no change | fewer bracketed non-speech tokens |
| audio_ctx | never sent, server default `server.cpp:83` | **0, meaning full 1500 frames** | 55 ms available, Finding 8 | per-request sizing only | silent audio loss if set blindly |
| response_format | `whisper_server.rs:282` | **`json`** | correct. The comment at `whisper_server.rs:280-281` records that `verbose_json` crashes the server when VAD removes all audio | no change | loses `detected_language`, which only matters in `auto` mode |

### Startup and warm-up

**Eager, and warmed.** `app.rs:191-193` spawns `engine.apply()` at application startup, well before any hotkey. `WhisperServer::start` waits for the port (`whisper_server.rs:181-205`, polling every 150 ms) and then sends a real inference request over 0.5 seconds of silence (`whisper_server.rs:206-209`) before reporting Ready. The `ready in N ms` figure is the whole span including that warm request:

```
2026-09-07T04:44:17.625277Z  INFO lalia_lib::asr::whisper_server: whisper-server ready in 2261 ms (model large-v3-q5_0)
```

Across 22 engine starts in the two logs the range is 1344 ms to 6469 ms, median around 2300 ms.

**The first real dictation runs at the same speed as any other.** Grouping every successful dictation by which engine start preceded it:

| | n | median release-to-insert | median tail inference | median audio |
|---|---|---|---|---|
| First dictation after an engine start | 14 | **660 ms** | 478 ms | 35530 ms |
| All later dictations | 69 | **727 ms** | 605 ms | 15840 ms |

The first-dictation values were 95, 362, 529, 536, 556, 640, 647, 673, 788, 929, 1039, 1048, 1358, 1480 ms; none of them is an outlier. The comparison is if anything conservative, because the first-dictation group had more than twice the median audio length.

**One structural gap with no measured cost.** The warm-up request uses `language: "en"`, `beam_size: 1` and `vad: false` (`whisper_server.rs:208`), while the first real request uses the configured language, `beam_size: 3` and `vad: true`. Two things are therefore first touched by the user's first dictation: the Silero VAD context, which is created lazily and then cached in engine state (`whisper.cpp:6681-6688`), and the beam-search KV cache, which is reallocated when the decoder count rises (`whisper.cpp:7159-7165`). Neither shows up in the numbers above, so this is a correctness-of-design point with no measured loss behind it. The fix is free: make the warm-up request mirror the live settings.

### Insertion

Total 70 ms median, 160 ms max, of which the 60 ms settle is the dominant term. See Finding 5. The settle is **not** a plain sleep; it is a `recv_timeout` that extends whenever another clipboard read arrives, and whose only exit is a timeout, so the final `settle_ms` is always paid. All 89 logged pastes reported exactly one clipboard read, none later than 10 ms after Ctrl+V.

Two other insertion waits are bounded but rarely hit. `wait_for_modifier_release(400 ms)` at `insertion.rs:584` exists because the toggle key can still be physically held when the text is ready; it timed out exactly **once** in the two logs (`modifiers still held after 400 ms`). `paste not consumed` appears 7 times, and each of those pays the full 700 ms first-read timeout at `insertion.rs:633` before giving up; retries were deliberately disabled after a double-paste incident on 2026-09-05 (`insertion.rs:648-651`).

---

## 4. Every hardcoded sleep, poll and timeout in the insertion and pipeline paths

Marked **[HOT]** when it can land inside release-to-insert.

### `src-tauri/src/insertion.rs`

| Line | Value | What | Hot? |
|---|---|---|---|
| 223 | 40 ms sleep x 12 | `open_clipboard_retry` back-off | [HOT] only under clipboard contention |
| 428 | 5 ms poll, 3 s cap | `ensure_started` waits for the clipboard thread | startup only |
| 481 | 10 ms poll | `wait_for_modifier_release` inner loop | [HOT] |
| 537 | 15 ms sleep | after `SendInput` releases held modifiers | [HOT] only when a modifier was held |
| 577 | 2 s timeout | `wait_done` for the delayed-render publish | [HOT] as a cap |
| 584 | **400 ms** cap | `wait_for_modifier_release` | [HOT], timed out once in 2 days |
| 609 | 2 s timeout | `wait_done` after refusing an unusable target | failure path |
| 633 | **700 ms** first attempt, 500 ms later | wait for the target's first clipboard read | [HOT], paid in full on the 7 not-consumed cases |
| 638 | **`settle_ms.max(60)`** | quiet period after the last read; always paid once | [HOT], Finding 5 |
| 683 | 2 s timeout | `wait_done` on the not-consumed path | failure path |
| 704 | 2 s timeout | `wait_done` for the clipboard restore | [HOT] as a cap |
| 722 | 2 s timeout | `wait_done` in `copy_only` | [HOT] on the copy path |
| 757 | 4 ms sleep per 64 inputs | `type_text` chunk pacing | [HOT] only in Type mode |
| 873, 875 | 20 ms then 30 ms sleep | `restore_focus` ALT-press trick | [HOT] only when focus drifted |

### `src-tauri/src/pipeline.rs`

| Line | Value | What | Hot? |
|---|---|---|---|
| 245 | `tap_ms` (280 ms default) | tap-to-hands-free discrimination | pre-recording |
| 262 | 1500 ms | window for adding Space to switch to hands-free | pre-recording |
| 375 | **400 ms** timeout | UI Automation password-field check | at recording start, never in the tail |
| 412 | 33 ms interval | level meter tick | during recording |
| 424 | every 8th tick, about 264 ms | segment boundary check cadence | during recording, but it delays each cut and so lengthens the tail |
| 525 | caller-supplied ms | overlay idle timer | after insertion |
| 588 | 200 ms sleep x 40, 8 s total | wait for a Starting engine | [HOT] only when the engine has yet to report Ready |
| 666 | `(20 + audio_ms / 2000).max(60)` s | outer transcription timeout | [HOT] as a cap |
| 906 | 1400 ms | overlay idle timer in `paste_last` | after insertion |

### `src-tauri/src/asr/whisper_server.rs`

| Line | Value | What | Hot? |
|---|---|---|---|
| 71 | 65 s | reqwest client default timeout | [HOT] as a cap |
| 180 | 120 s | engine start deadline | startup only |
| 204 | 150 ms poll | wait for the port to accept | startup only |
| 298 | `(20 + audio_secs / 2).max(65)` s | per-request timeout | [HOT] as a cap |

### `src-tauri/src/audio.rs`

| Line | Value | What | Hot? |
|---|---|---|---|
| 128 | 8 s | audio thread command timeout | at recording start |
| 153 | 1 s | staleness window for `is_alive` | watchdog |

None of the tail-critical waits is a plain `thread::sleep` on a fixed duration except the 60 ms settle floor and the modifier-release poll. The rest are caps on conditions.

---

## 5. What turned out to be fine

Stating these plainly, because each was a live hypothesis and each is answered by evidence.

- **No end-of-utterance full re-decode exists.** The tail request covers `samples[cut..]` only (`pipeline.rs:641`). `(0 ms inference)` is the synthetic no-request result when the tail is silent (`pipeline.rs:670`). Whole-recording re-transcription happens only on the segment-failure fallback, which fired once in two days.
- **Cold start is a solved problem.** The engine starts eagerly at app launch and is warmed with a real inference before reporting Ready. First dictations are, if anything, faster than later ones.
- **Segment concurrency does not oversubscribe the GPU under normal load.** 7 overlapping segments out of 355, and the one healthy overlap cost nothing. The cap proposed in Finding 4 is a safety measure for degraded GPUs. It buys no throughput.
- **`no_context`, `translate`, `flash_attn`, `no_timestamps` and `response_format` are all already at the fast and correct settings.**
- **Cleanup is free.** The full deterministic pass over rules, dictionary and snippets runs in 3 ms at the median, 73 ms worst case.
- **The thread count comes from a real measurement.** `hw.rs:35-38` records a sweep on this class of machine.

---

## 6. Summary of expected savings

Against the 722 ms healthy median on the machine that produced these logs.

| Finding | Change | Median saving | Evidence |
|---|---|---|---|
| 1 | beam 3 to 1 | **130 ms** | measured, `eval/latency-results.json` |
| 2 | shorter segment thresholds and faster boundary checks | **~105 ms** | modelled from the fitted cost model; unmeasured |
| 5 | paste settle 60 ms to 25 ms | **35 ms** | measured, 89 paste log lines |
| 8 | per-request `audio_ctx` | 55 ms | measured, hazardous without a length guard |
| 3 | reorder `process()` so the tail request goes first | 0 ms median, **150 ms at p90**, 750 ms worst healthy case | measured, 14 of 89 dictations |
| 7 | `temperature_inc = 0` | 0 ms median, removes a multi-second tail risk | frequency unverified |
| 4 | cap concurrent segment requests at 1 | 0 ms median, caps a 37 s worst case | measured on the two incident runs |
| 6 | resolve the language at first run | **220 ms, fresh installs only** | measured, `eval/optimization-results.json` |
| 9 | turbo model as an explicit speed setting | up to 223 ms | measured, at a third of dictionary-term accuracy |

Findings 1, 2 and 5 together are about 270 ms, taking the median from 722 ms to roughly 450 ms. Below that the 355 ms fixed cost of a single whisper request becomes the floor, and further progress needs either a smaller model (Finding 9) or making the tail silent more often (the `tail_silent` path in Finding 2).
