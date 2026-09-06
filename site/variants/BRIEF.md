# fuckyouflow.app front page: shared brief for the five variants

You are building ONE self-contained HTML file: `C:\Claude Projects\lalia\site\variants\vN.html`
(N is given in your task). It must open by double-click from disk (file://), with no build step,
no external CDN, no web fonts. All CSS and JS inline in the file. Images only from `../assets/`.

## What the product is
Fuck You Flow is a free, open-source (MIT) dictation app for Windows 10 and 11. Press the right
Alt key, speak Greek or English or both in one sentence, press right Alt again, the text lands
where the cursor is, with punctuation. Everything runs on the user's own PC (whisper.cpp, GPU
acceleration on NVIDIA, AMD and Intel through Vulkan; CPU fallback). No account, no server, no
internet after setup. It is a satirical reply to Wispr Flow, which charges 12 to 15 dollars a
month and raised 361 million dollars (280 million Series B at a 2 billion valuation, TechCrunch,
17 August 2026). Made by Luram AI Agency, Thessaloniki (luram.gr).

Audience: Windows users who write a lot (Greek professionals, developers, bilingual people),
people who saw the Wispr Flow price, people who care where their voice goes.

## Brand: this is fixed, do not invent another look
The app itself is black, monospace, square corners, one acid-green accent. The site must read as
the same family, only richer: images, graphics and a WebGL shader playing softly in the
background. The client asked for "premium". Premium here means generous spacing, confident type
scale, restraint, crisp alignment, one memorable element, no clutter.

Tokens (dark theme, the default):
- `--bg: #000` page, `--bg2: #0c0c0c` raised surfaces, `--card: #141414`, `--line: #2a2a2a`
- `--fg: #f2f2f2`, `--muted: #a3a3a3` (never lighter than this on black; contrast matters)
- `--acid: #c8ff5a` the ONE accent, `--acid-deep: #8fd400` for gradients/shader depth,
  `--acid-fg: #0c0c0c` text on acid fills, `--rec: #ff4d4d` only for the "recording" dot
Light theme (`html[data-theme="light"]`), must look intentional, not inverted:
- `--bg: #f6f7f2`, `--bg2: #ffffff`, `--card: #ffffff`, `--line: #d6d9cc`
- `--fg: #0c0c0c`, `--muted: #4f5548`, `--acid: #c8ff5a` for fills with `--acid-fg: #0c0c0c`,
  acid as TEXT on light backgrounds must use `--acid-text: #2f6b00` (contrast), `--acid-deep: #6fae00`
Type: `Consolas, "Cascadia Mono", ui-monospace, "SF Mono", Menlo, monospace` for everything.
Use size, weight and spacing for hierarchy, not a second family.
Corners: square (border-radius 0) everywhere. Borders 1px `--line`.

Scale (accessibility is a permanent rule; the client has diabetic retinopathy):
- body 18px, line-height 1.6; small print never below 15px
- h1 between 44px and 96px depending on the variant, h2 32px to 44px, h3 22px+
- contrast: text on black at least 7:1, focus ring 3px acid visible on every interactive element

## Theme toggle (must exist and work in every variant)
Put this exact script in `<head>` before the stylesheet-dependent markup, then a button with
`id="theme-toggle"` in the header:
```html
<script>
(function(){var r=document.documentElement,s=null;try{s=localStorage.getItem("fyf-theme")}catch(e){}
var sys=window.matchMedia&&window.matchMedia("(prefers-color-scheme: light)").matches?"light":"dark";
r.setAttribute("data-theme",s||sys);
document.addEventListener("DOMContentLoaded",function(){var b=document.getElementById("theme-toggle");if(!b)return;
function label(){b.textContent=r.getAttribute("data-theme")==="light"?"Dark":"Light"}label();
b.addEventListener("click",function(){var n=r.getAttribute("data-theme")==="light"?"dark":"light";
r.setAttribute("data-theme",n);try{localStorage.setItem("fyf-theme",n)}catch(e){}label();
document.dispatchEvent(new CustomEvent("fyf-theme"))})});})();
</script>
```
Your shader must listen for the `fyf-theme` event and re-tint itself (dark: acid light on black;
light: acid ink on pale paper, softer).

## Shader (WebGL, inline GLSL) rules
- One `<canvas>` behind the hero (or behind the whole page if the variant says so), a
  fullscreen quad with a fragment shader. Plain WebGL1 is fine.
