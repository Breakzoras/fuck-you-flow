"""python -m unittest server/report-receiver/test_receiver.py (from the repo root)"""
import json
import os
import sys
import tempfile
import threading
import time
import unittest
import urllib.error
import urllib.request
from http.server import ThreadingHTTPServer
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import receiver  # noqa: E402


class ReceiverTest(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        os.environ.update({"DRY_RUN": "1", "REPORT_DIR": self.dir.name, "REPORT_TO": "maker@example.com",
                           "REPORT_FROM": "Reports <reports@example.com>"})
        receiver.LIMITS = receiver.Limits()
        self.server = ThreadingHTTPServer(("127.0.0.1", 0), receiver.Handler)
        threading.Thread(target=self.server.serve_forever, daemon=True).start()
        self.url = f"http://127.0.0.1:{self.server.server_address[1]}/api/report"

    def tearDown(self):
        self.server.shutdown()
        self.server.server_close()
        self.dir.cleanup()

    def post(self, body, addr="203.0.113.7"):
        data = body if isinstance(body, bytes) else json.dumps(body).encode()
        req = urllib.request.Request(self.url, data=data, method="POST",
                                     headers={"Content-Type": "application/json", "X-Forwarded-For": addr})
        try:
            with urllib.request.urlopen(req) as r:
                return r.status, json.loads(r.read())
        except urllib.error.HTTPError as e:
            return e.code, json.loads(e.read())

    def test_a_problem_is_kept_and_mailed_with_the_log(self):
        code, body = self.post({"kind": "problem", "message": "The key stopped working\nafter sleep",
                                "email": "user@example.org", "app_version": "0.9.13", "os": "windows",
                                "ui_language": "en", "log": "line one\nline two"})
        self.assertEqual(code, 200)
        self.assertTrue(body["ok"] and body["mailed"])
        saved = json.loads((Path(self.dir.name) / f"{body['id']}.json").read_text(encoding="utf-8"))
        self.assertEqual(saved["message"], "The key stopped working\nafter sleep")
        self.assertNotIn("203.0.113.7", json.dumps(saved))
        mail = json.loads((Path(self.dir.name) / "outbox" / f"{body['id']}.json").read_text(encoding="utf-8"))
        self.assertEqual(mail["to"], ["maker@example.com"])
        self.assertEqual(mail["reply_to"], "user@example.org")
        self.assertEqual(mail["subject"], "[FU Flow] Problem: The key stopped working")
        self.assertEqual(len(mail["attachments"]), 1)

    def test_an_idea_without_address_or_log(self):
        code, body = self.post({"kind": "idea", "message": "A dark blue skin"})
        self.assertEqual(code, 200)
        mail = json.loads((Path(self.dir.name) / "outbox" / f"{body['id']}.json").read_text(encoding="utf-8"))
        self.assertNotIn("reply_to", mail)
        self.assertNotIn("attachments", mail)

    def test_bad_input_is_refused(self):
        self.assertEqual(self.post({"kind": "spam", "message": "x"})[0], 400)
        self.assertEqual(self.post({"kind": "idea", "message": "   "})[0], 400)
        self.assertEqual(self.post({"kind": "idea", "message": "hi", "email": "not an address"})[0], 400)
        self.assertEqual(self.post(b"{not json")[0], 400)
        self.assertEqual(self.post({"kind": "idea", "message": "x" * 500_000})[0], 413)

    def test_reports_older_than_90_days_are_deleted(self):
        folder = Path(self.dir.name)
        old, new = folder / "old.json", folder / "new.json"
        old.write_text("{}"), new.write_text("{}")
        long_ago = time.time() - 91 * 86400
        os.utime(old, (long_ago, long_ago))
        receiver.prune(folder, time.time())
        self.assertFalse(old.exists())
        self.assertTrue(new.exists())

    def test_one_address_is_held_to_five_an_hour(self):
        codes = [self.post({"kind": "idea", "message": f"idea {i}"})[0] for i in range(6)]
        self.assertEqual(codes, [200] * 5 + [429])
        self.assertEqual(self.post({"kind": "idea", "message": "someone else"}, addr="198.51.100.1")[0], 200)


class DictionaryTest(unittest.TestCase):
    INSTALL = "0123456789abcdef0123456789abcdef"

    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        os.environ.update({"DRY_RUN": "1", "REPORT_DIR": self.dir.name, "DICT_DIR": str(Path(self.dir.name) / "dictionary")})
        receiver.DICT_LIMITS = receiver.Limits(receiver.DICT_PER_HOUR, receiver.DICT_PER_DAY)
        receiver.NEW_INSTALL_LIMITS = receiver.Limits(receiver.NEW_INSTALLS_PER_HOUR, receiver.NEW_INSTALLS_PER_DAY)
        self.server = ThreadingHTTPServer(("127.0.0.1", 0), receiver.Handler)
        threading.Thread(target=self.server.serve_forever, daemon=True).start()
        self.base = f"http://127.0.0.1:{self.server.server_address[1]}/api/dictionary"

    def tearDown(self):
        self.server.shutdown()
        self.server.server_close()
        self.dir.cleanup()

    def post(self, body, path=""):
        data = body if isinstance(body, bytes) else json.dumps(body).encode()
        req = urllib.request.Request(self.base + path, data=data, method="POST",
                                     headers={"Content-Type": "application/json", "X-Forwarded-For": "203.0.113.9"})
        try:
            with urllib.request.urlopen(req) as r:
                return r.status, json.loads(r.read())
        except urllib.error.HTTPError as e:
            return e.code, json.loads(e.read())

    def kept(self):
        return Path(self.dir.name) / "dictionary" / f"{self.INSTALL}.jsonl"

    def test_shared_corrections_are_kept_per_install_and_forgotten_on_request(self):
        rules = [{"wrong": "kubernets", "correct": "Kubernetes", "language": "en", "match_mode": "whole_word"},
                 {"wrong": "", "correct": "dropped, one side is empty"},
                 {"wrong": "x" * 200, "correct": "dropped, too long"},
                 {"wrong": ["a", "list"], "correct": "dropped, a side that is no text"},
                 {"wrong": "γκίτχαμπ", "correct": "GitHub", "language": "not a code", "match_mode": "odd"}]
        code, body = self.post({"install_id": self.INSTALL, "app_version": "0.9.14", "primary": "el", "rules": rules,
                                "history": "a field nobody asked for"})
        self.assertEqual((code, body["kept"]), (200, 2))
        self.post({"install_id": self.INSTALL, "rules": [{"wrong": "teh", "correct": "the"}]})
        lines = [json.loads(l) for l in self.kept().read_text(encoding="utf-8").splitlines()]
        self.assertEqual(len(lines), 2)
        self.assertEqual(lines[0]["rules"][1], {"wrong": "γκίτχαμπ", "correct": "GitHub", "language": None, "match_mode": "whole_word"})
        self.assertEqual(set(lines[0]), {"app_version", "primary", "rules", "received"})
        self.assertEqual(len(lines[0]["received"]), 10, "the day, with no time of day")
        self.assertNotIn("203.0.113.9", self.kept().read_text(encoding="utf-8"))
        self.assertEqual(self.post({"install_id": self.INSTALL}, "/forget"), (200, {"ok": True}))
        self.assertFalse(self.kept().exists())
        self.assertEqual(self.post({"install_id": "f" * 32}, "/forget"), (200, {"ok": True}))

    def test_bad_batches_are_refused(self):
        one = [{"wrong": "teh", "correct": "the"}]
        self.assertEqual(self.post({"install_id": "../../etc/passwd", "rules": one})[0], 400)
        self.assertEqual(self.post({"install_id": self.INSTALL, "rules": []})[0], 400)
        self.assertEqual(self.post({"install_id": self.INSTALL, "rules": one * (receiver.MAX_RULES + 1)})[0], 400)
        self.assertEqual(self.post({"install_id": "nope"}, "/forget")[0], 400)
        self.assertEqual(self.post({"install_id": self.INSTALL + "\n", "rules": one})[0], 400)
        self.assertEqual(self.post({"install_id": 12345, "rules": one})[0], 400)
        self.assertEqual(self.post(b"[" * 20000)[0], 400)
        self.assertFalse(self.kept().exists())
        self.assertEqual(list((Path(self.dir.name) / "dictionary").glob("*")), [])

    def test_one_address_opens_only_a_few_installs_a_day(self):
        one = [{"wrong": "teh", "correct": "the"}]
        codes = [self.post({"install_id": f"{i:032x}", "rules": one})[0] for i in range(receiver.NEW_INSTALLS_PER_HOUR + 1)]
        self.assertEqual(codes, [200] * receiver.NEW_INSTALLS_PER_HOUR + [429])
        # an install that already has its file keeps adding to it
        self.assertEqual(self.post({"install_id": f"{0:032x}", "rules": one})[0], 200)

    def test_one_install_cannot_fill_the_disk(self):
        receiver.MAX_INSTALL_BYTES, before = 300, receiver.MAX_INSTALL_BYTES
        try:
            big = [{"wrong": f"w{i}", "correct": "c" * 60} for i in range(10)]
            self.assertEqual(self.post({"install_id": self.INSTALL, "rules": big})[0], 507)
            self.assertFalse(self.kept().exists())
        finally:
            receiver.MAX_INSTALL_BYTES = before


if __name__ == "__main__":
    unittest.main()
