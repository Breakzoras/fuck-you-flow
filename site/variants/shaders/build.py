"""Generate the ten shader candidate pages and the gallery that compares them.

Every page is the v4 Studio hero with one shader behind it, so the choice is made
in the real setting. Run: "C:/Python314/python.exe" build.py
"""
import io
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))

# key, file slug, human title. Order matches presets.js.
DEMOS = [
    ("godRays", "god-rays", "God rays"),
    ("meshGradient", "mesh-gradient", "Mesh gradient"),
    ("grainGradient", "grain-gradient", "Grain gradient"),
    ("smokeRing", "smoke-ring", "Smoke ring"),
    ("liquidMetal", "liquid-metal", "Liquid metal"),
    ("neuroNoise", "neuro-noise", "Neuro noise"),
    ("warp", "warp", "Warp"),
    ("dithering", "dithering", "Dithering"),
    ("waves", "waves", "Waves"),
    ("gemSmoke", "gem-smoke", "Gem smoke"),
]

NOTES = {}


def read_notes():
    """Pull the one-line description of each shader out of presets.js."""
    s = io.open(os.path.join(HERE, "presets.js"), encoding="utf-8").read()
    for key, _, _ in DEMOS:
        m = re.search(re.escape(key) + r":\s*\{\s*\n\s*title:.*?\n\s*note:\s*\"(.*?)\"", s, re.S)
        NOTES[key] = m.group(1) if m else ""


THEME = """<script>
(function(){var r=document.documentElement,s=null;try{s=localStorage.getItem("fyf-theme")}catch(e){}
var sys=window.matchMedia&&window.matchMedia("(prefers-color-scheme: light)").matches?"light":"dark";
r.setAttribute("data-theme",s||sys);
document.addEventListener("DOMContentLoaded",function(){var b=document.getElementById("theme-toggle");if(!b)return;
function label(){b.textContent=r.getAttribute("data-theme")==="light"?"Dark":"Light"}label();
b.addEventListener("click",function(){var n=r.getAttribute("data-theme")==="light"?"dark":"light";
r.setAttribute("data-theme",n);try{localStorage.setItem("fyf-theme",n)}catch(e){}label();
document.dispatchEvent(new CustomEvent("fyf-theme"))})});})();
</script>"""


def nav(current):
    out = []
    for key, slug, title in DEMOS:
        n = "%02d" % (index_of(key) + 1)
        cur = ' aria-current="page"' if key == current else ""
        out.append('<a href="s%s-%s.html"%s>%s %s</a>' % (n, slug, cur, n, title))
    out.append('<a href="index.html">All ten</a>')
    return "\n        ".join(out)


def index_of(key):
    return [k for k, _, _ in DEMOS].index(key)


PAGE = """<!doctype html>
<html lang="en" data-theme="dark">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{n}. {title}: Fuck You Flow shader candidate</title>
<meta name="robots" content="noindex">
<link rel="icon" href="../../assets/favicon.ico" sizes="48x48">
<link rel="stylesheet" href="demo.css">
{theme}
</head>
<body>
<a class="skip" href="#main">Skip to content</a>

<header class="top">
  <div class="wrap top-in">
    <a class="brand" href="index.html"><img src="../../assets/icon-64.png" alt="" width="32" height="32">Fuck You Flow</a>
    <nav class="nav" aria-label="Sections">
      <a href="#main">Features</a><a href="#main">Speed</a><a href="#main">The bill</a>
      <a href="#main">FAQ</a><a href="https://github.com/Breakzoras/fuck-you-flow">GitHub</a>
    </nav>
    <div class="tools">
      <a href="#main" hreflang="el" lang="el">ΕΛ</a>
      <button id="theme-toggle" type="button">Light</button>
    </div>
  </div>
</header>

<main id="main">
<section class="hero" aria-labelledby="h1">
  <div id="shader-host" aria-hidden="true"></div>
  <div class="gl-fallback" aria-hidden="true"></div>
  <div class="wrap">
    <div class="hero-copy">
      <div class="badges"><span>beta</span><span>Windows 10 and 11</span></div>
      <h1 id="h1">Dictation that lives on your PC.</h1>
      <p class="lead">Wispr Flow raised 361 million dollars to rent you your own voice for 15 dollars a month. We put the same thing behind one key. Greek and English, everything on your PC, nothing in the cloud.</p>
      <div class="cta">
        <a class="btn btn-acid" href="https://github.com/Breakzoras/fuck-you-flow/releases/download/v0.1.0/Fuck.You.Flow.Setup.0.1.0.exe">Download for Windows</a>
        <a class="btn" href="https://github.com/Breakzoras/fuck-you-flow">Read the code</a>
      </div>
      <p class="fine">Version 0.1.0. 1.4 GB with every model inside. No internet connection after setup. MIT license. The SHA256 checksum is on the <a href="https://github.com/Breakzoras/fuck-you-flow/releases/tag/v0.1.0">release page</a>.</p>
    </div>

    <div class="frame hero-frame">
      <div class="frame-bar"><span>Fuck You Flow</span><i aria-hidden="true"><b></b><b></b><b></b></i></div>
      <img src="../../assets/1-home.png" width="1280" height="820" alt="The Fuck You Flow home screen: the large-v3 engine on the GPU, the right Alt key for start and stop, words dictated today and the last five transcripts in Greek and English.">
      <div class="listen" role="img" aria-label="The listening window: a red dot, Listening 4.2 s, and a green waveform.">
        <div class="st"><span class="dot"></span><span>Listening 4.2 s</span></div>
        <div class="bars" aria-hidden="true">{bars}</div>
      </div>
    </div>
  </div>
</section>

<section class="demo-bar">
  <div class="wrap">
    <h2>Candidate {n} of 10: <b>{title}</b></h2>
    <p>{note} <span id="shader-state"></span></p>
    <nav class="demo-nav" aria-label="The ten candidates">
        {nav}
    </nav>
  </div>
</section>
</main>

<script src="vendor/paper-shaders.iife.js"></script>
<script src="vendor/shader-config.js"></script>
<script src="presets.js"></script>
<script>window.DEMO_KEY = "{key}";</script>
<script src="demo.js"></script>
</body>
</html>
"""

