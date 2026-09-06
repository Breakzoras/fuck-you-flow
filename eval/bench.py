"""Benchmark harness: speech models x corpus -> WER/CER, term accuracy, hallucination, latency.

Runs whisper-server (the same binary and flags the app uses) once per model,
replays the corpus over HTTP exactly like the app does, and writes
eval/bench-results.json plus a Markdown table to docs/BENCHMARKS.md.

Usage: python eval/bench.py [--models large-v3-q5_0,large-v3-turbo-q5_0,medium-q5_0] [--gpu|--cpu] [--beam 5]
"""
import argparse
import json
import os
import re
import socket
import subprocess
import sys
import time
import unicodedata

import requests

try:
    import jiwer
except ImportError:
    print("pip install jiwer requests")
    sys.exit(1)

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CORPUS = os.path.join(ROOT, "eval", "corpus")
PRIVATE = os.path.join(ROOT, "eval", "private")
SERVER = os.path.join(ROOT, "vendor", "whisper", "Release", "whisper-server.exe")
MODELS_DIR = os.path.join(ROOT, "models")
VAD = os.path.join(MODELS_DIR, "ggml-silero-v5.1.2.bin")
FILES = {
    "large-v3-q5_0": "ggml-large-v3-q5_0.bin",
    "large-v3-turbo-q5_0": "ggml-large-v3-turbo-q5_0.bin",
    "large-v3-turbo-q8_0": "ggml-large-v3-turbo-q8_0.bin",
    "medium-q5_0": "ggml-medium-q5_0.bin",
}


def norm(text: str) -> str:
    t = unicodedata.normalize("NFC", text).lower()
    t = re.sub(r"[^\w\s@.]", " ", t)
    t = re.sub(r"\s+", " ", t).strip()
    return t


def free_port():
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    p = s.getsockname()[1]
    s.close()
    return p


def start_server(model_file, gpu, port, threads=8):
    args = [SERVER, "-m", os.path.join(MODELS_DIR, model_file), "--host", "127.0.0.1", "--port", str(port), "-t", str(threads), "-l", "auto",
            "--no-timestamps", "--suppress-nst", "--flash-attn", "--inference-path", "/inference"]
    if os.path.exists(VAD):
        args += ["--vad", "--vad-model", VAD, "--vad-threshold", "0.5"]
    if not gpu:
        args.append("--no-gpu")
    # stderr must go to a file: whisper-server prints per-request diagnostics and blocks when a pipe fills up
    logf = open(os.path.join(ROOT, "eval", f"server-{os.path.basename(model_file)}.log"), "w", encoding="utf-8", errors="replace")
    p = subprocess.Popen(args, cwd=os.path.dirname(SERVER), stdout=logf, stderr=logf, creationflags=0x08000000)
    t0 = time.time()
    while time.time() - t0 < 180:
        try:
            socket.create_connection(("127.0.0.1", port), timeout=0.3).close()
            return p, round((time.time() - t0) * 1000)
        except OSError:
            if p.poll() is not None:
                raise RuntimeError("server exited early, see " + logf.name)
            time.sleep(0.2)
    raise RuntimeError("server did not start")


def infer(port, wav_path, language, prompt, beam, vad=True):
    with open(wav_path, "rb") as f:
        # plain json: verbose_json makes whisper-server crash on audio without speech when VAD is on
        data = {"response_format": "json", "temperature": "0.0", "temperature_inc": "0.2", "no_timestamps": "true",
                "suppress_non_speech": "true", "language": language, "beam_size": str(beam), "vad": "true" if vad else "false"}
        if prompt:
            data["prompt"] = prompt
        t0 = time.perf_counter()
        r = requests.post(f"http://127.0.0.1:{port}/inference", files={"file": ("audio.wav", f, "audio/wav")}, data=data, timeout=60)
        ms = round((time.perf_counter() - t0) * 1000)
    r.raise_for_status()
    j = r.json()
    text = j.get("text") or "".join(s.get("text", "") for s in j.get("segments", []))
    return text.strip(), ms, j.get("detected_language") or j.get("language")


