"""Build the repeatable evaluation corpus with synthetic Greek and English speech.

Voices come from Microsoft Edge neural TTS (edge-tts). This is NOT a human
corpus: it measures the pipeline (recognition, dictionary, cleanup, timing) in a
repeatable way. Private human recordings go in eval/private/ (git-ignored).

Usage:  python eval/make_corpus.py
Output: eval/corpus/*.wav (16 kHz mono) + eval/corpus/manifest.json
"""
import asyncio
import json
import os
import subprocess
import sys

import edge_tts

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "corpus")
os.makedirs(OUT, exist_ok=True)

GREEK_VOICES = ["el-GR-NestorasNeural", "el-GR-AthinaNeural"]
ENGLISH_VOICES = ["en-GB-RyanNeural", "en-US-AriaNeural"]

# (id, language, spoken text, expected clean text, dictionary terms used)
ITEMS = [
    ("el-01", "el", "Καλημέρα, θα ήθελα να κλείσουμε ένα ραντεβού την Τρίτη στις τέσσερις το απόγευμα.",
     "Καλημέρα, θα ήθελα να κλείσουμε ένα ραντεβού την Τρίτη στις τέσσερις το απόγευμα.", []),
    ("el-02", "el", "Στείλε μου το αρχείο στο email μου, είναι το info παπάκι oneclickclaw τελεία io.",
     "Στείλε μου το αρχείο στο email μου, είναι το info@oneclickclaw.io.", []),
    ("el-03", "el", "Ο server του OpenClaw τρέχει στο Webdock και η βάση είναι PostgreSQL.",
     "Ο server του OpenClaw τρέχει στο Webdock και η βάση είναι PostgreSQL.", ["OpenClaw", "Webdock", "PostgreSQL"]),
    ("el-04", "el", "Θα πληρώσω τριακόσια πενήντα ευρώ στις δεκαπέντε Σεπτεμβρίου δύο χιλιάδες είκοσι έξι.",
     "Θα πληρώσω 350 ευρώ στις 15 Σεπτεμβρίου 2026.", []),
    ("el-05", "el", "Η εταιρεία μου λέγεται Luram και φτιάχνει αυτοματισμούς με τεχνητή νοημοσύνη για μικρές επιχειρήσεις.",
     "Η εταιρεία μου λέγεται Luram και φτιάχνει αυτοματισμούς με τεχνητή νοημοσύνη για μικρές επιχειρήσεις.", ["Luram"]),
    ("el-06", "el", "Άνοιξε το αρχείο settings τελεία json και άλλαξε το port σε οκτώ χιλιάδες ογδόντα.",
     "Άνοιξε το αρχείο settings.json και άλλαξε το port σε 8080.", []),
    ("el-07", "el", "Θα σε πάρω τηλέφωνο την Τετάρτη, όχι, την Πέμπτη το πρωί.",
     "Θα σε πάρω τηλέφωνο την Πέμπτη το πρωί.", []),
    ("el-08", "el", "Εεε, νομίζω ότι, εμ, πρέπει να ξαναδούμε το πλάνο για τα social media.",
     "Νομίζω ότι πρέπει να ξαναδούμε το πλάνο για τα social media.", []),
    ("el-09", "el", "Ο Γιάννης Παπαδόπουλος και η Μαρία Κωνσταντίνου θα έρθουν στη Θεσσαλονίκη.",
     "Ο Γιάννης Παπαδόπουλος και η Μαρία Κωνσταντίνου θα έρθουν στη Θεσσαλονίκη.", []),
    ("el-10", "el", "Τι κάνεις; Είμαι καλά, ευχαριστώ. Εσύ πώς πας με τη δουλειά;",
     "Τι κάνεις; Είμαι καλά, ευχαριστώ. Εσύ πώς πας με τη δουλειά;", []),
    ("en-01", "en", "Please push the fix to the main branch and open a pull request before noon.",
     "Please push the fix to the main branch and open a pull request before noon.", []),
    ("en-02", "en", "The meeting is on Tuesday, no, Friday at three thirty.",
     "The meeting is on Friday at three thirty.", []),
    ("en-03", "en", "Um, I think we should, uh, deploy the new version tonight.",
     "I think we should deploy the new version tonight.", []),
    ("en-04", "en", "Send the invoice for one thousand two hundred euros to accounts at luram dot gr.",
     "Send the invoice for 1200 euros to accounts@luram.gr.", []),
    ("mix-01", "el", "Έκανα deploy το frontend στο Cloudflare και τώρα το dashboard φορτώνει σε μισό δευτερόλεπτο.",
     "Έκανα deploy το frontend στο Cloudflare και τώρα το dashboard φορτώνει σε μισό δευτερόλεπτο.", ["Cloudflare"]),
    ("mix-02", "el", "Πες στον Claude να γράψει ένα script σε Python που διαβάζει το CSV.",
     "Πες στον Claude να γράψει ένα script σε Python που διαβάζει το CSV.", ["Claude"]),
]


async def synth(text: str, voice: str, mp3: str):
    communicate = edge_tts.Communicate(text, voice, rate="+0%")
    await communicate.save(mp3)


def to_wav(mp3: str, wav: str):
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", mp3, "-ac", "1", "-ar", "16000", "-sample_fmt", "s16", wav], check=True)
    os.remove(mp3)


async def main():
    manifest = []
    for i, (id_, lang, spoken, expected, terms) in enumerate(ITEMS):
        voices = GREEK_VOICES if lang == "el" else ENGLISH_VOICES
        voice = voices[i % len(voices)]
        wav = os.path.join(OUT, f"{id_}.wav")
        if not os.path.exists(wav):
            mp3 = wav.replace(".wav", ".mp3")
            await synth(spoken, voice, mp3)
            to_wav(mp3, wav)
            print("made", id_, voice)
        manifest.append({"id": id_, "language": lang, "voice": voice, "spoken": spoken, "expected": expected, "terms": terms, "file": os.path.basename(wav)})
    # silence and noise for hallucination checks
    sil = os.path.join(OUT, "silence-3s.wav")
    if not os.path.exists(sil):
        subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-f", "lavfi", "-i", "anullsrc=r=16000:cl=mono", "-t", "3", "-sample_fmt", "s16", sil], check=True)
    noise = os.path.join(OUT, "noise-3s.wav")
    if not os.path.exists(noise):
        subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-f", "lavfi", "-i", "anoisesrc=r=16000:a=0.02:c=pink", "-t", "3", "-ac", "1", "-sample_fmt", "s16", noise], check=True)
    manifest.append({"id": "silence", "language": "none", "voice": "", "spoken": "", "expected": "", "terms": [], "file": "silence-3s.wav"})
    manifest.append({"id": "noise", "language": "none", "voice": "", "spoken": "", "expected": "", "terms": [], "file": "noise-3s.wav"})
    with open(os.path.join(OUT, "manifest.json"), "w", encoding="utf-8") as f:
        json.dump(manifest, f, ensure_ascii=False, indent=2)
    print("manifest:", len(manifest), "items")


if __name__ == "__main__":
    if sys.platform == "win32":
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())
    asyncio.run(main())
