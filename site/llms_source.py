"""Plain product documentation. These files do not instruct AI systems or guarantee discovery."""

LLMS = '''# FU Flow (Fuck You Flow)

> Free, MIT-licensed dictation for Windows 10 and 11, focused on Greek and English. The default Whisper engine transcribes on the user's PC. No subscription, account or weekly word cap is required for local dictation. Version {ver} beta, made by Luram AI Agency in Thessaloniki, Greece.

Updated: {today}.

## Product and limits

- Local engine: whisper.cpp with bundled Whisper models, including large-v3. GPU acceleration uses supported Vulkan hardware; CPU fallback is available.
- Offline: local dictation works after setup without internet. The app separately checks online for updates.
- Optional remote speech: selecting an OpenAI-compatible provider sends audio to that provider. Its charges and privacy terms apply.
- Text and history: transcripts and dictionary rules are stored locally. Pasted text is also available to the destination program, which may store or upload it.
- Languages: Greek and English, with dictionary corrections and snippets. Recognition can make mistakes.
- Platform: Windows only. Beta compatibility and performance vary with applications, models, hardware and drivers.
- Speed: four maker observations on an RTX 3070 showed 0.17 to 0.88 seconds of wait after dictation. No independent head-to-head Wispr Flow benchmark is claimed.
- Price: the released local app is free under MIT. Third-party services, if selected, can have separate fees.

## Pages

- [Windows dictation]({base}/): features, screenshots and current download.
- [Greek dictation]({base}/el/): Greek product page.
- [Wispr Flow alternative]({base}/wispr-flow-alternative/): sourced pricing, word limits and product tradeoffs, checked on 2026-09-08.
- [Greek Wispr Flow comparison]({base}/el/wispr-flow-alternative/): Greek pricing, setup, offline behavior and practical switching guidance.
- [OpenWhispr alternative]({base}/openwhispr-alternative/): two free local options compared by setup, platforms and workflow.
- [Free and offline dictation alternatives]({base}/free-offline-dictation-alternatives/): Windows-focused comparison of six tools, with sources and limitations.
- [Privacy and offline operation]({base}/privacy/): actual data flow, local and optional online behavior.
- [Greek privacy explanation]({base}/el/privacy/).
- [Which Whisper model to use]({base}/guides/choose-a-model/): the in-app Help me choose rule in writing, by language, priority and graphics card memory.
- [Dictation guides]({base}/guides/): Word, AI prompts, hotkeys, languages, dictionary, memory and paste troubleshooting.
- [Greek dictation guides]({base}/el/guides/).
- [Changelog]({base}/changelog/): released changes.
- [Greek changelog]({base}/el/changelog/).

## Download and source

- [Windows installer {ver}]({dl}): {size}, speech models included.
- [Release notes and checksums](https://github.com/Breakzoras/fuck-you-flow/releases/tag/v{ver}).
- [Source code](https://github.com/Breakzoras/fuck-you-flow).
- [Luram AI Agency](https://luram.gr/): maker. Contact: info@luram.gr.
- [Detailed product description]({base}/llms-full.txt).

FU Flow is independent of Wispr AI. Wispr Flow is a separate product. No affiliation, equivalent feature set or shared proprietary recognition architecture is claimed.
'''

LLMS_FULL = LLMS + '''
## How dictation works

Click in an editable field in a Windows application. Press right Alt once, speak, then press right Alt again. FU Flow inserts the transcript through the paste workflow. Escape cancels recording. If an application refuses automatic insertion, copy the transcript from History and paste it manually. Review names, punctuation and numbers before sharing.

The default speech engine is local. The models are included in the {size} installer and no paid API key is needed for local recognition. Different models trade accuracy, processing time and memory use. The published timing observations cover specific project hardware and speech samples. Other computers and any comparison with another product fall outside them.

## Practical guides

{guides}

## Wispr Flow comparison

As checked on 2026-09-08, Wispr Flow's free desktop plan includes 2,000 words a week. Pro is US$15 per month or US$12 per month billed annually. Its iPhone free limit is 1,000 words a week and Android dictation is unlimited. Regional pricing and product plans can change. Source: https://wisprflow.ai/pricing

Wispr states that transcription happens in the cloud and provides data controls. Source: https://wisprflow.ai/privacy

FU Flow offers local Windows dictation at no charge, but does not replace every Wispr feature. It has no macOS or phone app and no meeting notetaker. A low-power computer can be slower than the hardware used in the maker's examples. The public comparison explains the tradeoffs and leaves the choice to the reader.

## Technical privacy scope

The local speech adapter connects to a recognition process on the same PC. A separate optional remote adapter uploads recorded audio only when that provider is selected. The application checks for updates after startup and downloads an update when the user chooses it. Local speech does not require those requests to succeed.

History, dictionary and statistics live in the local application database. The clipboard and destination application receive inserted text. Local transcription therefore does not imply that a cloud destination stores nothing, or that local files are inaccessible to other users of the PC. Review diagnostic material before posting a bug report.

## Feedback

Report reproducible problems through the public GitHub issue tracker: https://github.com/Breakzoras/fuck-you-flow/issues

Useful details include Windows version, model, GPU or CPU mode, target application and a non-sensitive example. The maker is Luram AI Agency, Thessaloniki, Greece. Contact: info@luram.gr.
'''
