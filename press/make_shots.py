"""Clean interface pictures for a public post.

Renders the real dashboard markup with the real stylesheet (src/App.css) and
neutral sample content, then photographs each page with headless Chrome. The
result matches what the app draws, without the floating overlay sitting on the
text and without any private dictation in the frame.

Run from the project root: python press/make_shots.py
"""
import io
import os
import subprocess
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "press", "shots")
# Chrome headless treats a space in --screenshot as a second target, so the
# rendering runs in a path without spaces and the files are copied back.
WORK = os.path.join(os.environ.get("TEMP", "."), "ffshots")
CHROME = r"C:\Program Files\Google\Chrome\Application\chrome.exe"
W, H = 1280, 820

CSS = io.open(os.path.join(ROOT, "src", "App.css"), encoding="utf-8").read()

NAV = ["home", "history", "dictionary", "snippets", "suggestions", "statistics", "settings", "diagnostics"]


def shell(active, body, status="Speech engine ready (on the GPU)", chip="ready"):
    items = "".join(
        f'<button class="{"active" if n == active else ""}">{n}</button>' for n in NAV
    )
    return f"""<!doctype html><html lang="el"><head><meta charset="utf-8"><style>
{CSS}
html,body{{width:{W}px;height:{H}px;overflow:hidden}}
</style></head><body><div class="layout">
<nav class="nav">
  <div class="brand"><span class="brand-name"><span>Fuck </span><span class="accent">You </span><span>Flow</span></span></div>
  {items}
  <div class="spacer"></div>
  <div class="status-chip {chip}"><span class="dot"></span><span>{status}</span></div>
</nav>
<main class="main"><div class="page">{body}</div></main>
</div></body></html>"""


def fact(k, v):
    return f'<div class="fact"><span class="k">{k}</span><span class="v">{v}</span></div>'


def kbd(t):
    return f'<span class="kbd">{t}</span>'


HOME = shell("home", f"""
<h1>ready</h1>
<div class="facts">
  {fact("engine", '<b>large-v3-q5_0</b> on the GPU, warm in 1.7 s')}
  {fact("mic", "Windows default")}
  {fact("key", f'{kbd("RAlt")} start · {kbd("RAlt")} stop and insert · {kbd("Esc")} cancel')}
  {fact("today", '<b>2.684</b> words · median wait <b>0.35 s</b> · <b>46 min</b> saved')}
</div>
<div class="row" style="margin:14px 0 26px">
  <button class="btn primary">Test dictation</button>
  <button class="btn">Paste last</button>
</div>
<h2>last transcripts</h2>
<div class="log" style="margin-top:8px">
  <div><span class="t">11:42</span> <span class="ok">ok</span> <span class="t">133w 0.35s</span>  <span class="body">Καλημέρα, στείλε στον λογιστή το τιμολόγιο του Σεπτεμβρίου και βάλε στο θέμα τον…</span></div>
  <div><span class="t">11:39</span> <span class="ok">ok</span> <span class="t">47w 0.59s</span>  <span class="body">Το ραντεβού με τον πελάτη μετακινείται για την Πέμπτη στις έντεκα το πρωί.</span></div>
  <div><span class="t">11:36</span> <span class="ok">ok</span> <span class="t">22w 0.41s</span>  <span class="body">Θέλω μια σύντομη περίληψη της συνάντησης για να την περάσω στο αρχείο μας.</span></div>
  <div><span class="t">11:31</span> <span class="ok">ok</span> <span class="t">88w 1.12s</span>  <span class="body">Please push the fix to the main branch and open a pull request before noon.</span></div>
  <div><span class="t">11:28</span> <span class="ok">ok</span> <span class="t">41w 0.61s</span>  <span class="body">Να ετοιμάσουμε την προσφορά με τρεις επιλογές και να τη στείλουμε σήμερα.</span></div>
</div>
""")


def hist(text, meta, extra=""):
    return f"""<div class="history-item">
  <div class="history-text">{text}</div>
  <div class="history-meta">{meta}</div>
  {extra}
  <div class="history-actions">
    <button class="btn">Copy</button><button class="btn">Paste</button>
    <button class="btn">Raw transcript</button><button class="btn">Run cleanup again</button>
    <button class="btn">Add to dictionary</button>
  </div>
  <div class="history-actions"><button class="btn">Edit the transcript. Fuck You Flow learns from your edits.</button><button class="btn danger">Delete</button></div>
</div>"""