- Cap `devicePixelRatio` at 1.5. Pause the loop when the canvas is off screen
  (IntersectionObserver) or the tab is hidden.
- `prefers-reduced-motion: reduce`: render ONE frame and stop.
- If WebGL is unavailable: fall back to a static CSS gradient, the page must still look designed.
- Uniforms: `u_time`, `u_res`, `u_theme` (0 dark, 1 light), optionally `u_mouse`.
- The shader is a background. Text on top must stay readable: keep the shader dark and low in
  luminance under text (dark theme), pale under text (light theme). Add a solid or gradient
  scrim where copy sits if needed.
- Keep the GLSL small (under ~80 lines) and cheap: fbm noise, sine waves, distance fields.

## Assets you may use (relative from variants/)
- `../assets/logo-dark.png` and `../assets/logo-light.png` (1024x1024, the locked logo: a hand
  built from vertical bars with the middle bar tallest, on a black rounded square; the light
  version is the inverse). Never redraw the logo. You may echo its bar motif in CSS/SVG
  decorations.
- `../assets/icon-64.png` for the header brand mark.
- App screenshots 1280x820, dark UI, real markup: `../assets/1-home.png`, `2-history.png`,
  `3-dictionary.png`, `4-suggestions.png`, `5-statistics.png`, `6-settings.png`.
- Generated art (being produced in parallel with ChatGPT, may NOT exist yet when you build):
  `../assets/art-hand-hero.png` (2:3 or 1:1 portrait key visual: the bar-hand as a glossy 3D
  object lit in acid green), `../assets/art-key-macro.png` (16:9 macro photo of a black keyboard
  with one acid-green glowing right Alt key), `../assets/art-receipt.png` (4:5 a crumpled
  subscription receipt / price tag in acid green on black), `../assets/art-wave-letters.png`
  (21:9 an acid sound wave turning into Greek and Latin letters on black).
  Reference them with `<img src="../assets/art-....png" alt="..." onerror="this.parentElement.classList.add('art-missing')">`
  and give the parent a CSS fallback (`.art-missing`) that draws something deliberate with
  CSS/SVG (bars, gradient, the logo) so the page still looks finished if the file is absent.
- The floating "listening" overlay of the app does not exist as an image. Draw it in CSS when a
  variant wants it: 260x84 box, black, 1px line, a red dot, the word "Listening 4.2 s", a row
  of acid-green vertical bars (waveform), all in the mono face.

## Content (all of this is real; numbers are verified, do not change them)
Header: brand "Fuck You Flow" with icon-64; nav anchors; language link "ΕΛ" to `/el/`;
theme toggle. Footer must link luram.gr.

Hero copy (you may re-cut it, keep the facts and the tone: dry, confident, funny once):
- Headline ideas: "Dictation for Windows. Free. Yours." / "Your voice. Your PC. Zero dollars."
- Lead: "Wispr Flow raised 361 million dollars to rent you your own voice for 15 dollars a month.
  We put the same thing behind one key. Greek and English, everything on your PC, nothing in
  the cloud."
- Badge: "beta" and "Windows 10 and 11". Version line in fine print: "Version 0.1.0".
- Primary CTA: "Download for Windows" ->
  https://github.com/Breakzoras/fuck-you-flow/releases/download/v0.1.0/Fuck.You.Flow.Setup.0.1.0.exe
- Secondary: "Read the code" -> https://github.com/Breakzoras/fuck-you-flow
- Fine print: "1.4 GB with every model inside. No internet connection after setup. MIT license.
  The SHA256 checksum is on the release page." (release page:
  https://github.com/Breakzoras/fuck-you-flow/releases/tag/v0.1.0)

The numbers (satire table, every row verified):
| | Wispr Flow | Fuck You Flow |
| Price | 12 to 15 dollars a month | 0, forever |
| Company valuation | 2 billion dollars (August 2026) | One domain name |
| Investor money | 361 million dollars | One weekend |
| Where your voice goes | Their servers | Into the text. Then nowhere. |
| Account | Required | None |
| Internet | Always | Only to download it |
| Greek | One of 100+ languages | First language, mixed with English in the same sentence |
| Source code | Secret | Open, MIT, on GitHub |
Quote: "The models that do Greek well have been open and free for two years. Someone had to put
them behind a key. We did." (Luram AI Agency, Thessaloniki)

How it works (a real sequence, numbering is allowed here):
1. Press right Alt. A small window with a waveform appears. It listens.
2. Speak. Greek, English or both. Each finished phrase is transcribed while you keep talking.
3. Press right Alt again. The text lands where your cursor was. With periods, commas and
   question marks.
