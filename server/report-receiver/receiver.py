"""Receives problem and idea reports sent from inside the FU Flow app.

POST /api/report with a JSON body:
    {"kind": "problem" | "idea", "message": "...", "email": "" (optional),
     "app_version": "0.9.13", "os": "windows 10.0.19045", "ui_language": "el",
     "log": "..." (optional, the diagnostic log the user chose to include)}

Each report is saved as one JSON file and mailed as plain text to the maker
through Resend, with the log attached. Nothing is ever mailed to the sender:
the address is only set as Reply-To, so a reply is a person's decision.

It runs behind Caddy in its own container and is reachable from Caddy only,
so the client address comes from X-Forwarded-For. The source is public, so
nothing here is secret apart from the Resend key in the environment. Abuse is
held off by size limits, a per-address limit and a daily cap on mail.

POST /api/dictionary with a JSON body (0.9.14):
    {"install_id": "<32 hex>", "app_version": "0.9.14", "primary": "el",
     "rules": [{"wrong": "...", "correct": "...", "language": "el" | "en" | null,
                "match_mode": "whole_word" | "phrase" | "exact"}]}
These are Dictionary corrections the app shares (on from the start, with a
switch in Settings under Privacy), so that the ones useful to everyone can be
built into a later version. They are appended to one
file per install id and are never mailed, never published as they arrive and
never pruned by age. POST /api/dictionary/forget with {"install_id": "..."}
deletes that file. The id is a random number the app made up; no address and
no account is stored next to it.

Standard library only. Environment:
    RESEND_API_KEY   key allowed to send mail (not needed with DRY_RUN=1)
    REPORT_TO        where reports go (info@luram.gr)
    REPORT_FROM      verified sender, e.g. "FU Flow reports <reports@example.com>"
    REPORT_DIR       where reports are kept (default /data/reports)
    DICT_DIR         where shared corrections are kept (default /data/dictionary)
    DRY_RUN=1        keep each mail in REPORT_DIR/outbox, send nothing
    PORT             default 8787
"""
import base64
import json
import os
import re
import secrets
import sys
import threading
import time
import urllib.request
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

MAX_BODY = 400 * 1024
MAX_MESSAGE = 5000
MAX_LOG = 300 * 1024
KEEP_DAYS = 90
PER_HOUR = 5
PER_DAY = 20
MAILS_PER_DAY = 60
MAX_DICT_BODY = 200 * 1024
MAX_RULES = 300
MAX_TERM = 80
MAX_INSTALL_BYTES = 1024 * 1024
MAX_DICT_DIR_BYTES = 300 * 1024 * 1024
DICT_PER_HOUR = 30
DICT_PER_DAY = 100
NEW_INSTALLS_PER_HOUR = 3
NEW_INSTALLS_PER_DAY = 6
INSTALL_RE = re.compile(r"^[0-9a-f]{32}$")
LANG_RE = re.compile(r"^[a-z]{2,3}$")
MATCH_MODES = ("whole_word", "phrase", "exact")
EMAIL_RE = re.compile(r"^[^@\s<>\"',;]{1,64}@[^@\s<>\"',;]{1,190}\.[A-Za-z]{2,24}$")
CONTROL_RE = re.compile(r"[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]")


class Limits:
    """Counts per client address in memory only. Addresses are never written
    to disk and are forgotten after a day."""

    def __init__(self, per_hour=PER_HOUR, per_day=PER_DAY):
        self.lock = threading.Lock()
        self.hits = {}
        self.mails = []
        self.per_hour = per_hour
        self.per_day = per_day

    def allow(self, addr, now):
        with self.lock:
            times = [t for t in self.hits.get(addr, []) if now - t < 86400]
            if len([t for t in times if now - t < 3600]) >= self.per_hour or len(times) >= self.per_day:
                self.hits[addr] = times
                return False
            times.append(now)
            self.hits[addr] = times
            if len(self.hits) > 10000:
                self.hits = {a: t for a, t in self.hits.items() if t and now - t[-1] < 86400}
            return True

    def may_mail(self, now):
        with self.lock:
            self.mails = [t for t in self.mails if now - t < 86400]
            if len(self.mails) >= MAILS_PER_DAY:
                return False
            self.mails.append(now)
            return True


LIMITS = Limits()
DICT_LIMITS = Limits(DICT_PER_HOUR, DICT_PER_DAY)
# How many install ids one address may open in a day. Without it a handful of
# addresses could fill the folder with made-up ids and lock everyone else out.
NEW_INSTALL_LIMITS = Limits(NEW_INSTALLS_PER_HOUR, NEW_INSTALLS_PER_DAY)


def clean(text, limit):
    return CONTROL_RE.sub("", str(text or ""))[:limit].strip()


