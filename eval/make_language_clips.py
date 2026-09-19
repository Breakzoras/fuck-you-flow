"""Generate synthetic short speech for the ignored live_short_pieces test.

Requires edge-tts and ffmpeg. Run with --output PATH, then set FYF_SHORT_CLIPS
to PATH and start the local Whisper test server on port 47555.
"""
import argparse
import asyncio
import json
from pathlib import Path
import subprocess

import edge_tts

ITEMS = [
    ("el-chara", "el-GR-NestorasNeural", "Χαρά", "el"),
    ("el-nai", "el-GR-AthinaNeural", "Ναι", "el"),
    ("el-entaxi", "el-GR-NestorasNeural", "Εντάξει", "el"),
    ("el-gianni", "el-GR-AthinaNeural", "Γιάννη", "el"),
    ("en-thanks", "en-US-AriaNeural", "Thanks", "en"),
    ("en-ok", "en-GB-RyanNeural", "Okay", "en"),
]


async def main(root: Path):
    root.mkdir(parents=True, exist_ok=True)
    for name, voice, text, _ in ITEMS:
        mp3 = root / (name + ".mp3")
        await edge_tts.Communicate(text, voice).save(str(mp3))
        subprocess.run([
            "ffmpeg", "-y", "-loglevel", "error", "-i", str(mp3),
            "-ac", "1", "-ar", "16000", "-sample_fmt", "s16", str(root / (name + ".wav")),
        ], check=True)
        mp3.unlink()
    manifest = [{"id": n, "text": t, "language": lang} for n, _, t, lang in ITEMS]
    (root / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    print("Created", len(ITEMS), "synthetic short speech clips")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    asyncio.run(main(parser.parse_args().output))
