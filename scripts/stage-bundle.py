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
import glob
import os
import shutil
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STAGE = os.path.join(ROOT, "src-tauri", "bundled")

# The Vulkan build runs on AMD, Intel and NVIDIA cards and is 54 MB; the CUDA
# build is 1.1 GB and only helps NVIDIA, so it stays out of the installer and
# the developer checkout keeps it for comparison.
#
# Since 0.9.20 the Windows engine loads its parts at runtime (whisper.cpp built
# with GGML_BACKEND_DL and GGML_CPU_ALL_VARIANTS, scripts/build-engine-dl.ps1):
# ggml-vulkan.dll is tried and skipped when the PC has no Vulkan driver, and the
# best ggml-cpu-<level>.dll is picked for the processor. Up to 0.9.19 ggml.dll
# needed vulkan-1.dll at load time and ggml-cpu.dll needed AVX2, so a PC without
# either could not start the engine at all (0xc0000135, report of 6 November 2026).
# The Visual C++ runtime goes in next to it: a clean Windows does not have it.
ENGINE = ["whisper-server.exe", "whisper.dll", "ggml.dll", "ggml-base.dll", "ggml-vulkan.dll",
          "msvcp140.dll", "vcruntime140.dll", "vcruntime140_1.dll", "vcomp140.dll"]
# Every CPU level the build made; names vary with the whisper.cpp version.
ENGINE_GLOBS = ["ggml-cpu-*.dll"]
# Linux: one statically linked whisper-server with the Vulkan backend inside,
# built from the same patched whisper.cpp tree. FYF_ENGINE_DIR points at it.
IS_LINUX = sys.platform.startswith("linux")
if IS_LINUX:
    ENGINE = ["whisper-server"]
    ENGINE_GLOBS = []


def engine_files(engine_src):
    """The fixed engine names plus every file the globs find in the source."""
    names = list(ENGINE)
    for pattern in ENGINE_GLOBS:
        found = sorted(os.path.basename(p) for p in glob.glob(os.path.join(engine_src, pattern)))
        if not found:
            raise SystemExit(f"missing: {os.path.join(engine_src, pattern)} (an engine built before 0.9.20?)")
        names += found
    return names


def drop_stale(folder, keep):
    """Removes files a previous staging left that this engine no longer has,
    such as the AVX2-only ggml-cpu.dll of 0.9.19, so the glob cannot ship them."""
    if not os.path.isdir(folder):
        return
    for n in os.listdir(folder):
        p = os.path.join(folder, n)
        if n not in keep and os.path.isfile(p):
            try:
                os.remove(p)
                print("removed stale", n)
            except OSError as err:
                raise SystemExit(f"cannot remove stale {p}: {err} (is the app running from here?)")
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
    engine_src = os.environ.get("FYF_ENGINE_DIR") or os.path.join(ROOT, "vendor", "whisper-linux" if IS_LINUX else "whisper-vulkan")
    engine = engine_files(engine_src)
    drop_stale(os.path.join(STAGE, "engine-vulkan"), set(engine))
    parts = [("engine-vulkan", engine, engine_src)]
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
