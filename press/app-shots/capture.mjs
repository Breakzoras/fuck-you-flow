// The site's interface pictures, taken from the real dashboard.
//
//   node press/app-shots/capture.mjs <output folder> [en|el ...]
//
// Starts the Vite server of this folder (port 1432) unless it already runs,
// opens the installed Chrome headless through the DevTools protocol (no npm
// packages, Node 24 has WebSocket built in), renders at 1280 x 820 like the
// earlier pictures, clicks through the real sidebar and writes, per language,
// 1-home, 2-history, 3-dictionary, 4-suggestions, 5-statistics, 6-settings
// (Speech models tab) and 7-card-full (Home, the card out of room). The Greek
// set carries the suffix -el, as site/assets expects.
//
// A picture is refused when the page threw, asked the stand-in backend for a
// command it does not know, shows a red toast, an em-dash, the GitHub card or
// a test-build stamp.

import { spawn } from "node:child_process";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { resolve, join } from "node:path";
import { tmpdir } from "node:os";

const ROOT = resolve(import.meta.dirname, "..", "..");
const OUT = resolve(process.argv[2] ?? join(tmpdir(), "fyf-app-shots"));
const LANGS = process.argv.slice(3).length ? process.argv.slice(3) : ["en", "el"];
const BASE = "http://127.0.0.1:1432/";
const CDP_PORT = 9353;
const W = 1280, H = 820;

const CHROME = [
  "C:/Program Files/Google/Chrome/Application/chrome.exe",
  "C:/Program Files (x86)/Google/Chrome/Application/chrome.exe",
  `${process.env.LOCALAPPDATA}/Google/Chrome/Application/chrome.exe`,
].find((p) => p && existsSync(p));

// nav: position of the menu button (home, history, dictionary, snippets,
// suggestions, statistics, settings, diagnostics).
const SHOTS = [
  { stem: "1-home", nav: 0, ready: `document.querySelectorAll('.log > div').length >= 5 && !!document.querySelector('.facts b')` },
  { stem: "2-history", nav: 1, ready: `document.querySelectorAll('.history-item').length >= 5` },
  // The page opens with the ready-made dictionary and the new-rule form; the
  // picture shows the rules, as the earlier one did.
  { stem: "3-dictionary", nav: 2, ready: `document.querySelectorAll('table tbody tr').length >= 6`,
    scroll: `document.querySelector('.main table').closest('.card')` },
  { stem: "4-suggestions", nav: 4, ready: `document.querySelector('.main').innerText.includes('n8n')` },
  { stem: "5-statistics", nav: 5, ready: `document.querySelectorAll('.stat').length >= 6` },
  // Speech models opens with "Help me choose" and the engine card; the picture shows the models.
  { stem: "6-settings", nav: 6, tab: 3, ready: `document.querySelectorAll('.main table tbody tr').length >= 3 && !!document.querySelector('.guide-result')`,
    scroll: `document.querySelector('.main table').closest('.card')` },
  { stem: "7-card-full", nav: 0, query: "gauge=full", ready: `document.querySelectorAll('.log > div').length >= 5 && !!document.querySelector('.gauge-switch')` },
];

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
async function up(url) { try { return (await fetch(url)).ok; } catch { return false; } }
async function waitUp(url, ms) { const end = Date.now() + ms; while (Date.now() < end) { if (await up(url)) return true; await sleep(200); } return false; }

class Cdp {
  constructor(ws) {
    this.ws = ws; this.id = 0; this.pending = new Map(); this.handlers = [];
    ws.addEventListener("message", (ev) => {
      const msg = JSON.parse(typeof ev.data === "string" ? ev.data : Buffer.from(ev.data).toString());
      if (msg.id && this.pending.has(msg.id)) {
        const { res, rej } = this.pending.get(msg.id);
        this.pending.delete(msg.id);
        msg.error ? rej(new Error(`${msg.error.message} ${msg.error.data ?? ""}`)) : res(msg.result);
      } else if (msg.method) {
        for (const h of this.handlers) h(msg);
      }
    });
  }
  static async connect(url) {
    const ws = new WebSocket(url);
    await new Promise((res, rej) => { ws.addEventListener("open", res, { once: true }); ws.addEventListener("error", rej, { once: true }); });
    return new Cdp(ws);
  }
  send(method, params = {}) {
    const id = ++this.id;
    this.ws.send(JSON.stringify({ id, method, params }));
    return new Promise((res, rej) => this.pending.set(id, { res, rej }));
  }
  async eval(expr) {
    const r = await this.send("Runtime.evaluate", { expression: expr, awaitPromise: true, returnByValue: true });
    if (r.exceptionDetails) throw new Error(`eval failed: ${r.exceptionDetails.text}`);
    return r.result.value;
  }
  async waitFor(expr, ms, what) {
    const end = Date.now() + ms;
    while (Date.now() < end) {
      try { if (await this.eval(`!!(${expr})`)) return; } catch { /* still loading */ }
      await sleep(100);
    }
    throw new Error(`timed out waiting for ${what}: ${expr}`);
  }
}

