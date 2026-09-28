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


if __name__ == "__main__":
    unittest.main()