Works in any program: Word, Chrome, Slack, Outlook, even a terminal.

Speed (measured on an RTX 3070, large model; wait = key release to text on screen):
| Words | Speech | Wait |
| 32 | 17.5 s | 0.59 s |
| 108 | 82 s | 0.17 s |
| 179 | 77 s | 0.53 s |
| 312 | 152 s | 0.88 s |
Note: the wait depends only on the last phrase, because earlier phrases were transcribed while
you spoke. AMD and Intel cards run through Vulkan at the same speed. Without a graphics card
it runs on the processor, with a wait of a few seconds.

What is inside (six features):
- Greek that holds up: question marks from grammar rules or from the rise of your voice; spoken
  self-corrections are applied.
- A dictionary that learns: correct a word once in History and it becomes a rule.
- Every graphics card: NVIDIA, AMD, Intel. The first start reads the machine and picks the model.
- Snippets: say a phrase, get a whole block of text (signatures, addresses, daily replies).
- History and statistics: words, time saved, what you said yesterday. One file on your disk.
- Nothing leaves: no server, no analytics, no account. Password fields are detected and stay empty.

The app (screenshots, captions): Home: engine, microphone, words today | History: correct it,
it learns | Dictionary: your own words | Suggestions | Statistics | Settings: models, card, keys.

FAQ (six):
1. Is it a free alternative to Wispr Flow? Yes. Same job, one-key dictation into any Windows
   program, no subscription, no account. Open source under MIT on GitHub.
2. Where does my voice go? Nowhere. Recognition runs on your own machine with the Whisper model.
   After setup no internet connection is needed. Password fields are detected and left empty.
3. Does it need an NVIDIA card? No. NVIDIA, AMD and Intel through Vulkan. Without a graphics
   card it runs on the processor, more slowly.
4. Why does Windows show a blue warning during install? The installer carries no purchased
   code-signing certificate. Click "More info", then "Run anyway". The SHA256 checksum is on GitHub.
5. What does beta mean? It runs daily on our machines and is now going out to yours. Some
   programs may refuse the paste, some cards are untested. Whatever breaks, we want it written down.
6. Why the name? A dictation company was valued at 2 billion dollars for something open models
   have done for free for two years. The name is the reply. On social media we write FU Flow.

Feedback (three doors, no backend):
- Something broke -> https://github.com/Breakzoras/fuck-you-flow/issues/new?labels=bug&title=%5Bbug%5D+
  ("Tell us the program, the graphics card and what you saw. The Diagnostics tab has Recent
  problems ready to copy.")
- Something you want -> https://github.com/Breakzoras/fuck-you-flow/issues/new?labels=idea&title=%5Bidea%5D+
- No GitHub account -> mailto:info@luram.gr?subject=Fuck%20You%20Flow%20feedback ("An email is
  enough. Humans answer.")

Footer: "Created by luram.gr, Luram AI Agency. We build tools and automations for Greek
businesses." Links: Code on GitHub, All releases, Known limits
(https://github.com/Breakzoras/fuck-you-flow/blob/main/docs/KNOWN-LIMITATIONS.md), llms.txt,
Ελληνικά (/el/). "MIT license. The whisper.cpp engine and the models carry their own licenses.
Wispr Flow is a trademark of Wispr AI, Inc. We have no relationship with them. The key is yours."

## Writing rules (hard, checked with grep before delivery)
- English, sentence case. Plain verbs. No filler.
- FORBIDDEN in any text: the construction "X, not Y" and its cousins. Never write ", not ",
  " is not ", "rather than", "instead of". Say what something IS.
  Check: `grep -n -iE ", not |is not |rather than|instead of" vN.html` must return nothing.
- No em dashes (the character "—"), no emoji, no "→" in links or buttons, no ALL-CAPS eyebrow
  labels above headings, no middle-dot meta strings ("A · B · C").
- The name stays "Fuck You Flow" exactly.

## Quality floor
- Responsive to 360px wide. Header collapses cleanly.
- Every image has a real alt (decorative ones alt="").
- `prefers-reduced-motion` respected for shader and any CSS animation.
- Keyboard focus visible. Skip link to `#main`.
- Both themes look designed. Test mentally: the light theme is pale paper with black type and
  acid fills, never a washed-out inversion.
- Validate the file opens without console errors (no external requests at all).
- Keep the file under ~60 KB.
