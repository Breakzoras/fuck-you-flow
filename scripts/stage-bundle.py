"""Builds src-tauri/bundled/, the folder the installer ships verbatim.

Tauri's `resources` map writes every source file to the same target when the
target is given as a directory, so the engine and the models are staged into a
real folder first and shipped with a glob, which keeps the tree. Hard links are
used, so the 1.6 GB is not duplicated on disk.

Run from the project root: python scripts/stage-bundle.py
"""
import os
import shutil

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STAGE = os.path.join(ROOT, "src-tauri", "bundled")

# The Vulkan build runs on AMD, Intel and NVIDIA cards and is 54 MB; the CUDA
# build is 1.1 GB and only helps NVIDIA, so it stays out of the installer and
# the developer checkout keeps it for comparison.
ENGINE = ["whisper-server.exe", "whisper.dll", "ggml.dll", "ggml-base.dll", "ggml-cpu.dll", "ggml-vulkan.dll"]
MODELS = ["ggml-large-v3-q5_0.bin", "ggml-large-v3-turbo-q5_0.bin", "ggml-silero-v5.1.2.bin"]


def link(src, dst):
    if os.path.exists(dst):
        os.remove(dst)
    try:
        os.link(src, dst)
    except OSError:
        shutil.copy2(src, dst)


def main():
    total = 0
    for sub, names, base in (
        ("engine-vulkan", ENGINE, os.path.join(ROOT, "vendor", "whisper-vulkan")),
        ("models", MODELS, os.path.join(ROOT, "models")),
    ):
        d = os.path.join(STAGE, sub)
        os.makedirs(d, exist_ok=True)
        for n in names:
            src = os.path.join(base, n)
            if not os.path.exists(src):
                raise SystemExit(f"missing: {src}")
            link(src, os.path.join(d, n))
            total += os.path.getsize(src)
        print(sub, len(names), "files")
    print("staged %.2f GB in %s" % (total / 1073741824, STAGE))


if __name__ == "__main__":
    main()
