"""Clean interface pictures for the site and for a public post.

Renders the real dashboard markup with the real stylesheet (src/App.css) and
neutral sample content, then photographs each page with headless Chrome. The
result matches what the app draws, without the floating overlay sitting on the
text and without any private dictation in the frame.

Two sets come out of one run:

    1-home.png      English sample content, for the English page at /
    1-home-el.png   Greek sample content, for the Greek page at /el/

The Greek set is the original one. The English set is not a translation of it:
a dictionary of Greek mishearings makes no sense to an English reader, so the
sample rules, transcripts and hints are written again in English.

Both sets are copied into site/assets/ at the end, which is where the pages
read them from.

Run from the project root: python press/make_shots.py
"""
import io
import os
import shutil
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "press", "shots")
SITE = os.path.join(ROOT, "site", "assets")
# Chrome headless treats a space in --screenshot as a second target, so the
# rendering runs in a path without spaces and the files are copied back.
WORK = os.path.join(os.environ.get("TEMP", "."), "ffshots")
CHROME = r"C:\Program Files\Google\Chrome\Application\chrome.exe"
W, H = 1280, 820

CSS = io.open(os.path.join(ROOT, "src", "App.css"), encoding="utf-8").read()

NAV = ["home", "history", "dictionary", "snippets", "suggestions", "statistics", "settings", "diagnostics"]

# The brand as it is drawn in the sidebar. The site uses the full name. A reel cannot:
# platforms read the text inside a frame, so video builds pass BRAND=("FU ", "", "Flow").
BRAND = ("Fuck ", "You ", "Flow")


# ---------------------------------------------------------------- memory rail

# The rail on the right, drawn from numbers that were measured rather than
# invented: an RTX 3070 with room to spare, and the same card on the afternoon
# of 7 September 2026, when other programs had filled it and Windows had pushed
# 1825 MB of the speech model out to system RAM. Dictation went from about one
# second to over thirty. The markup mirrors src/GpuGauge.tsx, so the real
# stylesheet draws it here exactly as the app draws it.
TOTAL_MB = 8017
NEEDS_MB = 1960

GAUGE_TEXT = {
    "en": {
        "title": "Card memory", "full": "Full", "resident": "Model on the card", "model": "Model", "accuracy": "Accuracy", "speed": "Speed",
        "ok": "Room to spare", "spilled": "Out of room",
        "used": "{used} of {total} MB taken", "free": "{free} MB free",
        "needs": "This model wants {needs} MB",
        "on_card": "{on} of {needs} MB still on the card", "pushed": "{out} MB pushed out to system RAM",
        "why": "Other programs filled the card, so dictation has slowed right down. "
               "Close something heavy, or drop to a smaller model.",
        "switch": "Drop to Whisper medium (q5_0)",
    },
    "el": {
        "title": "Μνήμη κάρτας", "full": "Γεμάτη", "resident": "Το μοντέλο μέσα", "model": "Μοντέλο", "accuracy": "Ακρίβεια", "speed": "Ταχύτητα",
        "ok": "Άνετα", "spilled": "Δεν χωράει",
        "used": "{used} από {total} MB πιασμένα", "free": "{free} MB ελεύθερα",
        "needs": "Το μοντέλο θέλει {needs} MB",
        "on_card": "{on} από {needs} MB πάνω στην κάρτα", "pushed": "{out} MB βγήκαν έξω, στη μνήμη του υπολογιστή",
        "why": "Η κάρτα γέμισε από άλλα προγράμματα, οπότε η υπαγόρευση αργεί πολύ. "
               "Κλείστε κάτι βαρύ ή κατεβείτε σε μικρότερο μοντέλο.",
        "switch": "Κατέβα στο Whisper medium (q5_0)",
    },
}


def bar(label, note, pct, tone=""):
    return (
        '<div class="gauge-bar"><span class="gauge-bar-label">' + label
        + '<span class="gauge-bar-note">' + note + "</span></span>"
        + '<div class="gauge-bar-track"><div class="' + tone + '" style="width:'
        + str(pct) + '%"></div></div></div>'
    )


def gnote(text, tone=""):
    return '<p class="gauge-note' + ((" " + tone) if tone else "") + '">' + text + "</p>"


