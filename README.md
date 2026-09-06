# Fuck You Flow

Dictation for Windows that runs entirely on your own machine. Hold a key, speak,
and the text lands wherever your cursor is: Word, Chrome, Slack, a terminal,
anywhere. Greek and English are first class, mixed in the same sentence too.

Nothing leaves the computer. No account, no subscription, no server, no
internet connection required after setup.

Free and open source, from [Luram AI Agency](https://luram.gr).

![Home](press/shots/1-home.png)

## Why it exists

Commercial dictation apps charge a monthly fee, send the audio to a server, and
treat Greek as an afterthought. The speech models that do Greek well have been
free and open for two years. This puts them behind a key on your keyboard.

## What you get

- **Right Alt starts, Right Alt stops.** Press, speak, press again. The text is
  inserted where the cursor was. Escape cancels.
- **Text appears in well under a second**, even after two minutes of speech: the
  engine transcribes each finished phrase while you are still talking, so only
  the last sentence is left when you stop.
- **Greek and English**, and both in the same sentence.
- **Cleanup that respects the meaning.** Filler words go, spoken self-corrections
  ("Τρίτη, όχι όχι, Παρασκευή") are applied, and question marks are added from
  the wording using a rule set built from Greek grammar sources.
- **A dictionary that learns.** Correct a word once in History and it becomes a
  rule; names and technical terms come out right from then on.
- **Snippets.** Say a phrase, get a block of text.
- **History, statistics and per-application styles**, all on your disk.
- **Nothing is sent anywhere.** A password field is detected and refused. An
  optional cloud provider exists and is off.

![Dictionary](press/shots/3-dictionary.png)

## Install

Download the installer from
[Releases](https://github.com/Breakzoras/fuck-you-flow/releases) and run it. It needs
no administrator rights and about 1.7 GB of disk.

Windows shows a blue "Windows protected your PC" dialog because the installer
carries no purchased signature. Click "More info", then "Run anyway".

The installer carries everything: the speech engine built with the Vulkan
backend (it runs on AMD, Intel and NVIDIA cards, and on the processor when there
is no card), both speech models, and the voice activity detector. The first
start reads the machine, picks the model that fits the graphics memory and the
thread count that fits the processor, and the Speech models tab in Settings
shows what it found.

## Speed

Measured on an RTX 3070 with `large-v3-q5_0`, from the moment the key is
released to the text appearing:

| Words | Speech | Wait |
|---:|---:|---:|
| 32 | 17.5 s | 0.59 s |
| 108 | 82 s | 0.17 s |
| 179 | 77 s | 0.53 s |
| 312 | 152 s | 0.88 s |

The wait depends on the length of the last phrase only. A short pause before
pressing the key again gives the fastest result.

Engine comparison on the same card and phrase (5 s of Greek, `large-v3-q5_0`):
Vulkan 593 ms, CUDA 663 ms, processor alone 12.5 s. Vulkan is what ships, so an
AMD or Intel card gets the same speed as an NVIDIA one; a machine with no card
at all falls back to the processor and the turbo model.

## Speaking well

Speed is fine: 68 to 158 words per minute all transcribed correctly in testing.
What helps most:

- A half-second pause at the end of each sentence. It gives correct punctuation
  and speed at the same time.
- Pronounce the last word of a sentence fully. Swallowed endings are the first
  thing that breaks Greek text.
- Names and foreign words: say them clearly and in one piece.
- A question with no question word gets its mark from the rise of your voice;
  say "ερωτηματικό" to force one.
- To fix yourself mid-sentence: "όχι όχι" and then the right word.

## Build it yourself

Requirements: Rust stable, Node 20 or newer, pnpm, the Visual Studio C++ build
tools. See [docs/DEV-SETUP.md](docs/DEV-SETUP.md).

```bash
pnpm install
pnpm tauri dev
```

Always build through the Tauri CLI. A plain `cargo build --release` produces a
development binary that looks for the interface on a dev server.

## Documentation

- [docs/USER-GUIDE.md](docs/USER-GUIDE.md) in Greek
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/THREAT-MODEL.md](docs/THREAT-MODEL.md)
- [docs/QUESTION-RULES.md](docs/QUESTION-RULES.md), the Greek grammar rules
- [docs/KNOWN-LIMITATIONS.md](docs/KNOWN-LIMITATIONS.md), read this before
  reporting a bug
- [docs/THIRD-PARTY-NOTICES.md](docs/THIRD-PARTY-NOTICES.md)

## Contributing

Issues and pull requests are welcome. The parts that most need other people's
machines: applications that refuse a paste, keyboard layouts, older processors,
and Greek words the dictionary should know.

## License

MIT. See [LICENSE](LICENSE). The speech engine and the models carry their own
licenses, listed in the third-party notices.

This is a clean-room implementation. It contains no code, assets or branding
from any other dictation product.