def load_items():
    items = json.load(open(os.path.join(CORPUS, "manifest.json"), encoding="utf-8"))
    for it in items:
        it["path"] = os.path.join(CORPUS, it["file"])
    pm = os.path.join(PRIVATE, "manifest.json")
    if os.path.exists(pm):
        for it in json.load(open(pm, encoding="utf-8")):
            it["path"] = os.path.join(PRIVATE, it["file"])
            it["private"] = True
            items.append(it)
    return items


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--models", default="large-v3-q5_0,large-v3-turbo-q5_0,medium-q5_0")
    ap.add_argument("--cpu", action="store_true")
    ap.add_argument("--beam", type=int, default=5)
    ap.add_argument("--runs", type=int, default=1)
    a = ap.parse_args()
    items = load_items()
    all_terms = sorted({t for it in items for t in it.get("terms", [])})
    prompt_el = "Λεξιλόγιο: " + ", ".join(all_terms) + "." if all_terms else None
    results = {}
    for model in a.models.split(","):
        mf = FILES.get(model)
        if not mf or not os.path.exists(os.path.join(MODELS_DIR, mf)):
            print("skip", model, "(not downloaded)")
            continue
        port = free_port()
        print(f"== {model} ({'CPU' if a.cpu else 'GPU'})")
        proc, start_ms = start_server(mf, not a.cpu, port)
        try:
            # warm-up
            infer(port, os.path.join(CORPUS, "silence-3s.wav"), "en", None, 1, vad=False)
            per = []
            for it in items:
                lang = {"el": "el", "en": "en"}.get(it["language"], "auto")
                prompt = prompt_el if it.get("terms") or it["language"] == "el" else None
                best = None
                for _ in range(a.runs):
                    try:
                        # silence/noise: VAD off on purpose, so this measures the raw model (the app has two guards before it)
                        text, ms, det = infer(port, it["path"], lang if it["language"] != "none" else "auto", prompt, a.beam, vad=(it["language"] != "none"))
                    except Exception as e:  # server died or timed out: restart it and record the failure
                        print(f"  {it['id']:8} ERROR {type(e).__name__}; restarting server")
                        proc.kill()
                        port = free_port()
                        proc, _ = start_server(mf, not a.cpu, port)
                        text, ms, det = "<error: %s>" % type(e).__name__, 120000, None
                    if best is None or ms < best[1]:
                        best = (text, ms, det)
                text, ms, det = best
                ref = norm(it["spoken"]) if it["spoken"] else ""
                hyp = norm(text)
                wer = jiwer.wer(ref, hyp) if ref and hyp else (0.0 if not ref and not hyp else 1.0)
                cer = jiwer.cer(ref, hyp) if ref and hyp else (0.0 if not ref and not hyp else 1.0)
                terms_ok = [t for t in it.get("terms", []) if t.lower() in text.lower()]
                audio_s = max(0.1, (os.path.getsize(it["path"]) - 44) / 32000.0)
                per.append({"id": it["id"], "language": it["language"], "hyp": text, "ref": it["spoken"], "wer": round(wer, 3), "cer": round(cer, 3),
                            "terms": it.get("terms", []), "terms_ok": terms_ok, "ms": ms, "rtf": round(ms / 1000.0 / audio_s, 3), "detected": det,
                            "hallucinated": it["language"] == "none" and bool(hyp)})
                print(f"  {it['id']:8} wer={wer:.2f} cer={cer:.2f} {ms:5d} ms  {text[:70]!r}")
            def agg(lang):
                rows = [p for p in per if p["language"] == lang]
                return {"n": len(rows), "wer": round(sum(r["wer"] for r in rows) / max(1, len(rows)), 3), "cer": round(sum(r["cer"] for r in rows) / max(1, len(rows)), 3),
                        "p50_ms": sorted(r["ms"] for r in rows)[len(rows) // 2] if rows else 0, "max_ms": max((r["ms"] for r in rows), default=0)}
            terms_total = sum(len(p["terms"]) for p in per)
            terms_hit = sum(len(p["terms_ok"]) for p in per)
            results[model] = {"gpu": not a.cpu, "start_ms": start_ms, "el": agg("el"), "en": agg("en"), "hallucinations": sum(1 for p in per if p["hallucinated"]),
                              "term_accuracy": round(terms_hit / terms_total, 3) if terms_total else None, "items": per}
        finally:
            proc.kill()
    out = os.path.join(ROOT, "eval", "bench-results.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump({"date": time.strftime("%Y-%m-%d %H:%M"), "beam": a.beam, "results": results}, f, ensure_ascii=False, indent=2)
    # markdown summary
    lines = ["| Model | GPU | Load ms | Greek WER | Greek CER | English WER | Term acc. | Halluc. (silence+noise) | P50 ms | Max ms |", "|---|---|---|---|---|---|---|---|---|---|"]
    for m, r in results.items():
        lines.append(f"| {m} | {'yes' if r['gpu'] else 'no'} | {r['start_ms']} | {r['el']['wer']:.3f} | {r['el']['cer']:.3f} | {r['en']['wer']:.3f} | {r['term_accuracy']} | {r['hallucinations']}/2 | {r['el']['p50_ms']} | {max(r['el']['max_ms'], r['en']['max_ms'])} |")
    print("\n".join(lines))
    with open(os.path.join(ROOT, "eval", "bench-summary.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