GALLERY = """<!doctype html>
<html lang="el" data-theme="dark">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Δέκα shader για το Studio</title>
<meta name="robots" content="noindex">
<link rel="icon" href="../../assets/favicon.ico" sizes="48x48">
<style>
body{{margin:0;background:#000;color:#f2f2f2;
font:18px/1.55 Consolas,"Cascadia Mono",ui-monospace,monospace;padding:28px}}
h1{{font-size:30px;margin:0 0 8px}}
.intro{{color:#a3a3a3;max-width:84ch;margin:0 0 10px}}
.intro b{{color:#c8ff5a;font-weight:400}}
.grid{{display:grid;grid-template-columns:repeat(auto-fill,minmax(430px,1fr));gap:22px;margin-top:26px}}
figure{{margin:0;border:1px solid #2a2a2a;background:#0c0c0c}}
figure img{{width:100%;display:block;border-bottom:1px solid #2a2a2a}}
figcaption{{padding:14px 16px}}
figcaption h2{{font-size:21px;margin:0 0 6px}}
figcaption h2 a{{color:#c8ff5a}}
figcaption p{{margin:0 0 10px;color:#a3a3a3;font-size:16px}}
figcaption .links a{{font-size:15px;color:#f2f2f2;margin-right:14px}}
.pair{{display:grid;grid-template-columns:1fr 1fr}}
.pair img{{border-bottom:1px solid #2a2a2a}}
.pair img:first-child{{border-right:1px solid #2a2a2a}}
</style>
</head>
<body>
<h1>Δέκα shader για το Studio</h1>
<p class="intro">Κάθε εικόνα είναι το ΙΔΙΟ hero του Studio με άλλο φόντο. Αριστερά σκοτεινό, δεξιά φωτεινό.
Πάτα τον τίτλο για να το δεις να κινείται. Βιβλιοθήκη: <b>Paper Shaders 0.0.80, άδεια Apache 2.0</b>,
ελεύθερη για εμπορική χρήση, χωρίς αναφορά. Τα χρώματα είναι δικά μας.</p>
<div class="grid">
{cards}
</div>
</body>
</html>
"""

CARD = """<figure>
  <div class="pair">
    <img src="shots/s{n}-{slug}-dark.png" alt="{title}, σκοτεινό θέμα" loading="lazy">
    <img src="shots/s{n}-{slug}-light.png" alt="{title}, φωτεινό θέμα" loading="lazy">
  </div>
  <figcaption>
    <h2><a href="s{n}-{slug}.html">{n}. {title}</a></h2>
    <p>{note}</p>
    <div class="links"><a href="s{n}-{slug}.html">Άνοιξέ το</a></div>
  </figcaption>
</figure>"""


def main():
    read_notes()
    bars = "<b></b>" * 28
    for key, slug, title in DEMOS:
        n = "%02d" % (index_of(key) + 1)
        html = PAGE.format(n=n, slug=slug, title=title, key=key, theme=THEME,
                           bars=bars, nav=nav(key), note=NOTES[key])
        path = os.path.join(HERE, "s%s-%s.html" % (n, slug))
        io.open(path, "w", encoding="utf-8").write(html)
        print("wrote", os.path.basename(path))

    cards = "\n".join(
        CARD.format(n="%02d" % (index_of(k) + 1), slug=s, title=t, note=NOTES[k])
        for k, s, t in DEMOS)
    io.open(os.path.join(HERE, "index.html"), "w", encoding="utf-8").write(
        GALLERY.format(cards=cards))
    print("wrote index.html")


if __name__ == "__main__":
    main()
