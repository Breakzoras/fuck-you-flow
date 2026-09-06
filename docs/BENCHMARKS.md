# Benchmarks (measured, 2026-09-05)

Machine: AMD Ryzen 7 5800X3D, 32 GB RAM, NVIDIA GeForce RTX 3070 8 GB, driver 610.88, Windows 10 22H2. Engine: whisper.cpp b4938 CUDA 12.4 build, flash attention on, 8 threads, beam size 5, temperature 0.
Corpus: `eval/corpus` (16 synthetic sentences from Microsoft Edge neural voices, Greek, English and mixed, plus 3 s of digital silence and 3 s of pink noise). Harness: `eval/bench.py`. Raw results: `eval/bench-results.json`.

WER and CER are computed against the words as spoken (numbers written out in words in the reference), so every "τριακόσια πενήντα" that the model correctly writes as "350" counts against it. Treat the numbers as relative, and read the transcripts below.

## Speech models

| Model | Size on disk | Load to first answer | Greek WER | Greek CER | English WER | Dictionary terms recognised | Hallucination on silence and noise (VAD off) | P50 per 5 s clip | Max |
|---|---|---|---|---|---|---|---|---|---|
| Whisper large-v3 q5_0 (default) | 1.08 GB | 1.5 s | 0.182 | 0.144 | 0.212 | 5 of 6 | 2 of 2 ("you", "Okay.") | 783 ms | 1238 ms |
| Whisper large-v3-turbo q5_0 | 574 MB | 1.0 s | 0.250 | 0.171 | 0.212 | 3 of 6 | 2 of 2 ("you", "Thank you.") | 304 ms | 397 ms |
| Whisper medium q5_0 | 539 MB | 1.0 s | 0.219 | 0.149 | 0.212 | 5 of 6 | 2 of 2 ("you") | 560 ms | 863 ms |

Decision: large-v3 stays the default. It makes the fewest Greek errors and keeps names best; its 0.8 s per sentence is well inside the 1.5 s target. Turbo is 2.6 times faster and is offered for machines with less GPU memory; medium sits in between with no advantage on this hardware.

Real-time factor on the GPU: 0.08 to 0.23 for large-v3 (a 5 s clip takes about 1 s), 0.04 to 0.08 for turbo.

## What the transcripts show

Greek sentence with an English brand inside (el-05), all three models:

> Η εταιρεία μου λέγεται Luram και φτιάχνει αυτοματισμούς με τεχνητή νοημοσύνη για μικρές επιχειρήσεις.

large-v3 and medium wrote it exactly (the term "Luram" came from the dictionary hint prompt). Turbo wrote "LuRam"; the Dictionary rule "LuRam -> Luram" would fix it after the fact, and the case-insensitive whole-word match already does.

Names and places (el-09): all three exact.

Numbers and dates (el-04): all three wrote "Θα πληρώσω 350 ευρώ στις 15 Σεπτεμβρίου 2026." (counted as errors by WER because the reference spells the numbers out).

Self-correction (el-07): large-v3 "Θα σε πάρω τηλέφωνο την Δετάρτη, όχι την Πέμπτη το πρωί." The deterministic cleanup turns this into "Θα σε πάρω τηλέφωνο την Πέμπτη το πρωί." (unit test `resolves_self_corrections`). "Δετάρτη" is a synthetic-voice artefact; the human word is Τετάρτη.

Fillers (el-08, en-03): large-v3 wrote "YYY." for a long "εεε"; turbo wrote "Ε, Ε, Ε."; English "Em, ... ah,". The filler list now includes ε, em, ah, ehm.

Greek question mark (el-10): large-v3 and medium wrote "Τι κάνεις?" with a Latin question mark. Cleanup now converts "?" to ";" in Greek text.

Spoken email addresses (el-02, en-04): "info παπάκι oneclickclaw τελεία io" came out garbled in every model; "accounts at luram dot gr" became "accounts at lurum.gr". A spoken-address rule is on the backlog; a Dictionary entry for the domain fixes the brand part today.

Technical terms (el-03): "OpenClaw" and "Webdock" survive in all models (case differs, the Dictionary normalises it); "PostgreSQL" became "πιο στην SQL" in large-v3 and turbo, medium got it right. Hint prompts help but do not guarantee rare tokens.

Silence and noise with VAD switched off produce the classic Whisper hallucinations ("you", "Okay.", "Thank you."). In the app three guards sit in front of the model: the energy gate rejects recordings with less than 200 ms of speech, Silero VAD inside whisper-server removes non-speech, and a hallucination list drops exact matches such as "you".

## End-to-end timings in the app (debug build, harness `eval/e2e_notepad.py`)

| Clip | Audio | Inference | Key release to text in Notepad |
|---|---|---|---|
| el-05 (Greek, 14 words) | 5.56 s | 1167 ms | 1374 ms |
| en-01 (English, 15 words) | 4.74 s | 600 ms | 787 ms |

Paste through the clipboard with delayed rendering and restore: 184 to 190 ms of the total. Overlay reaction to the hotkey: "recording started in 0 to 1 ms" in the log (target capture plus overlay event); the pill is a separate always-loaded window so nothing is created on the press.

Engine warm start: 1.75 s (typical) to 4.1 s (first start after a rebuild) from process spawn to the first answered request.

## Not measured yet

- CPU fallback latency (`--no-gpu`).
- Idle memory of the app (the engine process holds about 475 MB of system RAM plus the model in VRAM).
- P95 over a long session with a human voice; the statistics page computes it from history once there is data.
- Human Greek speech. Put private WAV files and a `manifest.json` under `eval/private/` (git-ignored) and rerun `eval/bench.py`.