def gauge(lang, full=False):
    """The right-hand rail. `full` draws the card that had run out of room."""
    s = GAUGE_TEXT[lang]
    used, on_card, pushed = (6160, 133, 1825) if full else (2100, 1960, 0)
    tone = "err" if full else "ok"
    card_pct = round(used / TOTAL_MB * 100)
    resident_pct = round(on_card / NEEDS_MB * 100)

    out = ['<aside class="gauge">']
    out.append("<h2>" + s["title"] + "</h2>")
    out.append('<p class="gauge-word ' + tone + '">' + s["spilled" if full else "ok"] + "</p>")
    out.append(bar(s["full"], str(card_pct) + "%", card_pct, tone))
    out.append(gnote(s["used"].format(used=used, total=TOTAL_MB)))
    out.append(gnote(s["free"].format(free=TOTAL_MB - used)))
    out.append(bar(s["resident"], str(resident_pct) + "%", resident_pct, tone))
    out.append(gnote(s["on_card"].format(on=on_card, needs=NEEDS_MB)))
    if pushed:
        out.append(gnote(s["pushed"].format(out=pushed), "err"))
        out.append('<p class="hint">' + s["why"] + "</p>")
    out.append("<h2>" + s["model"] + "</h2>")
    out.append('<p class="gauge-model-name">Whisper large-v3 (q5_0)</p>')
    out.append(bar(s["accuracy"], "82%", 82))
    out.append(bar(s["speed"], "783 ms", 39))
    if full:
        out.append('<button class="btn primary gauge-switch">' + s["switch"] + "</button>")
    out.append("</aside>")
    return "".join(out)


def shell(lang, active, body, status="Speech engine ready (on the GPU)", chip="ready", card_full=False):
    items = "".join(
        f'<button class="{"active" if n == active else ""}">{n}</button>' for n in NAV
    )
    return f"""<!doctype html><html lang="{lang}"><head><meta charset="utf-8"><style>
{CSS}
html,body{{width:{W}px;height:{H}px;overflow:hidden}}
</style></head><body><div class="layout">
<nav class="nav">
  <div class="brand"><span class="brand-name"><span>{BRAND[0]}</span><span class="accent">{BRAND[1]}</span><span>{BRAND[2]}</span></span></div>
  {items}
  <div class="spacer"></div>
  <div class="status-chip {chip}"><span class="dot"></span><span>{status}</span></div>
</nav>
<main class="main"><div class="page">{body}</div></main>
{gauge(lang, card_full)}
</div></body></html>"""


def fact(k, v):
    return f'<div class="fact"><span class="k">{k}</span><span class="v">{v}</span></div>'


def kbd(t):
    return f'<span class="kbd">{t}</span>'


def logline(t, w, s, body):
    return (f'<div><span class="t">{t}</span> <span class="ok">ok</span> '
            f'<span class="t">{w} {s}</span>  <span class="body">{body}</span></div>')


def hist(text, meta):
    name = "".join(BRAND).strip()
    return f"""<div class="history-item">
  <div class="history-text">{text}</div>
  <div class="history-meta">{meta}</div>
  <div class="history-actions">
    <button class="btn">Copy</button><button class="btn">Paste</button>
    <button class="btn">Raw transcript</button><button class="btn">Run cleanup again</button>
    <button class="btn">Add to dictionary</button>
  </div>
  <div class="history-actions"><button class="btn">Edit the transcript. {name} learns from your edits.</button><button class="btn danger">Delete</button></div>
</div>"""


def stat(v, l, sub=""):
    s = f'<div class="stat-sub">{sub}</div>' if sub else ""
    return f'<div class="stat"><div class="stat-value">{v}</div><div class="stat-label">{l}</div>{s}</div>'


def tab(name, on=False):
    return f'<button class="{"active" if on else ""}">{name}</button>'


# --------------------------------------------------------------- sample content
# Everything a reader can actually read is in here, once per language. The two
# sets are written separately on purpose: an English dictionary of Greek
# mishearings would read as nonsense.

