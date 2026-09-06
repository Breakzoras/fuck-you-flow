"""Can the pitch contour at the end of a sentence tell a Greek yes/no question
from a statement, without any question word?

Synthesises matched pairs with Edge TTS (the text with "?" gets question
intonation, the same text with "." gets statement intonation), then measures
the fundamental frequency (F0) over the last voiced stretch of each clip and
prints a few candidate features per clip plus the accuracy of a simple
threshold. Synthetic voices only: a green light here means "worth testing on
Lu's voice", never "ship it".

Run from the project root: python eval/prosody_check.py
"""
import asyncio
import json
import os
import subprocess
import sys

import numpy as np

sys.path.insert(0, "eval")
import edge_tts  # noqa: E402

OUT = os.path.join("eval", "prosody")
VOICES = ["el-GR-AthinaNeural", "el-GR-NestorasNeural"]
PAIRS = [
    "Θα πάρει πολύ ώρα ακόμα",
    "Είσαι σίγουρος",
    "Το έστειλες στον Γιώργο",
    "Έχεις χρόνο αύριο το πρωί",
    "Θα έρθεις και εσύ",
    "Δούλεψε τελικά η αλλαγή",
    "Μπορούμε να το κάνουμε σήμερα",
    "Είναι έτοιμο το άρθρο",
    "Το κατάλαβες",
    "Θέλεις καφέ",
]


async def synth(text, voice, mp3):
    await edge_tts.Communicate(text, voice, rate="+0%").save(mp3)


def to_wav(mp3, wav):
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", mp3, "-ac", "1", "-ar", "16000", "-f", "wav", wav], check=True)


def read_wav(path):
    import wave
    with wave.open(path, "rb") as w:
        n = w.getnframes()
        data = np.frombuffer(w.readframes(n), dtype=np.int16).astype(np.float32) / 32768.0
    return data


def f0_track(x, sr=16000, frame=0.04, hop=0.01, fmin=70, fmax=400):
    """Autocorrelation pitch per frame; 0 where unvoiced."""
    n = int(frame * sr)
    h = int(hop * sr)
    lag_min, lag_max = int(sr / fmax), int(sr / fmin)
    out = []
    for start in range(0, max(0, len(x) - n), h):
        seg = x[start:start + n]
        seg = seg - seg.mean()
        energy = float(np.sqrt(np.mean(seg * seg)))
        if energy < 0.01:
            out.append(0.0)
            continue
        ac = np.correlate(seg, seg, mode="full")[n - 1:]
        ac = ac / (ac[0] + 1e-9)
        window = ac[lag_min:lag_max]
        k = int(np.argmax(window)) + lag_min
        out.append(sr / k if window.max() > 0.3 else 0.0)
    return np.array(out)


def features(f0):
    voiced = f0[f0 > 0]
    if len(voiced) < 12:
        return None
    # last voiced stretch: trailing frames after the final unvoiced gap
    idx = np.where(f0 > 0)[0]
    last = idx[-1]
    tail = f0[max(0, last - 40):last + 1]
    tail = tail[tail > 0]
    if len(tail) < 6:
        return None
    semis = 12 * np.log2(tail / np.median(voiced))
    q = len(semis) // 3
    head_mean = float(np.mean(semis[:q])) if q else float(semis[0])
    peak = float(np.max(semis[-2 * q:])) if q else float(semis[-1])
    end = float(np.mean(semis[-3:]))
    return dict(peak_minus_head=peak - head_mean, end_minus_head=end - head_mean, peak=peak, end=end)


async def main():
    os.makedirs(OUT, exist_ok=True)
    rows = []
    for i, text in enumerate(PAIRS):
        voice = VOICES[i % len(VOICES)]
        for kind, mark in (("question", ";"), ("statement", ".")):
            id_ = f"{i:02d}-{kind}"
            mp3 = os.path.join(OUT, id_ + ".mp3")
            wav = os.path.join(OUT, id_ + ".wav")
            if not os.path.exists(wav):
                await synth(text + mark, voice, mp3)
                to_wav(mp3, wav)
            f = features(f0_track(read_wav(wav)))
            rows.append(dict(id=id_, kind=kind, voice=voice, text=text, **(f or {})))
            print(id_, voice, f, flush=True)
    json.dump(rows, open(os.path.join(OUT, "results.json"), "w", encoding="utf-8"), ensure_ascii=False, indent=2)
    # simple threshold sweep on the "peak of the final stretch minus its start"
    ok_rows = [r for r in rows if "peak_minus_head" in r]
    best = None
    for thr in np.arange(-2, 12, 0.5):
        correct = sum(1 for r in ok_rows if (r["peak_minus_head"] >= thr) == (r["kind"] == "question"))
        if best is None or correct > best[1]:
            best = (float(thr), correct)
    print(f"best threshold {best[0]} semitones: {best[1]}/{len(ok_rows)} correct")


asyncio.run(main())
