"""Source-backed comparison pages, reviewed on 8 September 2026.

Keep product comparisons separate from release facts and bilingual workflows.
Competitor details are from the primary links included in the visible content.
"""
from site_data import DL, REPO, VERSION, SIZE


COMPARISONS = [
    {
        "path": "/openwhispr-alternative/",
        "lang": "en",
        "title": "OpenWhispr Alternative for Windows: Local Dictation",
        "description": "Compare FU Flow and OpenWhispr for free local dictation: Windows setup, Greek and English, model downloads, cloud cleanup and practical switching checks.",
        "lead": "OpenWhispr and FU Flow both offer free local dictation. The useful choice is the workflow you need: a Windows dictation beta focused on Greek and English, or a broader cross-platform tool.",
        "body": f'''<div class="cta"><a class="btn btn-acid" href="{DL}">Try FU Flow for Windows</a><a class="btn btn-coral" href="https://openwhispr.com/">Visit OpenWhispr</a></div>
<p>We make FU Flow at Luram AI Agency in Thessaloniki. This comparison uses our application source and OpenWhispr's official documentation, checked on 8 September 2026. It is not a report of hands-on competitor testing.</p>
<h2>What changes if you choose FU Flow?</h2>
<div class="tbl"><table class="bill"><thead><tr><th scope="col">Decision</th><th scope="col">FU Flow</th><th scope="col">OpenWhispr</th></tr></thead><tbody>
<tr><td>Desktop systems</td><td>Windows 10 and 11</td><td>Windows, macOS and Linux</td></tr>
<tr><td>Free local dictation</td><td>Unlimited, MIT licensed</td><td>Unlimited, MIT licensed</td></tr>
<tr><td>Speech models</td><td>Whisper models included in the {SIZE} installer</td><td>Whisper and Parakeet models selected and downloaded during setup</td></tr>
<tr><td>Main workflow</td><td>Greek and English dictation, dictionary, snippets and local history</td><td>Dictation plus meetings, notes and agent workflows</td></tr>
<tr><td>Optional cloud use</td><td>Your configured speech API, with provider fees</td><td>Managed cloud plans or your own API keys</td></tr>
</tbody></table></div>
<p>OpenWhispr's <a href="https://github.com/OpenWhispr/openwhispr">official repository</a> describes its supported systems, engines and wider feature set. FU Flow {VERSION} is a beta with a narrower scope. Free software and local processing are shared features, so neither alone is a reason to switch.</p>
<h2>Free local use and paid cloud use are different choices</h2>
<p>OpenWhispr's local dictation has no word limit. Its free managed cloud tier includes 2,000 words per week. Pro is listed at US$8 monthly or US$80 annually; bringing your own API key instead means the provider can charge for usage. See the <a href="https://openwhispr.com/pricing">pricing page</a> and <a href="https://docs.openwhispr.com/faq">official FAQ</a> for the current terms.</p>
<p>FU Flow has no paid local tier, account requirement or weekly word cap. Its bundled models let you begin without obtaining an API key. An optional OpenAI-compatible speech provider changes that arrangement: the endpoint receives audio and may charge you. Keeping the local engine selected avoids a speech API bill.</p>
<h2>Check cleanup as well as speech recognition</h2>
<p>For either app, identify which stages run locally. OpenWhispr's <a href="https://openwhispr.com/use-cases/developers">developer FAQ</a> says local speech recognition keeps audio on the computer. It also says signing in defaults cleanup to OpenWhispr Cloud unless you point it at a local model. Local recognition therefore does not automatically establish that every subsequent processing step is offline.</p>
<p>FU Flow defaults to its local Whisper engine and applies dictionary and snippet rules locally. It also checks online for updates. A remote speech provider uploads audio when selected, and a destination such as a cloud document or chat service can receive the inserted text. Our <a href="/privacy/">data-flow explanation</a> covers those boundaries.</p>
<h2>Run a useful Greek and English comparison</h2>
<ol><li>Use the same microphone and a short passage you actually write. Include a Greek sentence, an English product name and a number. Keep the original passage for checking.</li><li>Finish any model downloads, select the intended language and confirm local processing settings. Give the apps different hotkeys if both are running.</li><li>Dictate into the same editable text field. Count meaningful corrections and note the wait after you stop speaking. Repeat with your usual background noise.</li><li>Test insertion in Word or your browser. Add recurring names to Dictionary and check the resulting text before sending it.</li></ol>
<p>Use the <a href="/guides/greek-and-english/">Greek and English guide</a>, <a href="/guides/word-dictation/">Word workflow</a> and <a href="/guides/words-it-gets-wrong/">dictionary guide</a> to make that trial repeatable. Local model speed depends on the model and computer; we have no controlled FU Flow versus OpenWhispr accuracy or speed result.</p>
<p>Spoken punctuation is one useful detail to include. FU Flow's <a href="{REPO}/blob/main/src-tauri/src/cleanup/questions.rs">documented cleanup examples</a> convert <span lang="el">«Είσαι σίγουρος ερωτηματικό Πάμε.»</span> into <span lang="el">«Είσαι σίγουρος; Πάμε.»</span>, and “Are you sure question mark” into “Are you sure?”. These illustrate text-cleanup rules, not guaranteed microphone recognition. Test whether your own speech produces the expected words and punctuation.</p>
<h2>When to keep OpenWhispr, and when to try FU Flow</h2>
<p>Keep OpenWhispr on your shortlist if you need macOS or Linux, prefer its model choices, or use its meeting, note and agent features. Try FU Flow if you want a Windows beta with bundled models and documented Greek and English dictation workflows. Test the everyday applications you depend on before replacing a working setup.</p>
<p>Read the <a href="{REPO}">FU Flow source and release notes</a>, compare <a href="/wispr-flow-alternative/">Wispr Flow separately</a>, or explore the wider <a href="/free-offline-dictation-alternatives/">offline dictation alternatives</a>.</p>''',
    },
    {
        "path": "/free-offline-dictation-alternatives/",
        "lang": "en",
        "title": "Best Free Dictation Software for Windows (2026)",
        "description": "The best free dictation and voice to text software for Windows, compared: FU Flow, OpenWhispr, Handy, Wispr Flow, Superwhisper and VoiceInk, offline use and limits.",
        "lead": "For free offline dictation on Windows, start with FU Flow, OpenWhispr and Handy. Wispr Flow, Superwhisper and VoiceInk belong in the comparison too, with different pricing, processing or platform requirements.",
        "body": f'''<p>This guide is published by Luram AI Agency, the maker of FU Flow. We checked the linked official product documentation on 8 September 2026. These are documented capabilities and purchase conditions, not independent test scores.</p>
<h2>Which alternatives actually fit Windows and offline use?</h2>
<div class="tbl"><table class="bill"><thead><tr><th scope="col">Product</th><th scope="col">Windows</th><th scope="col">Local speech recognition</th><th scope="col">Cost distinction</th></tr></thead><tbody>
<tr><td><a href="/">FU Flow</a></td><td>Windows 10 and 11 beta</td><td>Default, bundled Whisper models</td><td>Free local dictation, no word cap</td></tr>
<tr><td><a href="https://openwhispr.com/pricing">OpenWhispr</a></td><td>Yes, also macOS and Linux</td><td>Local model option</td><td>Unlimited free local use; separate cloud plans</td></tr>
<tr><td><a href="https://handy.computer/">Handy</a></td><td>Yes, also macOS and Linux</td><td>Offline models</td><td>Free, MIT licensed</td></tr>
<tr><td><a href="https://wisprflow.ai/pricing">Wispr Flow</a></td><td>Yes</td><td>Cloud transcription</td><td>2,000 free desktop words weekly; paid unlimited plan</td></tr>
<tr><td><a href="https://superwhisper.com/docs/get-started/sw-pro">Superwhisper</a></td><td>Yes</td><td>Local voice models in Pro</td><td>Free basic tier; paid Pro for local models</td></tr>
<tr><td><a href="https://tryvoiceink.com/">VoiceInk by Pax</a></td><td>No Windows release listed</td><td>Local default on its Mac product</td><td>Mac licenses sold as a one-time purchase</td></tr>
</tbody></table></div>
<h2>Three free local options to try on Windows</h2>
<p><strong>FU Flow</strong> focuses on Greek and English dictation, with a local dictionary, snippets and transcript history. Version {VERSION} is a Windows beta; the {SIZE} installer includes models. Local use needs no account or API key. It suits people willing to test a beta in their own applications. Start with <a href="/guides/word-dictation/">dictation in Word</a> or <a href="/guides/ai-prompts/">speaking an AI prompt</a>.</p>
<p>For a Greek and English trial, say <span lang="el">«Στείλε το brief στο Slack για review.»</span> into a blank document and check the Greek text and English names. This is an example to try, not a claimed test result. Add a dictionary correction if a recurring name is misheard.</p>
<p><strong>OpenWhispr</strong> offers free unlimited local dictation and a broader cross-platform app with meetings, notes and agent features. It supports local Whisper and Parakeet engines as well as cloud services. Download the models you want and check both transcription and cleanup settings. Our <a href="/openwhispr-alternative/">FU Flow versus OpenWhispr comparison</a> explains the separate local and cloud choices.</p>
<p><strong>Handy</strong> is another free, MIT-licensed desktop app, with Windows, macOS and Linux builds. Its documented workflow is to press a shortcut, speak and put the transcription into the active text field. The <a href="https://github.com/cjpais/Handy">official repository</a> lists Whisper and Parakeet model options. It belongs on the shortlist if you want an open-source offline tool without requiring FU Flow's particular interface or language workflow.</p>
<h2>Options with a different tradeoff</h2>
<p><strong>Wispr Flow</strong> is relevant when comparing the familiar dictation workflow, but it is not an offline alternative. Its <a href="https://wisprflow.ai/privacy">privacy documentation</a> says transcription happens in the cloud and describes retention and training controls. The free desktop allowance is 2,000 words per week. Read our <a href="/wispr-flow-alternative/">Windows pricing and privacy comparison</a> before deciding whether a local tool addresses your reason for switching.</p>
<p><strong>Superwhisper</strong> supports Windows and offers local processing, but its <a href="https://superwhisper.com/docs/get-started/sw-pro">plan comparison</a> places local voice models in Pro. Pro lists US$8.49 monthly, US$84.99 annually or US$249.99 lifetime. That makes it an offline option to budget for, rather than a free-local equivalent. Its <a href="https://superwhisper.com/docs/get-started/windows">Windows documentation</a> also identifies feature differences from the Mac version.</p>
<p><strong>VoiceInk</strong> here means the product at tryvoiceink.com, which links to <a href="https://github.com/Beingpax/VoiceInk">Beingpax/VoiceInk</a>. Its Mac release requires Apple Silicon and macOS 14.4 or newer; the site also links an iOS app. It is relevant if you use Apple devices, not as a Windows download. Its <a href="https://tryvoiceink.com/privacy">privacy policy</a> distinguishes local default processing from optional cloud transcription and enhancement.</p>
<h2>A practical way to choose</h2>
<ol><li>Confirm your operating system and whether offline recognition is included in the plan you will use. Free installation is not the same as unlimited free local processing.</li><li>Download the required models first. Check recognition, cleanup and enhancement settings separately, then test with the network disconnected if offline use is essential.</li><li>Use the same microphone, sentences and target application. Include your actual names and technical terms. Compare corrections and waiting time on your own hardware.</li><li>Check hotkey conflicts, insertion reliability and transcript history. A good transcription is useful only if it reaches the intended document.</li></ol>
<p>Local dictation can keep speech processing on your computer while the receiving application still syncs text online. Read the selected app's settings and privacy information. For FU Flow, see the <a href="/privacy/">data flow</a> and <a href="/guides/when-the-paste-refuses/">insertion troubleshooting</a>, then <a href="{DL}">try the free Windows beta</a>.</p>''',
    },
]