HISTORY = shell("history", f"""
<h1>History</h1>
<div class="row" style="margin-bottom:14px">
  <input type="search" placeholder="Search" style="max-width:420px" value="">
  <button class="btn danger">Delete all</button>
</div>
<div class="card"><h2 style="margin-bottom:10px">Added to the Dictionary</h2>
  <div class="row" style="align-items:center;gap:12px"><span><b>τιμολόγειο</b> → <b>τιμολόγιο</b></span>
  <button class="btn primary">Accept</button><button class="btn">Dismiss</button></div>
</div>
<div class="history-day">ΣΆΒΒΑΤΟ, 6 ΣΕΠΤΕΜΒΡΊΟΥ 2026</div>
{hist("Καλημέρα, στείλε στον λογιστή το τιμολόγιο του Σεπτεμβρίου και βάλε στο θέμα τον αριθμό της παραγγελίας. Αν δεν απαντήσει μέχρι το μεσημέρι, πάρ' τον τηλέφωνο και ζήτα επιβεβαίωση ότι το έλαβε.",
      "6 Sept 2026, 11:42 · in Word · 38 words · 0.35 s · <span class='status-success'>success</span>")}
{hist("Το ραντεβού με τον πελάτη μετακινείται για την Πέμπτη στις έντεκα το πρωί. Θα χρειαστούμε την παρουσίαση και δύο αντίγραφα της προσφοράς.",
      "6 Sept 2026, 11:39 · in Chrome · 24 words · 0.59 s · <span class='status-success'>success</span>")}
""")

DICT = shell("dictionary", """
<h1>Dictionary</h1>
<div class="card"><h2 style="margin-bottom:12px">NEW RULE</h2>
<div class="inline-form">
  <label class="field"><span class="field-label">Heard as</span><input type="text" value="τιμολόγειο"><span class="hint">e.g. Λούραμ</span></label>
  <label class="field"><span class="field-label">Replace with</span><input type="text" value="τιμολόγιο"><span class="hint">e.g. Luram</span></label>
  <button class="btn primary">Save</button>
</div>
</div>
<div class="card">
  <div class="card-head"><h2>RULES (6)</h2><div class="card-actions"><input type="search" placeholder="Search" style="max-width:260px"><button class="btn">Import</button><button class="btn">Export</button></div></div>
  <table>
    <tr><th>Heard as</th><th>Replace with</th><th>Match</th><th>Applies to</th><th>Used</th><th></th></tr>
    <tr><td>τιμολόγειο</td><td>τιμολόγιο</td><td>Whole word</td><td>All languages</td><td>applied 4 times</td><td><button class="btn">Delete</button></td></tr>
    <tr><td>ουάτσαπ</td><td>WhatsApp</td><td>Phrase</td><td>All languages</td><td>applied 3 times</td><td><button class="btn">Delete</button></td></tr>
    <tr><td>μέηλ</td><td>email</td><td>Whole word</td><td>Greek</td><td>applied 9 times</td><td><button class="btn">Delete</button></td></tr>
    <tr><td>εξέλ</td><td>Excel</td><td>Whole word</td><td>All languages</td><td>applied 2 times</td><td><button class="btn">Delete</button></td></tr>
    <tr><td>άι ντι</td><td>ID</td><td>Phrase</td><td>All languages</td><td>never</td><td><button class="btn">Delete</button></td></tr>
    <tr><td>εν οκτώ εν</td><td>n8n</td><td>Phrase</td><td>All languages</td><td>applied 1 time</td><td><button class="btn">Delete</button></td></tr>
  </table>
</div>
""")

SUGG = shell("suggestions", """
<h1>Suggestions</h1>
<div class="card">
  <div class="row"><button class="btn">Pause learning</button><button class="btn danger">Delete learning data</button></div>
  <p class="hint" style="margin-top:10px">A correction you make in History becomes a Dictionary rule here.</p>
</div>
<div class="card">
  <table>
    <tr><th>Heard as</th><th>Replace with</th><th>Status</th><th>Why</th></tr>
    <tr><td>τιμολόγειο</td><td>τιμολόγιο</td><td><span class="badge ok">accepted</span></td><td>You changed "τιμολόγειο" to "τιμολόγιο" after dictating (3 time(s)).</td></tr>
    <tr><td>ουάτσαπ</td><td>WhatsApp</td><td><span class="badge ok">accepted</span></td><td>You changed "ουάτσαπ" to "WhatsApp" after dictating (2 time(s)).</td></tr>
    <tr><td>εν οκτώ εν</td><td>n8n</td><td><span class="badge">pending</span></td><td>You changed "εν οκτώ εν" to "n8n" after dictating (1 time(s)).</td></tr>
    <tr><td>Κλοντ</td><td>Claude</td><td><span class="badge">pending</span></td><td>You changed "Κλοντ" to "Claude" after dictating (1 time(s)).</td></tr>
  </table>
</div>
""")


