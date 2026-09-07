# -*- coding: utf-8 -*-
"""Photograph the built pages over a local server, so absolute /assets/ paths resolve.

Nothing here touches the live site. Run:
    "C:/Python314/python.exe" "C:/Claude Projects/lalia/site/shoot_pages.py"
"""
import base64
import functools
import http.server
import os
import socketserver
import sys
import threading
import time

from PIL import Image
from selenium import webdriver
from selenium.webdriver.chrome.options import Options

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "variants", "shots")
os.makedirs(OUT, exist_ok=True)
PORT = 8731

PAGES = sys.argv[1:] or ["/", "/el/", "/wispr-flow-alternative/"]


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass


def serve():
    handler = functools.partial(Quiet, directory=HERE)
    socketserver.TCPServer.allow_reuse_address = True
    httpd = socketserver.TCPServer(("127.0.0.1", PORT), handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


def driver(w, h):
    o = Options()
    for a in ("--headless=new", "--use-angle=swiftshader", "--enable-unsafe-swiftshader",
              "--hide-scrollbars", "--force-device-scale-factor=1", "--window-size=%d,%d" % (w, h)):
        o.add_argument(a)
    o.set_capability("goog:loggingPrefs", {"browser": "ALL"})
    return webdriver.Chrome(options=o)


def full(d, w, path):
    # cssContentSize over-reports on this page and Chrome fills the extra strip by
    # repeating the top of the document, which reads as a duplicated page. scrollHeight
    # is the honest number.
    # Grow the window to the whole document and take a plain screenshot. Capturing
    # beyond the viewport with a height read before the resize made Chrome fill the
    # extra strip by repeating the top of the page, which reads as a duplicated page.
    h = int(d.execute_script("return document.documentElement.scrollHeight"))
    d.set_window_size(w, min(h, 24000))
    time.sleep(1.2)
    h = int(d.execute_script("return document.documentElement.scrollHeight"))
    d.set_window_size(w, min(h, 24000))
    time.sleep(0.8)
    d.save_screenshot(path)
    return h


httpd = serve()
time.sleep(0.6)
try:
    # dark desktop is the default view, but light and phone break in different ways and
    # both have to be looked at. Shooting one of the three is how a broken theme ships.
    CONFIGS = [("dark", 1440, 900), ("light", 1440, 900), ("dark", 390, 844)]
    for page in PAGES:
        name = "page-root" if page == "/" else "page-" + page.strip("/").replace("/", "-")
        for theme, w, h0 in CONFIGS:
            tag = "%s-%s" % (theme, "phone" if w < 500 else "desk")
            d = driver(w, h0)
            try:
                d.get("http://127.0.0.1:%d%s" % (PORT, page))
                time.sleep(1.2)
                if theme == "light":
                    # the ASCII shader repaints on this event, same as the toggle does
                    d.execute_script(
                        "document.documentElement.setAttribute('data-theme','light');"
                        "document.dispatchEvent(new CustomEvent('fyf-theme'));")
                time.sleep(2.6)
                d.save_screenshot(os.path.join(OUT, "%s-%s-fold.png" % (name, tag)))
                tall = os.path.join(OUT, "%s-%s-full.png" % (name, tag))
                h = full(d, w, tall)
                im = Image.open(tall)
                im.thumbnail((900 if w > 500 else 420, 20000), Image.LANCZOS)
                im.save(tall)
                errs = [e for e in d.get_log("browser") if e["level"] == "SEVERE"]
                print("%-26s %-11s %4dw  height %5d px   console errors: %d"
                      % (page, tag, w, h, len(errs)))
                for e in errs[:3]:
                    print("      ", e["message"][:150])
            finally:
                d.quit()
finally:
    httpd.shutdown()
print("shots in", OUT)
