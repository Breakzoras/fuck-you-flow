# -*- coding: utf-8 -*-
"""Build the two changelog pages of fuckyouflow.app.

Output:
    site/changelog/index.html      English
    site/el/changelog/index.html   Greek

The text comes from changelog_source.py. The look comes from the same place the
home page gets it, site/variants/v4-coral.html: this script lifts that file's
theme script and its whole style block, so the changelog can never drift into a
second design. A handful of rules for the list itself are added at the end.

Run from anywhere:
    "C:/Python314/python.exe" "C:/Claude Projects/lalia/site/build_changelog.py"
"""
import io
import os
import re
import sys

import changelog_source

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "variants", "v4-coral.html")
BASE = "https://fuckyouflow.app"
REPO = "https://github.com/Breakzoras/fuck-you-flow"

OUT = {
    "en": os.path.join(HERE, "changelog", "index.html"),
    "el": os.path.join(HERE, "el", "changelog", "index.html"),
}

WORDS = {
    "en": {
        "title": "Changelog: every version of Fuck You Flow",
        "desc": "What changed and what was fixed in every released version of Fuck You Flow, "
                "the free offline dictation app for Windows.",
        "h1": "What changed, version by version",
        "lead": "Every release, newest first. Each line says what it does for you.",
        "home": "Home",
        "download": "Download",
        "code": "GitHub",
        "release": "Release page",
        "skip": "Skip to content",
        "toggle": "Light",
        "other": "ΕΛ",
        "other_href": "/el/changelog/",
        "foot": "Fuck You Flow is free software under the MIT licence. Luram AI Agency, Thessaloniki.",
        "back": "Back to the download",
    },
    "el": {
        "title": "Ιστορικό αλλαγών: κάθε έκδοση του Fuck You Flow",
        "desc": "Τι άλλαξε και τι διορθώθηκε σε κάθε έκδοση του Fuck You Flow, της δωρεάν "
                "εφαρμογής υπαγόρευσης για Windows που δουλεύει χωρίς ίντερνετ.",
        "h1": "Τι άλλαξε, έκδοση προς έκδοση",
        "lead": "Κάθε έκδοση, με τη νεότερη πρώτη. Κάθε γραμμή λέει τι σημαίνει για εσάς.",
        "home": "Αρχική",
        "download": "Κατέβασμα",
        "code": "GitHub",
        "release": "Σελίδα έκδοσης",
        "skip": "Στο περιεχόμενο",
        "toggle": "Φωτεινό",
        "other": "EN",
        "other_href": "/changelog/",
        "foot": "Το Fuck You Flow είναι ελεύθερο λογισμικό με άδεια MIT. Luram AI Agency, Θεσσαλονίκη.",
        "back": "Πίσω στο κατέβασμα",
    },
}

EXTRA_CSS = """
/* the changelog list */
.rel{border-top:1px solid var(--line);padding:34px 0}
.rel:first-of-type{border-top:0}
.rel-head{display:flex;align-items:baseline;gap:14px;flex-wrap:wrap;margin-bottom:6px}
.rel-head h2{font-size:clamp(26px,3.4vw,34px);line-height:1.15}
.rel-date{color:var(--muted);font-size:16px}
.rel-sum{color:var(--muted);font-size:18px;line-height:1.55;max-width:64ch;margin-bottom:20px}
.kind{margin-top:18px}
.kind h3{font-size:15px;letter-spacing:.14em;text-transform:uppercase;color:var(--coral-text);margin-bottom:10px}
.kind ul{list-style:none;display:grid;gap:12px}
.kind li{font-size:18px;line-height:1.6;max-width:74ch;padding-left:22px;position:relative}
.kind li::before{content:"";position:absolute;left:0;top:.62em;width:9px;height:2px;background:var(--acid)}
/* The home page centres the top of a section; here the heading has to line up
   with the list under it, or the page reads as two different pages. */
.log-top{padding:56px 0 4px}
.log-top .wrap{text-align:left}
.log-top h1,.log-top .lead{margin-left:0;margin-right:0;text-align:left}
.log-top h1{font-size:clamp(32px,5vw,52px);line-height:1.08;margin-bottom:12px;max-width:22ch}
.log-top .lead{color:var(--muted);font-size:19px;line-height:1.6;max-width:60ch}
.log-top+section{padding-top:8px}
.log-back{margin:34px 0 60px}
@media (max-width:640px){.kind li,.rel-sum{font-size:17px}}
"""


