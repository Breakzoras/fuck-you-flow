import { useEffect, useRef, useState } from "react";
import { api, EngineInfo } from "../api";
import { useApp } from "../hooks";
import { Button, Card } from "../ui";

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