CONTENT = {
    "en": {
        "today": '<b>2,684</b> words · median wait <b>0.35 s</b> · <b>46 min</b> saved',
        "log": [
            ("11:42", "133w", "0.35s", "Good morning, send the accountant the September invoice and put the order number in…"),
            ("11:39", "47w", "0.59s", "The meeting with the client moves to Thursday at eleven in the morning."),
            ("11:36", "22w", "0.41s", "I need a short summary of the meeting to file with the rest of the notes."),
            ("11:31", "88w", "1.12s", "Please push the fix to the main branch and open a pull request before noon."),
            ("11:28", "41w", "0.61s", "Let us put the quote together with three options and send it out today."),
        ],
        "day": "SATURDAY, 6 SEPTEMBER 2026",
        "sugg_pair": ("get hub", "GitHub"),
        "hist": [
            ("Good morning, send the accountant the September invoice and put the order number in the subject line. If nobody answers by midday, call and ask them to confirm it arrived.",
             "6 Sept 2026, 11:42 · in Word · 38 words · 0.35 s · <span class='status-success'>success</span>"),
            ("The meeting with the client moves to Thursday at eleven in the morning. We will need the presentation and two printed copies of the quote.",
             "6 Sept 2026, 11:39 · in Chrome · 24 words · 0.59 s · <span class='status-success'>success</span>"),
        ],
        "rules": [
            ("get hub", "GitHub", "Phrase", "All languages", "applied 4 times"),
            ("what's app", "WhatsApp", "Phrase", "All languages", "applied 3 times"),
            ("vee es code", "VS Code", "Phrase", "All languages", "applied 9 times"),
            ("claud", "Claude", "Whole word", "All languages", "applied 2 times"),
            ("ay pee eye", "API", "Phrase", "All languages", "never"),
            ("en eight en", "n8n", "Phrase", "All languages", "applied 1 time"),
        ],
        "suggestions": [
            ("get hub", "GitHub", "accepted", "ok", 'You changed "get hub" to "GitHub" after dictating (3 time(s)).'),
            ("what's app", "WhatsApp", "accepted", "ok", 'You changed "what\'s app" to "WhatsApp" after dictating (2 time(s)).'),
            ("en eight en", "n8n", "pending", "", 'You changed "en eight en" to "n8n" after dictating (1 time(s)).'),
            ("claud", "Claude", "pending", "", 'You changed "claud" to "Claude" after dictating (1 time(s)).'),
        ],
        "stats": {
            "words": "2,684", "minutes": "21 minutes", "wpm": "126", "dictations": "54",
            "p95": "1,644 ms", "chat": "1,412", "browser": "806", "editor": "466",
        },
        "engine_ms": "1,749 ms",
        "models": [
            ("Whisper large-v3 (q5_0)", "English, Greek and 97 more. The best accuracy on Greek.", "1,031 MB", "~2,600 MB"),
            ("Whisper large-v3 turbo (q5_0)", "Faster, and comfortable even without a graphics card.", "547 MB", "~1,500 MB"),
            ("Whisper medium (q5_0)", "Smaller, for older machines.", "514 MB", "~1,200 MB"),
        ],
    },
    "el": {
        "today": '<b>2.684</b> words · median wait <b>0.35 s</b> · <b>46 min</b> saved',
        "log": [
            ("11:42", "133w", "0.35s", "Καλημέρα, στείλε στον λογιστή το τιμολόγιο του Σεπτεμβρίου και βάλε στο θέμα τον…"),
            ("11:39", "47w", "0.59s", "Το ραντεβού με τον πελάτη μετακινείται για την Πέμπτη στις έντεκα το πρωί."),
            ("11:36", "22w", "0.41s", "Θέλω μια σύντομη περίληψη της συνάντησης για να την περάσω στο αρχείο μας."),
            ("11:31", "88w", "1.12s", "Please push the fix to the main branch and open a pull request before noon."),
            ("11:28", "41w", "0.61s", "Να ετοιμάσουμε την προσφορά με τρεις επιλογές και να τη στείλουμε σήμερα."),
        ],
        "day": "ΣΆΒΒΑΤΟ, 6 ΣΕΠΤΕΜΒΡΊΟΥ 2026",
        "sugg_pair": ("τιμολόγειο", "τιμολόγιο"),
        "hist": [
            ("Καλημέρα, στείλε στον λογιστή το τιμολόγιο του Σεπτεμβρίου και βάλε στο θέμα τον αριθμό της παραγγελίας. Αν δεν απαντήσει μέχρι το μεσημέρι, πάρ' τον τηλέφωνο και ζήτα επιβεβαίωση ότι το έλαβε.",
             "6 Sept 2026, 11:42 · in Word · 38 words · 0.35 s · <span class='status-success'>success</span>"),
            ("Το ραντεβού με τον πελάτη μετακινείται για την Πέμπτη στις έντεκα το πρωί. Θα χρειαστούμε την παρουσίαση και δύο αντίγραφα της προσφοράς.",
             "6 Sept 2026, 11:39 · in Chrome · 24 words · 0.59 s · <span class='status-success'>success</span>"),
        ],
        "rules": [
            ("τιμολόγειο", "τιμολόγιο", "Whole word", "All languages", "applied 4 times"),
            ("ουάτσαπ", "WhatsApp", "Phrase", "All languages", "applied 3 times"),
            ("μέηλ", "email", "Whole word", "Greek", "applied 9 times"),
            ("εξέλ", "Excel", "Whole word", "All languages", "applied 2 times"),
            ("άι ντι", "ID", "Phrase", "All languages", "never"),
            ("εν οκτώ εν", "n8n", "Phrase", "All languages", "applied 1 time"),
        ],
        "suggestions": [
            ("τιμολόγειο", "τιμολόγιο", "accepted", "ok", 'You changed "τιμολόγειο" to "τιμολόγιο" after dictating (3 time(s)).'),
            ("ουάτσαπ", "WhatsApp", "accepted", "ok", 'You changed "ουάτσαπ" to "WhatsApp" after dictating (2 time(s)).'),
            ("εν οκτώ εν", "n8n", "pending", "", 'You changed "εν οκτώ εν" to "n8n" after dictating (1 time(s)).'),
            ("Κλοντ", "Claude", "pending", "", 'You changed "Κλοντ" to "Claude" after dictating (1 time(s)).'),
        ],
        "stats": {
            "words": "2.684", "minutes": "21 minutes", "wpm": "126", "dictations": "54",
            "p95": "1.644 ms", "chat": "1.412", "browser": "806", "editor": "466",
        },
        "engine_ms": "1.749 ms",
        "models": [
            ("Whisper large-v3 (q5_0)", "Ελληνικά, Αγγλικά και 97 ακόμα. Η καλύτερη ακρίβεια στα ελληνικά.", "1.031 MB", "~2.600 MB"),
            ("Whisper large-v3 turbo (q5_0)", "Πιο γρήγορο, δουλεύει άνετα και χωρίς κάρτα γραφικών.", "547 MB", "~1.500 MB"),
            ("Whisper medium (q5_0)", "Μικρότερο, για παλιότερα μηχανήματα.", "514 MB", "~1.200 MB"),
        ],
    },
}