def validate(body):
    """Returns (report, error). The report holds only known, trimmed fields."""
    if not isinstance(body, dict):
        return None, "bad body"
    kind = body.get("kind")
    if kind not in ("problem", "idea"):
        return None, "bad kind"
    message = clean(body.get("message"), MAX_MESSAGE)
    if not message:
        return None, "empty message"
    email = clean(body.get("email"), 254)
    if email and not EMAIL_RE.match(email):
        return None, "bad email"
    report = {
        "kind": kind,
        "message": message,
        "email": email,
        "app_version": clean(body.get("app_version"), 20),
        "os": clean(body.get("os"), 100),
        "ui_language": clean(body.get("ui_language"), 8),
        "log": CONTROL_RE.sub("", str(body.get("log") or ""))[:MAX_LOG],
    }
    return report, None


def validate_dictionary(body):
    """Returns (entry, error). Only known, trimmed fields survive; a rule with
    an empty or oversized side is dropped rather than failing the whole batch."""
    if not isinstance(body, dict):
        return None, "bad body"
    install = body.get("install_id")
    if not (isinstance(install, str) and INSTALL_RE.fullmatch(install)):
        return None, "bad install id"
    raw = body.get("rules")
    if not isinstance(raw, list) or not raw:
        return None, "no rules"
    if len(raw) > MAX_RULES:
        return None, "too many rules"
    rules = []
    for r in raw:
        if not isinstance(r, dict) or not isinstance(r.get("wrong"), str) or not isinstance(r.get("correct"), str):
            continue
        wrong, correct = clean(r.get("wrong"), MAX_TERM + 1), clean(r.get("correct"), MAX_TERM + 1)
        if not wrong or not correct or len(wrong) > MAX_TERM or len(correct) > MAX_TERM:
            continue
        language = r.get("language")
        if not (isinstance(language, str) and LANG_RE.fullmatch(language)):
            language = None
        mode = r.get("match_mode")
        rules.append({"wrong": wrong, "correct": correct, "language": language,
                      "match_mode": mode if mode in MATCH_MODES else "whole_word"})
    if not rules:
        return None, "no rules"
    primary = clean(body.get("primary"), 3).lower()
    entry = {
        "install_id": install,
        "app_version": clean(body.get("app_version"), 20),
        "primary": primary if LANG_RE.fullmatch(primary) else "",
        "rules": rules,
    }
    return entry, None


def folder_bytes(folder):
    total = 0
    for f in folder.glob("*.jsonl"):
        try:
            total += f.stat().st_size
        except OSError:
            pass
    return total


def keep_dictionary(entry, folder):
    """Appends one line to the install's own file. False when that file or the
    whole folder is already as large as it may get."""
    path = folder / f"{entry['install_id']}.jsonl"
    line = json.dumps({k: v for k, v in entry.items() if k != "install_id"}, ensure_ascii=False) + "\n"
    size = path.stat().st_size if path.exists() else 0
    if size + len(line.encode("utf-8")) > MAX_INSTALL_BYTES or folder_bytes(folder) > MAX_DICT_DIR_BYTES:
        return False
    with path.open("a", encoding="utf-8") as f:
        f.write(line)
    return True


def prune(folder, now):
    for f in folder.glob("*.json"):
        try:
            if now - f.stat().st_mtime > KEEP_DAYS * 86400:
                f.unlink()
        except OSError:
            pass


