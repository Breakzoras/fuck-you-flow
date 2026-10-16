import { useEffect, useRef, useState } from "react";
import { api, EngineInfo, SeenKey } from "../api";
import { useApp } from "../hooks";
import { Button, Card } from "../ui";

/** HH:MM:SS.mmm from Unix milliseconds, for the key table. */
function clockMs(ms: number) {
  const d = new Date(ms);
  const p = (n: number, w = 2) => String(n).padStart(w, "0");
  return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}.${p(d.getMilliseconds(), 3)}`;
}

/**
 * The one confusion the table can name by itself: the other Alt arrived. A
 * bare tap of the wrong Alt looks exactly like "the hotkey does nothing", and
 * on 10 September 2026 it took forty minutes of log reading to tell that the
 * right Alt had stopped reaching Windows while the left one kept coming.
 * Returns [seen, bound] as "LAlt"/"RAlt", or null when there is nothing to say.
 */
function wrongAlt(keys: SeenKey[], handsFree: string): [string, string] | null {
  const bound = handsFree.split("+").find((k) => k === "LAlt" || k === "RAlt");
  if (!bound) return null;
  const other = bound === "RAlt" ? "LAlt" : "RAlt";
  const since = Date.now() - 20_000;
  const recent = keys.filter((k) => k.at_ms >= since && k.down && !k.injected);
  const sawBound = recent.some((k) => k.key === bound);
  const sawOther = recent.filter((k) => k.key === other).length;
  return !sawBound && sawOther >= 2 ? [other, bound] : null;
}

/** Live view of the modifier keys Windows delivers, so "the key does nothing" can be checked in one glance. */
function KeyCheck() {
  const { t, settings } = useApp();
  const [keys, setKeys] = useState<SeenKey[]>([]);

  useEffect(() => {
    let alive = true;
    const tick = () => api.recentKeys().then((k) => { if (alive) setKeys(k); }).catch(() => {});
    tick();
    const id = window.setInterval(tick, 400);
    return () => { alive = false; window.clearInterval(id); };
  }, []);

  const side = (k: string) => (k === "LAlt" ? t("key_left") : t("key_right"));
  const mismatch = wrongAlt(keys, settings.hotkeys.hands_free);
  const shown = keys.slice(0, 20);

  return (
    <Card title={t("key_check_title")}>
      <p className="muted" style={{ marginTop: 0 }}>{t("key_check_hint")}</p>
      <p className="mono" style={{ margin: "4px 0 10px" }}>
        {t("key_check_bound", { hf: settings.hotkeys.hands_free, ptt: settings.hotkeys.push_to_talk })}
      </p>
      {mismatch && (
        <p style={{ color: "var(--warn)", fontWeight: 600, margin: "0 0 10px" }}>
          {t("key_check_wrong_alt", { seen: side(mismatch[0]), bound: side(mismatch[1]) })}
        </p>
      )}
      {shown.length === 0
        ? <p className="muted">{t("key_check_none")}</p>
        : (
          <table className="mono" style={{ borderCollapse: "collapse", fontSize: 14 }}>
            <tbody>
              {shown.map((k, i) => (
                <tr key={`${k.at_ms}-${i}`} style={{ borderTop: "1px solid var(--line)" }}>
                  <td style={{ padding: "3px 10px 3px 0", whiteSpace: "nowrap", opacity: 0.7 }}>{clockMs(k.at_ms)}</td>
                  <td style={{ padding: "3px 10px 3px 0", whiteSpace: "nowrap", fontWeight: k.down ? 600 : 400 }}>{k.key}</td>
                  <td style={{ padding: "3px 10px 3px 0", whiteSpace: "nowrap" }}>{k.down ? t("key_down") : t("key_up")}</td>
                  <td style={{ padding: "3px 0", whiteSpace: "nowrap", opacity: 0.7 }}>{k.injected ? t("key_software") : ""}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
    </Card>
  );
}

/** One journal line, as written by src-tauri/src/journal.rs. */
type Event = { at: string; level: string; kind: string; d: Record<string, unknown> };

function parse(lines: string[]): Event[] {
  const out: Event[] = [];
  for (const l of lines) {
    try {
      out.push(JSON.parse(l) as Event);
    } catch {
      // a torn last line while the app is writing: skip it
    }
  }
  return out;
}

/** Time only: the day is almost always today and the column stays narrow. */
function clock(at: string) {
  const m = at.match(/T(\d{2}:\d{2}:\d{2})/);
  return m ? m[1] : at;
}

function summarise(d: Record<string, unknown>) {
  return Object.entries(d)
    .filter(([, v]) => v !== null && v !== "")
    .map(([k, v]) => `${k}=${typeof v === "object" ? JSON.stringify(v) : String(v)}`)
    .join("  ");
}

export default function Diagnostics({ engine }: { engine: EngineInfo | null }) {
  const { t, toast } = useApp();
  const [diag, setDiag] = useState<Record<string, unknown> | null>(null);
  const [fg, setFg] = useState<Record<string, unknown> | null>(null);
  const [problems, setProblems] = useState<string[]>([]);
  const [debug, setDebug] = useState(false);
  const [events, setEvents] = useState<Event[]>([]);
  const [onlyProblems, setOnlyProblems] = useState(false);
  const clicks = useRef<number[]>([]);

  const loadProblems = () => api.recentProblems().then(setProblems).catch(() => {});
  const loadEvents = () => api.debugEvents(300).then((l) => setEvents(parse(l))).catch(() => {});

  useEffect(() => {
    api.diagnostics().then(setDiag).catch(() => {});
    loadProblems();
    api.debugModeGet().then((on) => { setDebug(on); if (on) loadEvents(); }).catch(() => {});
  }, [engine]);

  // Debug mode has no button anywhere: five clicks on the title within three
  // seconds turn it on. It stays on across restarts until switched off here.
  const titleClick = () => {
    const now = Date.now();
    clicks.current = [...clicks.current, now].filter((c) => now - c < 3000);
    if (clicks.current.length < 5) return;
    clicks.current = [];
    const next = !debug;
    api.debugModeSet(next).then(() => {
      setDebug(next);
      toast(next ? "Debug mode on" : "Debug mode off");
      if (next) loadEvents();
    }).catch(() => {});
  };

  const copyBundle = () =>
    api.debugBundle().then(async (text) => {
      try {
        await navigator.clipboard.writeText(text);
        toast("Debug bundle copied, and saved to the logs folder");
      } catch {
        toast("Debug bundle saved to the logs folder");
      }
    }).catch(() => {});

  const shown = onlyProblems ? events.filter((e) => e.level !== "info") : events;

  return (
    <>
      <h1 onClick={titleClick} style={{ cursor: "default", userSelect: "none" }}>{t("nav_diagnostics")}</h1>

      <KeyCheck />

      <Card title={t("problems_title")} actions={<Button onClick={loadProblems}>{t("refresh")}</Button>}>
        {problems.length === 0
          ? <p className="muted">{t("problems_empty")}</p>
          : <pre className="mono" style={{ whiteSpace: "pre-wrap", maxHeight: 320, overflow: "auto" }}>{problems.join("\n")}</pre>}
      </Card>

      {debug && (
        <Card
          title="Debug"
          actions={<>
            <Button onClick={loadEvents}>{t("refresh")}</Button>
            <Button onClick={() => setOnlyProblems((v) => !v)}>{onlyProblems ? "All events" : "Problems only"}</Button>
            <Button onClick={copyBundle}>Copy debug bundle</Button>
            <Button kind="danger" onClick={() => api.debugModeSet(false).then(() => { setDebug(false); toast("Debug mode off"); })}>Turn off</Button>
          </>}
        >
          <p className="muted" style={{ marginTop: 0 }}>
            Every event is written to <code>events-YYYY-MM-DD.jsonl</code> in the logs folder, newest first here.
            Transcript text is never recorded, only counts and window names.
          </p>
          {shown.length === 0
            ? <p className="muted">Nothing recorded yet.</p>
            : (
              <div style={{ maxHeight: 420, overflow: "auto" }}>
                <table className="mono" style={{ width: "100%", borderCollapse: "collapse", fontSize: 14 }}>
                  <tbody>
                    {shown.map((e, i) => (
                      <tr key={i} style={{ borderTop: "1px solid var(--line)" }}>
                        <td style={{ padding: "4px 8px", whiteSpace: "nowrap", opacity: 0.7 }}>{clock(e.at)}</td>
                        <td style={{ padding: "4px 8px", whiteSpace: "nowrap", color: e.level === "info" ? undefined : "var(--danger, #ff6b6b)" }}>{e.kind}</td>
                        <td style={{ padding: "4px 8px", wordBreak: "break-word" }}>{summarise(e.d)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
        </Card>
      )}

      <Card actions={<>
        <Button onClick={() => api.openLogsFolder()}>{t("open_logs")}</Button>
        <Button onClick={() => api.openDataFolder()}>{t("open_folder")}</Button>
        <Button onClick={() => api.engineRestart().then(() => toast(t("saved")))}>{t("restart_engine")}</Button>
        <Button kind="danger" onClick={() => api.quit()}>{t("quit")}</Button>
      </>}>
        <pre className="mono" style={{ whiteSpace: "pre-wrap" }}>{JSON.stringify(diag, null, 2)}</pre>
      </Card>

      <Card actions={<Button onClick={() => setTimeout(() => api.foregroundApp().then(setFg), 3000)}>foreground app (3s)</Button>}>
        <pre className="mono" style={{ whiteSpace: "pre-wrap" }}>{fg ? JSON.stringify(fg, null, 2) : "-"}</pre>
      </Card>
    </>
  );
}
