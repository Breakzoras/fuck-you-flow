"""Builds src-tauri/bundled/, the folder the installer ships verbatim.

Tauri's `resources` map writes every source file to the same target when the
target is given as a directory, so the engine and the models are staged into a
real folder first and shipped with a glob, which keeps the tree. Hard links are
used, so the 1.6 GB is not duplicated on disk.

Two shapes come out of here:

  python scripts/stage-bundle.py           engine and models, 1.6 GB
                                           the installer a new user downloads
  python scripts/stage-bundle.py --slim    engine only, 55 MB
                                           the update an installed user gets

The slim one is safe because the app moves the models out of the install folder
the first time it runs (models::migrate_bundled_models), and an update only
replaces the install folder.
"""
import os
import shutil
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STAGE = os.path.join(ROOT, "src-tauri", "bundled")

# The Vulkan build runs on AMD, Intel and NVIDIA cards and is 54 MB; the CUDA
# build is 1.1 GB and only helps NVIDIA, so it stays out of the installer and
# the developer checkout keeps it for comparison.
ENGINE = ["whisper-server.exe", "whisper.dll", "ggml.dll", "ggml-base.dll", "ggml-cpu.dll", "ggml-vulkan.dll"]
MODELS = ["ggml-large-v3-q5_0.bin", "ggml-large-v3-turbo-q5_0.bin", "ggml-silero-v5.1.2.bin"]


def same_file(a, b):
    """True when the two names point at one file on disk (an existing link)."""
    try:
        sa, sb = os.stat(a), os.stat(b)
    except OSError:
        return False
    return (sa.st_ino, sa.st_dev) == (sb.st_ino, sb.st_dev) and sa.st_ino != 0


def link(src, dst):
    # Already the same file: leave it. Deleting it would fail anyway while the
    # app is running, because the engine it launched holds the very same bytes.
    if same_file(src, dst):
        return
    if os.path.exists(dst):
        os.remove(dst)
    try:
        os.link(src, dst)
    except OSError:
        shutil.copy2(src, dst)


def main():
    slim = "--slim" in sys.argv
    total = 0
    parts = [("engine-vulkan", ENGINE, os.path.join(ROOT, "vendor", "whisper-vulkan"))]
    if slim:
        # Leave nothing behind from a previous full staging, or the glob would
        # ship the models anyway.
        shutil.rmtree(os.path.join(STAGE, "models"), ignore_errors=True)
    else:
        parts.append(("models", MODELS, os.path.join(ROOT, "models")))
    for sub, names, base in parts:
        d = os.path.join(STAGE, sub)
        os.makedirs(d, exist_ok=True)
        for n in names:
            src = os.path.join(base, n)
            if not os.path.exists(src):
                raise SystemExit(f"missing: {src}")
            link(src, os.path.join(d, n))
            total += os.path.getsize(src)
        print(sub, len(names), "files")
    print("staged %.2f GB in %s%s" % (total / 1073741824, STAGE, " (slim, no models)" if slim else ""))


if __name__ == "__main__":
    main()