def mail(report, report_id, folder):
    kind = "Problem" if report["kind"] == "problem" else "Idea"
    first = report["message"].splitlines()[0][:70] if report["message"] else ""
    lines = [
        f"{kind} report {report_id}",
        f"App {report['app_version'] or '?'} on {report['os'] or '?'}, interface {report['ui_language'] or '?'}",
        f"Reply to: {report['email'] or '(no address given)'}",
        f"Diagnostic log: {'attached' if report['log'] else 'not included'}",
        "",
        report["message"],
    ]
    payload = {
        "from": os.environ.get("REPORT_FROM", "FU Flow reports <reports@luram.gr>"),
        "to": [os.environ.get("REPORT_TO", "info@luram.gr")],
        "subject": f"[FU Flow] {kind}: {first}",
        "text": "\n".join(lines),
    }
    if report["email"]:
        payload["reply_to"] = report["email"]
    if report["log"]:
        payload["attachments"] = [{
            "filename": f"fuflow-log-{report_id}.txt",
            "content": base64.b64encode(report["log"].encode("utf-8")).decode("ascii"),
        }]
    if os.environ.get("DRY_RUN") == "1":
        out = folder / "outbox"
        out.mkdir(exist_ok=True)
        (out / f"{report_id}.json").write_text(json.dumps(payload, ensure_ascii=False, indent=1), encoding="utf-8")
        return True
    req = urllib.request.Request(
        "https://api.resend.com/emails",
        data=json.dumps(payload).encode("utf-8"),
        headers={"Authorization": f"Bearer {os.environ['RESEND_API_KEY']}", "Content-Type": "application/json",
                 "User-Agent": "fuflow-report-receiver/1"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=20) as r:
            return 200 <= r.status < 300
    except Exception as e:  # the report is already on disk; say so in the log
        print(f"mail failed for {report_id}: {e}", file=sys.stderr, flush=True)
        return False


class Handler(BaseHTTPRequestHandler):
    server_version = "fuflow-reports"
    sys_version = ""

    def log_message(self, fmt, *args):  # no client addresses in the container log
        pass

    def reply(self, code, body):
        data = json.dumps(body).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        if self.path == "/api/report/health":
            return self.reply(200, {"ok": True})
        self.reply(404, {"ok": False})

    def dictionary(self):
        try:
            length = int(self.headers.get("Content-Length") or 0)
        except ValueError:
            length = 0
        if length <= 0 or length > MAX_DICT_BODY:
            return self.reply(413, {"ok": False, "error": "too large"})
        addr = (self.headers.get("X-Forwarded-For") or self.client_address[0]).split(",")[0].strip()
        now = time.time()
        if not DICT_LIMITS.allow(addr, now):
            return self.reply(429, {"ok": False, "error": "too many requests, try again later"})
        try:
            body = json.loads(self.rfile.read(length).decode("utf-8"))
        except (ValueError, UnicodeDecodeError, RecursionError):
            return self.reply(400, {"ok": False, "error": "bad json"})
        folder = Path(os.environ.get("DICT_DIR", "/data/dictionary"))
        folder.mkdir(parents=True, exist_ok=True)
        if self.path == "/api/dictionary/forget":
            install = body.get("install_id") if isinstance(body, dict) else None
            if not (isinstance(install, str) and INSTALL_RE.fullmatch(install)):
                return self.reply(400, {"ok": False, "error": "bad install id"})
            try:
                (folder / f"{install}.jsonl").unlink()
            except OSError:
                pass
            # The same answer whether or not a file was there, so the reply
            # never tells a stranger which ids exist.
            return self.reply(200, {"ok": True})
        entry, error = validate_dictionary(body)
        if error:
            return self.reply(400, {"ok": False, "error": error})
        if not (folder / f"{entry['install_id']}.jsonl").exists() and not NEW_INSTALL_LIMITS.allow(addr, now):
            return self.reply(429, {"ok": False, "error": "too many requests, try again later"})
        # The day and no more: a time to the second would pair this line with
        # one request in any log that keeps addresses.
        entry["received"] = datetime.now(timezone.utc).date().isoformat()
        if not keep_dictionary(entry, folder):
            return self.reply(507, {"ok": False, "error": "full"})
        self.reply(200, {"ok": True, "kept": len(entry["rules"])})

    def do_POST(self):
        if self.path in ("/api/dictionary", "/api/dictionary/forget"):
            return self.dictionary()
        if self.path != "/api/report":
            return self.reply(404, {"ok": False})
        try:
            length = int(self.headers.get("Content-Length") or 0)
        except ValueError:
            length = 0
        if length <= 0 or length > MAX_BODY:
            return self.reply(413, {"ok": False, "error": "too large"})
        addr = (self.headers.get("X-Forwarded-For") or self.client_address[0]).split(",")[0].strip()
        now = time.time()
        if not LIMITS.allow(addr, now):
            return self.reply(429, {"ok": False, "error": "too many reports, try again later"})
        try:
            body = json.loads(self.rfile.read(length).decode("utf-8"))
        except (ValueError, UnicodeDecodeError, RecursionError):
            return self.reply(400, {"ok": False, "error": "bad json"})
        report, error = validate(body)
        if error:
            return self.reply(400, {"ok": False, "error": error})
        folder = Path(os.environ.get("REPORT_DIR", "/data/reports"))
        folder.mkdir(parents=True, exist_ok=True)
        prune(folder, now)
        report_id = datetime.now(timezone.utc).strftime("%Y%m%d-%H%M%S-") + secrets.token_hex(3)
        report["received"] = datetime.now(timezone.utc).isoformat(timespec="seconds")
        (folder / f"{report_id}.json").write_text(json.dumps(report, ensure_ascii=False, indent=1), encoding="utf-8")
        mailed = mail(report, report_id, folder) if LIMITS.may_mail(now) else False
        self.reply(200, {"ok": True, "id": report_id, "mailed": mailed})


def prune_forever():
    """Old reports go even when no new one arrives: at start, then hourly."""
    folder = Path(os.environ.get("REPORT_DIR", "/data/reports"))
    while True:
        if folder.is_dir():
            prune(folder, time.time())
        time.sleep(3600)


def main():
    threading.Thread(target=prune_forever, daemon=True).start()
    port = int(os.environ.get("PORT", "8787"))
    ThreadingHTTPServer(("0.0.0.0", port), Handler).serve_forever()


if __name__ == "__main__":
    main()
