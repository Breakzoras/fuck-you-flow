"""How the models behave without a graphics card.

Runs the same Greek corpus through whisper-server with `--no-gpu`, for the model
shipped in the installer and for the accurate one, so a machine with an AMD or
an Intel card can be advised with numbers.

Run from the project root: python eval/cpu_check.py
"""
import json
import os
import socket
import subprocess
import sys
import time

sys.path.insert(0, "eval")
import bench  # noqa: E402

import itertools
MODELS = [("large-v3-turbo-q5_0", bench.FILES["large-v3-turbo-q5_0"])]
THREADS = [4, 8, 12, 16]


def start_cpu(model_file, port, threads=8):
    args = [bench.SERVER, "-m", os.path.join(bench.MODELS_DIR, model_file), "--host", "127.0.0.1",
            "--port", str(port), "-t", str(threads), "-l", "auto", "--no-timestamps", "--suppress-nst",
            "--flash-attn", "--inference-path", "/inference", "--no-gpu"]
    if os.path.exists(bench.VAD):
        args += ["--vad", "--vad-model", bench.VAD, "--vad-threshold", "0.5"]
    logf = open(os.path.join(bench.ROOT, "eval", "server-cpu.log"), "a", encoding="utf-8", errors="replace")
    p = subprocess.Popen(args, cwd=os.path.dirname(bench.SERVER), stdout=logf, stderr=logf, creationflags=0x08000000)
    t0 = time.time()
    while time.time() - t0 < 300:
        try:
            socket.create_connection(("127.0.0.1", port), timeout=0.3).close()
            return p, round((time.time() - t0) * 1000)
        except OSError:
            if p.poll() is not None:
                raise RuntimeError("server exited early, see eval/server-cpu.log")
            time.sleep(0.2)
    raise RuntimeError("server did not start")


def main():
    greek = [x for x in bench.load_items() if x["language"] == "el"]
    rows = []
    for (name, f), th in itertools.product(MODELS, THREADS):
        port = bench.free_port()
        p, warm = start_cpu(f, port, th)
        try:
            bench.infer(port, greek[0]["path"], "el", None, 1)  # warm-up
            times, wers, ratios = [], [], []
            for it in greek:
                text, ms, _ = bench.infer(port, it["path"], "el", None, 3)
                audio_ms = os.path.getsize(it["path"]) / 32.0  # 16 kHz, 16-bit mono
                times.append(ms)
                ratios.append(ms / audio_ms)
                wers.append(bench.jiwer.wer(bench.norm(it["spoken"]), bench.norm(text)))
                rows.append(dict(model=name, id=it["id"], ms=ms, audio_ms=round(audio_ms), wer=wers[-1]))
            print(f"{name:22} t={th:2}  warm {warm/1000:.1f} s | mean {sum(times)/len(times):6.0f} ms "
                  f"| {sum(ratios)/len(ratios):.2f} x realtime | WER {sum(wers)/len(wers):.3f}", flush=True)
        finally:
            p.terminate()
            p.wait(timeout=20)
    json.dump(rows, open("eval/cpu-results.json", "w", encoding="utf-8"), ensure_ascii=False, indent=2)


main()