def build_pages(lang):
    c = CONTENT[lang]

    home_body = f"""
<h1>ready</h1>
<div class="facts">
  {fact("engine", '<b>large-v3-q5_0</b> on the GPU, warm in 1.7 s')}
  {fact("mic", "Windows default")}
  {fact("key", f'{kbd("RAlt")} start · {kbd("RAlt")} stop and insert · {kbd("Esc")} cancel')}
  {fact("today", c["today"])}
</div>
<div class="row" style="margin:14px 0 26px">
  <button class="btn primary">Test dictation</button>
  <button class="btn">Paste last</button>
</div>
<h2>last transcripts</h2>
<div class="log" style="margin-top:8px">
  {"".join(logline(*r) for r in c["log"])}
</div>
"""
    # The same page twice, so a guide can put the two states of the card side by
    # side: room to spare, and the afternoon the card ran out of it.
    home = shell(lang, "home", home_body)
    card_full = shell(lang, "home", home_body, card_full=True)

    sp = c["sugg_pair"]
    history = shell(lang, "history", f"""
<h1>History</h1>
<div class="row" style="margin-bottom:14px">
  <input type="search" placeholder="Search" style="max-width:420px" value="">
  <button class="btn danger">Delete all</button>
</div>
<div class="card"><h2 style="margin-bottom:10px">Added to the Dictionary</h2>
  <div class="row" style="align-items:center;gap:12px"><span><b>{sp[0]}</b> → <b>{sp[1]}</b></span>
  <button class="btn primary">Accept</button><button class="btn">Dismiss</button></div>
</div>
<div class="history-day">{c["day"]}</div>
{"".join(hist(t, m) for t, m in c["hist"])}
""")

    rules = "".join(
        f"<tr><td>{a}</td><td>{b}</td><td>{m}</td><td>{ap}</td><td>{u}</td>"
        f'<td><button class="btn">Delete</button></td></tr>'
        for a, b, m, ap, u in c["rules"]
    )
    dictionary = shell(lang, "dictionary", f"""
<h1>Dictionary</h1>
<div class="card"><h2 style="margin-bottom:12px">NEW RULE</h2>
<div class="inline-form">
  <label class="field"><span class="field-label">Heard as</span><input type="text" value="{sp[0]}"><span class="hint">e.g. {"Λούραμ" if lang == "el" else "loo ram"}</span></label>
  <label class="field"><span class="field-label">Replace with</span><input type="text" value="{sp[1]}"><span class="hint">e.g. Luram</span></label>
  <button class="btn primary">Save</button>
</div>
</div>
<div class="card">
  <div class="card-head"><h2>RULES (6)</h2><div class="card-actions"><input type="search" placeholder="Search" style="max-width:260px"><button class="btn">Import</button><button class="btn">Export</button></div></div>
  <table>
    <tr><th>Heard as</th><th>Replace with</th><th>Match</th><th>Applies to</th><th>Used</th><th></th></tr>
    {rules}
  </table>
</div>
""")

    sugg_rows = "".join(
        f'<tr><td>{a}</td><td>{b}</td><td><span class="badge {cls}">{st}</span></td><td>{why}</td></tr>'
        for a, b, st, cls, why in c["suggestions"]
    )
    suggestions = shell(lang, "suggestions", f"""
<h1>Suggestions</h1>
<div class="card">
  <div class="row"><button class="btn">Pause learning</button><button class="btn danger">Delete learning data</button></div>
  <p class="hint" style="margin-top:10px">A correction you make in History becomes a Dictionary rule here.</p>
</div>
<div class="card">
  <table>
    <tr><th>Heard as</th><th>Replace with</th><th>Status</th><th>Why</th></tr>
    {sugg_rows}
  </table>
</div>
""")

    s = c["stats"]
    statistics = shell(lang, "statistics", f"""
<h1>Statistics</h1>
<div class="grid4" style="margin-bottom:14px">
  {stat(s["words"], "Total words")}{stat(s["minutes"], "Minutes dictated")}
  {stat(s["wpm"], "Average words per minute")}{stat(s["dictations"], "Dictations")}
</div>
<div class="grid4" style="margin-bottom:14px">
  {stat("46 minutes", "Time saved vs. typing", "words / 40 words per minute of typing, minus minutes spoken")}
  {stat("1", "Daily streak", "Longest streak: 1")}{stat("2", "Dictionary replacements")}{stat("0 / 6", "Retries / Manual edits")}
</div>
<div class="grid4" style="margin-bottom:14px">
  {stat("350 ms", "Median latency (p50)")}{stat(s["p95"], "Latency, 95th percentile (p95)")}
</div>
<div class="card"><h2 style="margin-bottom:10px">BY APPLICATION TYPE</h2>
<table><tr><th>Category</th><th>Words</th></tr>
<tr><td>Chat</td><td>{s["chat"]}</td></tr><tr><td>Browser</td><td>{s["browser"]}</td></tr><tr><td>Editor</td><td>{s["editor"]}</td></tr></table></div>
""")

    m = c["models"]
    settings = shell(lang, "settings", f"""
<h1>Settings</h1>
<div class="tabs">
  {tab("General")}{tab("Microphone")}{tab("Language")}{tab("Speech models", True)}{tab("Keyboard shortcuts")}
  {tab("Transcript cleanup")}{tab("Text insertion")}{tab("Overlay")}{tab("Privacy")}{tab("Per-app styles")}{tab("Cloud provider")}
</div>
<div class="card">
  <div class="card-head"><h2>SPEECH ENGINE (WHISPER.CPP WITH CUDA)</h2><div class="card-actions"><button class="btn">Restart speech engine</button></div></div>
  <p>Engine version: b4938 (CUDA 12.4) · Size: 671 MB · NVIDIA driver: <span class="badge ok">found</span> · <span class="badge ok">Installed</span></p>
  <p class="hint">ready · large-v3-q5_0 · on the GPU · {c["engine_ms"]}</p>
  <label class="toggle"><input type="checkbox" checked><span class="toggle-track"><span class="toggle-thumb"></span></span><span class="toggle-text">Use the GPU (CUDA)</span></label>
  <label class="toggle"><input type="checkbox" checked><span class="toggle-track"><span class="toggle-thumb"></span></span><span class="toggle-text">Voice activity detection (Silero VAD)<span class="hint">Installed</span></span></label>
  <div class="grid2">
    <label class="field"><span class="field-label">CPU threads</span><input type="number" value="8"></label>
    <label class="field"><span class="field-label">Beam size</span><input type="number" value="3"></label>
  </div>
</div>
<div class="card"><h2 style="margin-bottom:12px">SPEECH MODELS</h2>
<table>
  <tr><th>Model</th><th>Size</th><th>GPU memory</th><th></th></tr>
  <tr><td>{m[0][0]} <span class="badge ok">Recommended</span><div class="hint">{m[0][1]}</div></td><td>{m[0][2]}</td><td>{m[0][3]}</td><td><span class="badge ok">Installed</span> <span class="badge ok">Active</span> <button class="btn">Verify</button></td></tr>
  <tr><td>{m[1][0]}<div class="hint">{m[1][1]}</div></td><td>{m[1][2]}</td><td>{m[1][3]}</td><td><span class="badge ok">Installed</span> <button class="btn">Use this model</button></td></tr>
  <tr><td>{m[2][0]}<div class="hint">{m[2][1]}</div></td><td>{m[2][2]}</td><td>{m[2][3]}</td><td><button class="btn">Download</button></td></tr>
</table>
</div>
""")

    return {"1-home": home, "2-history": history, "3-dictionary": dictionary,
            "4-suggestions": suggestions, "5-statistics": statistics, "6-settings": settings,
            "7-card-full": card_full}


