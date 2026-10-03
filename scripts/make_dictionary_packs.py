"""Builds the downloadable dictionaries from dictionaries/source.json.

One file per language that has corrections of its own (`el.json` holds the
Greek ones plus the ones for every language) and `all.json` for everyone
else, plus `index.json`, which the app reads to see what is offered. A file
whose rules changed gets the next version number; an unchanged one keeps its
number, so the app does not fetch it again.

After running this, sign every pack with the update key (the app refuses an
unsigned or changed file):

    TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/fuckyouflow.key)" TAURI_SIGNING_PRIVATE_KEY_PASSWORD="" \
      pnpm tauri signer sign dictionaries/el.json
    (the same for every pack file)

    python scripts/make_dictionary_packs.py
"""

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / "dictionaries"
FORMAT = 1


def load(path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        return None


def main():
    rules = load(ROOT / "source.json")["rules"]
    for r in rules:
        if not r["wrong"].strip() or not r["correct"].strip() or any(c.isspace() for c in r["wrong"].strip()):
            sys.exit(f"bad rule: {r}")
    seen = set()
    for r in rules:
        key = r["wrong"].strip().lower()
        if key in seen:
            sys.exit(f"twice in the list: {r}")
        seen.add(key)
    everyone = [r for r in rules if r["for"] == "all"]
    languages = sorted({r["for"] for r in rules if r["for"] != "all"})
    packs = {"all": everyone}
    for lang in languages:
        packs[lang] = [r for r in rules if r["for"] == lang] + everyone
    old_index = load(ROOT / "index.json") or {"packs": {}}
    index = {"format": FORMAT, "packs": {}}
    for lang, items in packs.items():
        body = sorted(({"wrong": r["wrong"].strip(), "correct": r["correct"].strip()} for r in items), key=lambda r: r["wrong"].lower())
        old = load(ROOT / f"{lang}.json")
        version = old["version"] if old and old["rules"] == body else old_index["packs"].get(lang, {}).get("version", 0) + 1
        pack = {"format": FORMAT, "language": lang, "version": version, "rules": body}
        (ROOT / f"{lang}.json").write_text(json.dumps(pack, ensure_ascii=False, indent=1) + "\n", encoding="utf-8", newline="\n")
        index["packs"][lang] = {"version": version, "rules": len(body)}
        print(f"{lang}: version {version}, {len(body)} rules")
    (ROOT / "index.json").write_text(json.dumps(index, indent=1) + "\n", encoding="utf-8", newline="\n")


if __name__ == "__main__":
    main()
