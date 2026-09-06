# Backlog: deferred on purpose

Ordered by how much daily value each item adds for a single Greek and English speaker on one Windows PC.

## Next (after the MVP is stable)

1. **Local AI cleanup provider.** `llama-server` from llama.cpp as a second sidecar, Llama-Krikri-8B-Instruct q4_k_m (Greek instruction following 67.5 percent IFEval) or Qwen3 4B, loaded sequentially after whisper releases the GPU or on the CPU. Already designed: `CleanupProvider` slot in `cleanup/mod.rs`, divergence check in `cleanup/divergence.rs` rejects rewrites that change numbers, language or meaning. Needs a VRAM budget measurement first (1.1 GB whisper + about 5 GB for an 8B q4 model is tight on 8 GB).
2. **Spoken punctuation and commands in Greek and English** ("νέα γραμμή", "τελεία", "new paragraph", "delete that"). Deterministic, small table, unit tested.
3. **Number and date normalisation** ("τριακόσια πενήντα" to "350", "δεκαπέντε Σεπτεμβρίου" to "15 Σεπτεμβρίου") as an option per application style. Whisper already writes digits most of the time; the rule closes the gap.
4. **Per-application paste chord table** (Windows Terminal needs Ctrl+Shift+V, some apps Shift+Insert). The terminal case is handled; a user-editable table is the general fix.
5. **Overlay drag to a custom position** with the position remembered per monitor.
6. **Sounds** on start, stop and error (setting exists, no sounds shipped yet).
7. **Learning from edits made in the target application.** Today learning uses edits made inside the Lalia history editor only. Reading the edited text back from the target would need Context Awareness on and UIA text patterns, which vary per app.
8. **Retention sweep while running** (today retention is applied at startup only).
9. **Signed installer and Tauri updater** with signed manifests. Unsigned updates stay disabled on purpose.
10. **Parakeet TDT 0.6B v3 as a second local engine.** The whisper.cpp release already ships `parakeet-cli.exe`; the model supports 25 European languages including Greek and is several times faster than Whisper. Needs an HTTP or stdin wrapper and a Greek accuracy measurement on the corpus.

## Later

- Model Manager: pause and resume downloads, mirror URL fallback.
- Per-application Dictionary scope (the column exists in the database, the UI does not expose it yet).
- Import Wispr Flow dictionary exports.
- Whisper-style "whisper mode" gain boost for quiet speech.
- Voice command mode (select, delete, replace last sentence).
- Meeting or long-form recording mode with speaker turns.
- Streaming partial results in the pill (whisper.cpp streaming is experimental and hurts Greek accuracy).
- Personalised adapters trained on opt-in correction pairs, behind a versioned, reversible switch and a held-out test set.

## Explicitly out of scope

Mobile apps, team administration, social sharing, enterprise sign-on, cloud sync, training a new foundation model.