def head_bits():
    """The theme script and the style block, taken from the design source."""
    src = io.open(SRC, encoding="utf-8").read()
    script = re.search(r"<script>\n\(function\(\)\{var r=document\.documentElement.*?</script>", src, re.S)
    style = re.search(r"<style>.*?</style>", src, re.S)
    if not script or not style:
        raise SystemExit("could not lift the theme script or the style block from %s" % SRC)
    css = style.group(0).replace("</style>", EXTRA_CSS + "</style>")
    return script.group(0), css


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def page(lang, script, css):
    w = WORDS[lang]
    canon = BASE + ("/changelog/" if lang == "en" else "/el/changelog/")
    latest = changelog_source.RELEASES[0]
    dl = "%s/releases/download/v%s/Fuck.You.Flow.Setup.%s.exe" % (REPO, latest["version"], latest["version"])
    home = "/" if lang == "en" else "/el/"

    out = []
    for rel in changelog_source.RELEASES:
        ver, date = rel["version"], rel["date"]
        summary = rel["summary"][0 if lang == "en" else 1]
        out.append('<article class="rel" id="v%s">' % ver)
        out.append('  <div class="rel-head"><h2>%s</h2><span class="rel-date">'
                   '<time datetime="%s">%s</time></span>'
                   '<a href="%s/releases/tag/v%s">%s</a></div>'
                   % (esc(ver), date, date, REPO, ver, esc(w["release"])))
        out.append('  <p class="rel-sum">%s</p>' % esc(summary))
        for kind, titles in changelog_source.KINDS.items():
            items = [ln for ln in rel["lines"] if ln[0] == kind]
            if not items:
                continue
            out.append('  <div class="kind">')
            out.append('    <h3>%s</h3>' % esc(titles[0 if lang == "en" else 1]))
            out.append('    <ul>')
            for ln in items:
                out.append('      <li>%s</li>' % esc(ln[1 if lang == "en" else 2]))
            out.append('    </ul>')
            out.append('  </div>')
        out.append('</article>')

    return """<!doctype html>
<html lang="%(lang)s" data-theme="dark">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>%(title)s</title>
<meta name="description" content="%(desc)s">
<link rel="canonical" href="%(canon)s">
<link rel="alternate" hreflang="en" href="%(base)s/changelog/">
<link rel="alternate" hreflang="el" href="%(base)s/el/changelog/">
<link rel="alternate" hreflang="x-default" href="%(base)s/changelog/">
<link rel="icon" href="/assets/favicon.ico" sizes="48x48">
<link rel="icon" href="/assets/icon-64.png" type="image/png" sizes="64x64">
<link rel="apple-touch-icon" href="/assets/icon-180.png">
<meta name="theme-color" content="#000000">
<meta property="og:type" content="website">
<meta property="og:site_name" content="Fuck You Flow">
<meta property="og:url" content="%(canon)s">
<meta property="og:title" content="%(title)s">
<meta property="og:description" content="%(desc)s">
<meta property="og:image" content="%(base)s/assets/og.png">
<meta name="twitter:card" content="summary_large_image">
%(script)s
%(css)s
</head>
<body>
<a class="skip" href="#main">%(skip)s</a>

<header class="top">
  <div class="wrap top-in">
    <a class="brand" href="%(home)s"><img src="/assets/icon-64.png" alt="" width="32" height="32">Fuck You Flow</a>
    <nav class="nav" aria-label="Sections">
      <a href="%(home)s">%(home_word)s</a>
      <a href="%(dl)s">%(download)s</a>
      <a href="%(repo)s">%(code)s</a>
    </nav>
    <div class="tools">
      <a href="%(other_href)s" hreflang="%(other_lang)s" lang="%(other_lang)s">%(other)s</a>
      <button id="theme-toggle" type="button">%(toggle)s</button>
    </div>
  </div>
</header>

<main id="main">
  <section class="log-top">
    <div class="wrap">
      <h1>%(h1)s</h1>
      <p class="lead">%(lead)s</p>
    </div>
  </section>

  <section>
    <div class="wrap">
%(body)s
      <p class="log-back"><a class="btn btn-acid" href="%(dl)s">%(back)s</a></p>
    </div>
  </section>
</main>

<footer class="foot">
  <div class="wrap">
    <p>%(foot)s</p>
  </div>
</footer>
</body>
</html>
""" % {
        "lang": lang,
        "title": esc(w["title"]),
        "desc": esc(w["desc"]),
        "canon": canon,
        "base": BASE,
        "script": script,
        "css": css,
        "skip": esc(w["skip"]),
        "home": home,
        "home_word": esc(w["home"]),
        "dl": dl,
        "download": esc(w["download"]),
        "repo": REPO,
        "code": esc(w["code"]),
        "other_href": w["other_href"],
        "other_lang": "el" if lang == "en" else "en",
        "other": esc(w["other"]),
        "toggle": esc(w["toggle"]),
        "h1": esc(w["h1"]),
        "lead": esc(w["lead"]),
        "body": "\n".join("      " + line for line in out),
        "back": esc(w["back"]),
        "foot": esc(w["foot"]),
    }


def main():
    script, css = head_bits()
    for lang, path in OUT.items():
        os.makedirs(os.path.dirname(path), exist_ok=True)
        body = page(lang, script, css)
        io.open(path, "w", encoding="utf-8", newline="\n").write(body)
        print("wrote %s (%d bytes)" % (path, len(body.encode("utf-8"))))

    # The Greek page must not carry an English sentence, and neither page may
    # carry a comma before the Greek word "και".
    el = io.open(OUT["el"], encoding="utf-8").read()
    bad = re.findall(r", και\b", el)
    if bad:
        print("comma before και in the Greek page: %d places" % len(bad))
        sys.exit(1)
    # Every release in the data has to reach both pages.
    for rel in changelog_source.RELEASES:
        for path in OUT.values():
            if ('id="v%s"' % rel["version"]) not in io.open(path, encoding="utf-8").read():
                print("version %s missing from %s" % (rel["version"], path))
                sys.exit(1)
    print("%d releases on both pages" % len(changelog_source.RELEASES))


if __name__ == "__main__":
    main()
