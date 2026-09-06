"""Latency levers, measured on the synthetic corpus.

Same model (large-v3-q5_0, GPU), several decoder settings and audio-context sizes.
Prints mean WER and mean request time per configuration and writes every row to
eval/latency-results.json. Run from the project root: python eval/latency_check.py

The corpus is synthetic (Edge TTS), so treat WER differences below a few points as
noise and confirm any change on real dictations before shipping it as a default.
"""
import json
import os
import subprocess
import sys
import time

sys.path.insert(0, "eval")
import bench  # noqa: E402

MODEL = bench.FILES["large-v3-q5_0"]
# (label, extra server args)
SERVERS = [
    ("ctx-full", []),
    ("ctx-1024", ["-ac", "1024"]),
    ("ctx-768", ["-ac", "768"]),
]
# (language, beam)
DECODERS = [("auto", 5), ("el", 5), ("el", 3), ("el", 1)]


def start(extra, port):
    """bench.start_server with extra whisper-server arguments appended."""
    args = [bench.SERVER, "-m", os.path.join(bench.MODELS_DIR, MODEL), "--host", "127.0.0.1", "--port", str(port),
            "-t", "8", "-l", "auto", "--no-timestamps", "--suppress-nst", "--flash-attn", "--inference-path", "/inference"]
    if os.path.exists(bench.VAD):
        args += ["--vad", "--vad-model", bench.VAD, "--vad-threshold", "0.5"]
    args += extra
    logf = open(os.path.join(bench.ROOT, "eval", "server-latency.log"), "a", encoding="utf-8", errors="replace")
    p = subprocess.Popen(args, cwd=os.path.dirname(bench.SERVER), stdout=logf, stderr=logf, creationflags=0x08000000)
    t0 = time.time()
    while time.time() - t0 < 180:
        try:
            import socket
            socket.create_connection(("127.0.0.1", port), timeout=0.3).close()
            return p
        except OSError:
            if p.poll() is not None:
                raise RuntimeError("server exited early, see eval/server-latency.log")
            time.sleep(0.2)
    raise RuntimeError("server did not start")


def main():
    items = bench.load_items()
    greek = [x for x in items if x["language"] == "el"]
    english = [x for x in items if x["language"] == "en"]
    rows = []
    for label, extra in SERVERS:
        port = bench.free_port()
        p = start(extra, port)
        try:
            # warm-up so GPU clocks and caches do not land on the first measurement
            bench.infer(port, greek[0]["path"], "el", None, 1)
            for lang, beam in DECODERS:
                for it in greek:
                    text, ms, _ = bench.infer(port, it["path"], lang, None, beam)
                    rows.append(dict(server=label, id=it["id"], corpus="el", language=lang, beam=beam, text=text, ms=ms,
                                     wer=bench.jiwer.wer(bench.norm(it["spoken"]), bench.norm(text))))
                g = [r for r in rows if r["server"] == label and r["corpus"] == "el" and r["language"] == lang and r["beam"] == beam]
                print(f"{label:9} el-corpus lang={lang:4} beam={beam}  WER {sum(r['wer'] for r in g)/len(g):.3f}  mean {sum(r['ms'] for r in g)/len(g):.0f} ms", flush=True)
            # English sentences in auto mode: a Greek-only default must not break them
            for it in english:
                text, ms, _ = bench.infer(port, it["path"], "auto", None, 5)
                rows.append(dict(server=label, id=it["id"], corpus="en", language="auto", beam=5, text=text, ms=ms,
                                 wer=bench.jiwer.wer(bench.norm(it["spoken"]), bench.norm(text))))
            g = [r for r in rows if r["server"] == label and r["corpus"] == "en"]
            print(f"{label:9} en-corpus lang=auto beam=5  WER {sum(r['wer'] for r in g)/len(g):.3f}  mean {sum(r['ms'] for r in g)/len(g):.0f} ms", flush=True)
        finally:
            p.terminate()
            p.wait(timeout=15)
    with open("eval/latency-results.json", "w", encoding="utf-8") as f:
        json.dump(rows, f, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    main()