def shoot(name, html):
    """Photograph one page. Returns the path of the PNG, or None."""
    src = os.path.join(WORK, name + ".html")
    io.open(src, "w", encoding="utf-8", newline="\n").write(html)
    png = os.path.join(WORK, name + ".png")
    if os.path.exists(png):
        os.remove(png)
    subprocess.run([CHROME, "--headless=new", "--disable-gpu", "--hide-scrollbars",
                    "--user-data-dir=" + os.path.join(WORK, "profile-" + name),
                    f"--window-size={W},{H}", f"--screenshot={png}", "--force-device-scale-factor=1",
                    "file:///" + src.replace("\\", "/")], check=False, capture_output=True, timeout=120)
    # headless Chrome hands the work to a child process and returns first, so the
    # file shows up a moment after the command is done
    for _ in range(120):
        if os.path.exists(png) and os.path.getsize(png) > 0:
            break
        time.sleep(0.5)
    return png if os.path.exists(png) else None


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    os.makedirs(WORK, exist_ok=True)
    os.makedirs(SITE, exist_ok=True)
    failed = []
    langs = sys.argv[1:] or ["en", "el"]
    tail = os.environ.get("SHOT_SUFFIX", "")
    for lang in langs:
        suffix = ("" if lang == "en" else "-el") + tail
        for name, html in build_pages(lang).items():
            stem = name + suffix
            png = shoot(stem, html)
            if png is None:
                failed.append(stem)
                print(stem, "-> MISSING")
                continue
            shutil.copy(png, os.path.join(OUT, stem + ".png"))
            shutil.copy(png, os.path.join(SITE, stem + ".png"))
            print("%-18s %s bytes  -> press/shots + site/assets" % (stem, os.path.getsize(png)))
    if failed:
        raise SystemExit("MISSING: " + ", ".join(failed))
    print("all pictures written")