def stat(v, l, sub=""):
    s = f'<div class="stat-sub">{sub}</div>' if sub else ""
    return f'<div class="stat"><div class="stat-value">{v}</div><div class="stat-label">{l}</div>{s}</div>'


STATS = shell("statistics", f"""
<h1>Statistics</h1>
<div class="grid4" style="margin-bottom:14px">
  {stat("2.684", "Total words")}{stat("21 minutes", "Minutes dictated")}
  {stat("126", "Average words per minute")}{stat("54", "Dictations")}
</div>
<div class="grid4" style="margin-bottom:14px">
  {stat("46 minutes", "Time saved vs. typing", "words / 40 words per minute of typing, minus minutes spoken")}
  {stat("1", "Daily streak", "Longest streak: 1")}{stat("2", "Dictionary replacements")}{stat("0 / 6", "Retries / Manual edits")}
</div>
<div class="grid4" style="margin-bottom:14px">
  {stat("350 ms", "Median latency (p50)")}{stat("1.644 ms", "Latency, 95th percentile (p95)")}
</div>
<div class="card"><h2 style="margin-bottom:10px">BY APPLICATION TYPE</h2>
<table><tr><th>Category</th><th>Words</th></tr>
<tr><td>Chat</td><td>1.412</td></tr><tr><td>Browser</td><td>806</td></tr><tr><td>Editor</td><td>466</td></tr></table></div>
""")


def tab(name, on=False):
    return f'<button class="{"active" if on else ""}">{name}</button>'


SETTINGS = shell("settings", f"""
<h1>Settings</h1>
<div class="tabs">
  {tab("General")}{tab("Microphone")}{tab("Language")}{tab("Speech models", True)}{tab("Keyboard shortcuts")}
  {tab("Transcript cleanup")}{tab("Text insertion")}{tab("Overlay")}{tab("Privacy")}{tab("Per-app styles")}{tab("Cloud provider")}
</div>
<div class="card">
  <div class="card-head"><h2>SPEECH ENGINE (WHISPER.CPP WITH CUDA)</h2><div class="card-actions"><button class="btn">Restart speech engine</button></div></div>
  <p>Engine version: b4938 (CUDA 12.4) · Size: 671 MB · NVIDIA driver: <span class="badge ok">found</span> · <span class="badge ok">Installed</span></p>
  <p class="hint">ready · large-v3-q5_0 · on the GPU · 1.749 ms</p>
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
  <tr><td>Whisper large-v3 (q5_0) <span class="badge ok">Recommended</span><div class="hint">Ελληνικά, Αγγλικά και 97 ακόμα. Η καλύτερη ακρίβεια στα ελληνικά.</div></td><td>1.031 MB</td><td>~2.600 MB</td><td><span class="badge ok">Installed</span> <span class="badge ok">Active</span> <button class="btn">Verify</button></td></tr>
  <tr><td>Whisper large-v3 turbo (q5_0)<div class="hint">Πιο γρήγορο, δουλεύει άνετα και χωρίς κάρτα γραφικών.</div></td><td>547 MB</td><td>~1.500 MB</td><td><span class="badge ok">Installed</span> <button class="btn">Use this model</button></td></tr>
  <tr><td>Whisper medium (q5_0)<div class="hint">Μικρότερο, για παλιότερα μηχανήματα.</div></td><td>514 MB</td><td>~1.200 MB</td><td><button class="btn">Download</button></td></tr>
</table>
</div>
""")

PAGES = {"1-home": HOME, "2-history": HISTORY, "3-dictionary": DICT, "4-suggestions": SUGG, "5-statistics": STATS, "6-settings": SETTINGS}

import shutil

os.makedirs(OUT, exist_ok=True)
os.makedirs(WORK, exist_ok=True)
for name, html in PAGES.items():
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
    if os.path.exists(png):
        shutil.copy(png, os.path.join(OUT, name + ".png"))
        print(name, "->", os.path.getsize(png), "bytes")
    else:
        print(name, "-> MISSING")