const CHECK = `(() => {
  // The Suggestions page titles its answered list with a lone dash, in the app
  // itself since 0.9.0. That one is the interface; any other dash is refused.
  const text = document.body.innerText.split('\\n').filter((l) => l.trim() !== '\\u2014').join('\\n');
  const out = [];
  if (/\\u2014/.test(text)) out.push('em-dash visible');
  if (document.querySelector('.toast.err')) out.push('red toast: ' + document.querySelector('.toast.err').innerText);
  if (document.querySelector('.rate-card')) out.push('the GitHub card is visible');
  if (document.querySelector('.test-build')) out.push('a test-build stamp is visible');
  return out;
})()`;

async function main() {
  if (!CHROME) throw new Error("Chrome not found");
  mkdirSync(OUT, { recursive: true });
  const kids = [];
  process.on("exit", () => { for (const k of kids) { try { k.kill(); } catch { /* gone */ } } });

  if (!(await up(BASE))) {
    kids.push(spawn(process.execPath, [join(ROOT, "node_modules/vite/bin/vite.js"), "--config", "press/app-shots/vite.config.ts"], { cwd: ROOT, stdio: "ignore" }));
    if (!(await waitUp(BASE, 40000))) throw new Error("the picture server did not start on 1432");
  }
  const chrome = spawn(CHROME, [
    "--headless=new", `--remote-debugging-port=${CDP_PORT}`, `--user-data-dir=${join(tmpdir(), `fyf-app-shots-${process.pid}`)}`,
    "--remote-allow-origins=*", "--hide-scrollbars", "--no-first-run", "--no-default-browser-check", "--disable-extensions",
    "--force-color-profile=srgb", "--font-render-hinting=none", `--window-size=${W},${H}`, "about:blank",
  ], { stdio: "ignore" });
  kids.push(chrome);
  if (!(await waitUp(`http://127.0.0.1:${CDP_PORT}/json/version`, 20000))) throw new Error("Chrome debug port did not open");
  const page = (await (await fetch(`http://127.0.0.1:${CDP_PORT}/json/list`)).json()).find((t) => t.type === "page");
  const cdp = await Cdp.connect(page.webSocketDebuggerUrl);

  let faults = [];
  cdp.handlers.push((m) => {
    if (m.method === "Runtime.exceptionThrown") faults.push(`exception: ${m.params.exceptionDetails.exception?.description ?? m.params.exceptionDetails.text}`);
    if (m.method === "Runtime.consoleAPICalled") {
      const text = m.params.args.map((a) => a.value ?? a.description ?? "").join(" ");
      if (/\[shots\]/.test(text)) faults.push(`backend: ${text}`);
    }
  });
  await cdp.send("Page.enable");
  await cdp.send("Runtime.enable");
  await cdp.send("Emulation.setDeviceMetricsOverride", { width: W, height: H, deviceScaleFactor: 1, mobile: false });
  await cdp.send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-color-scheme", value: "dark" }, { name: "prefers-reduced-motion", value: "reduce" }] });

  const failed = [];
  for (const lang of LANGS) {
    for (const s of SHOTS) {
      faults = [];
      const query = `lang=${lang}${s.query ? "&" + s.query : ""}`;
      await cdp.send("Page.navigate", { url: `${BASE}?${query}` });
      await cdp.waitFor(`document.querySelectorAll('.nav > button').length >= 8 && !!document.querySelector('.gauge')`, 30000, "the app");
      await cdp.eval(`document.querySelectorAll('.nav > button')[${s.nav}].click()`);
      if (s.tab !== undefined) {
        await cdp.waitFor(`document.querySelectorAll('.tabs button').length > ${s.tab}`, 10000, "the tabs");
        await cdp.eval(`document.querySelectorAll('.tabs button')[${s.tab}].click()`);
      }
      await cdp.waitFor(s.ready, 15000, s.stem);
      if (s.scroll) {
        await cdp.eval(`(() => { const m = document.querySelector('.main'); const el = ${s.scroll}; m.scrollTo(0, el.getBoundingClientRect().top - m.getBoundingClientRect().top + m.scrollTop - 14); })()`);
      } else {
        await cdp.eval(`document.querySelector('.main').scrollTo(0, 0)`);
      }
      await sleep(700);
      const problems = [...(await cdp.eval(CHECK)), ...faults];
      const stem = s.stem + (lang === "en" ? "" : `-${lang}`);
      if (problems.length) {
        failed.push(stem);
        console.log(`${stem}: REFUSED\n  ${problems.join("\n  ")}`);
        continue;
      }
      const shot = await cdp.send("Page.captureScreenshot", { format: "png" });
      writeFileSync(join(OUT, `${stem}.png`), Buffer.from(shot.data, "base64"));
      console.log(`${stem}: ok`);
    }
  }
  if (failed.length) { console.log(`REFUSED: ${failed.join(", ")}`); process.exit(1); }
  console.log(`all pictures in ${OUT}`);
  process.exit(0);
}

main().catch((e) => { console.error(e); process.exit(2); });
