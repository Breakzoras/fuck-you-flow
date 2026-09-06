"""Does the Vulkan build actually accelerate?

Runs the Greek corpus through the engine we built with the Vulkan backend and
prints the same numbers as the CUDA and CPU runs, so the three can be compared.
Vulkan works on AMD, Intel and NVIDIA, so a good result here is the answer for
every machine without an NVIDIA card.

Run from the project root: python eval/vulkan_check.py
"""
import json
import os
import socket
import subprocess
import sys
import time

sys.path.insert(0, "eval")
import bench  # noqa: E402

SERVER = os.path.join(bench.ROOT, "vendor", "whisper-vulkan", "whisper-server.exe")
MODELS = ["large-v3-turbo-q5_0", "large-v3-q5_0"]


def start(model_file, port, gpu, threads=8):
    args = [SERVER, "-m", os.path.join(bench.MODELS_DIR, model_file), "--host", "127.0.0.1",
            "--port", str(port), "-t", str(threads), "-l", "auto", "--no-timestamps", "--suppress-nst",
            "--flash-attn", "--inference-path", "/inference"]
    if os.path.exists(bench.VAD):
        args += ["--vad", "--vad-model", bench.VAD, "--vad-threshold", "0.5"]
    if not gpu:
        args.append("--no-gpu")
    logf = open(os.path.join(bench.ROOT, "eval", "server-vulkan.log"), "a", encoding="utf-8", errors="replace")
    p = subprocess.Popen(args, cwd=os.path.dirname(SERVER), stdout=logf, stderr=logf, creationflags=0x08000000)
    t0 = time.time()
    while time.time() - t0 < 300:
        try:
            socket.create_connection(("127.0.0.1", port), timeout=0.3).close()
            return p, round((time.time() - t0) * 1000)
        except OSError:
            if p.poll() is not None:
                raise RuntimeError("server exited early, see eval/server-vulkan.log")
            time.sleep(0.2)
    raise RuntimeError("server did not start")


def main():
    if not os.path.exists(SERVER):
        raise SystemExit("build it first: scripts/build-vulkan.ps1")
    greek = [x for x in bench.load_items() if x["language"] == "el"]
    rows = []
    for model in MODELS:
        port = bench.free_port()
        p, warm = start(bench.FILES[model], port, True)
        try:
            bench.infer(port, greek[0]["path"], "el", None, 1)
            times, wers, ratios = [], [], []
            for it in greek:
                text, ms, _ = bench.infer(port, it["path"], "el", None, 3)
                audio_ms = os.path.getsize(it["path"]) / 32.0
                times.append(ms)
                ratios.append(ms / audio_ms)
                wers.append(bench.jiwer.wer(bench.norm(it["spoken"]), bench.norm(text)))
                rows.append(dict(backend="vulkan", model=model, id=it["id"], ms=ms, wer=wers[-1]))
            print(f"vulkan {model:22} warm {warm/1000:.1f} s | mean {sum(times)/len(times):6.0f} ms "
                  f"| {sum(ratios)/len(ratios):.2f} x realtime | WER {sum(wers)/len(wers):.3f}", flush=True)
        finally:
            p.terminate()
            p.wait(timeout=20)
    json.dump(rows, open("eval/vulkan-results.json", "w", encoding="utf-8"), ensure_ascii=False, indent=2)


main()
