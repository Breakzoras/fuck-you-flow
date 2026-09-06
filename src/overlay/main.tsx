import React, { useEffect, useRef, useState } from "react";
import ReactDOM from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import "./overlay.css";

type OverlayState =
  | "idle" | "starting" | "recording" | "hands_free" | "processing" | "cleaning" | "success"
  | "cancelled" | "no_speech" | "mic_unavailable" | "model_unavailable" | "offline" | "failed"
  | "target_changed" | "sensitive";

interface Payload {
  state: OverlayState;
  message?: string | null;
  preview?: string | null;
  can_retry: boolean;
  seconds: number;
}

const LABELS: Record<string, Record<OverlayState, string>> = {
  en: {
    idle: "Fuck You Flow",
    starting: "Starting",
    recording: "Listening",
    hands_free: "Listening",
    processing: "Transcribing",
    cleaning: "Cleaning up",
    success: "Done",
    cancelled: "Cancelled",
    no_speech: "No speech detected",
    mic_unavailable: "Microphone unavailable",
    model_unavailable: "Speech model not ready",
    offline: "Offline",
    failed: "Failed",
    target_changed: "Copied to clipboard",
    sensitive: "Sensitive field",
  },
  el: {
    idle: "Fuck You Flow",
    starting: "Ξεκινάει",
    recording: "Ακούω",
    hands_free: "Ακούω",
    processing: "Μετατρέπω",
    cleaning: "Καθαρίζω",
    success: "Έτοιμο",
    cancelled: "Ακυρώθηκε",
    no_speech: "Δεν άκουσα ομιλία",
    mic_unavailable: "Χωρίς μικρόφωνο",
    model_unavailable: "Το μοντέλο δεν είναι έτοιμο",
    offline: "Χωρίς σύνδεση",
    failed: "Απέτυχε",
    target_changed: "Στο πρόχειρο",
    sensitive: "Ευαίσθητο πεδίο",
  },
};

const RETRY_TITLE: Record<string, string> = { en: "Try again", el: "Ξαναπροσπάθησε" };

// The overlay has no settings of its own: it asks the app which language the
// dashboard uses. The app may not be ready when the window first loads, so keep
// asking for a few seconds, and ask again at the start of every dictation so a
// change in Settings shows up without a restart.
async function fetchPrefs(): Promise<{ lang: string; style: string } | null> {
  for (let attempt = 0; attempt < 40; attempt++) {
    try {
      const s = await invoke<{ general: { ui_language: string }; overlay: { style?: string } }>("get_settings");
      return { lang: s.general.ui_language in LABELS ? s.general.ui_language : "en", style: s.overlay.style ?? "full" };
    } catch {
      await new Promise((r) => setTimeout(r, 250));
    }
  }
  return null;
}

function Bars({ level, active }: { level: number; active: boolean }) {
  const history = useRef<number[]>(new Array(28).fill(0));
  const [, force] = useState(0);
  useEffect(() => {
    history.current.push(level);
    history.current.shift();
    force((n) => n + 1);
  }, [level]);
  return (
    <div className={"bars" + (active ? " active" : "")} aria-hidden="true">
      {history.current.map((v, i) => (
        <span key={i} style={{ transform: `scaleY(${Math.max(0.14, Math.min(1, Math.sqrt(v) * 1.3)).toFixed(3)})` }} />
      ))}
    </div>
  );
}

function Overlay() {
  const [p, setP] = useState<Payload>({ state: "idle", can_retry: false, seconds: 0 });
  const [level, setLevel] = useState(0);
  const [seconds, setSeconds] = useState(0);
  const [lang, setLang] = useState("en");
  const [style, setStyle] = useState("full");
  const applyPrefs = (pr: { lang: string; style: string } | null) => { if (pr) { setLang(pr.lang); setStyle(pr.style); } };

  useEffect(() => {
    fetchPrefs().then(applyPrefs);
    const un1 = listen<Payload>("lalia://overlay", (e) => {
      setP(e.payload);
      if (e.payload.state === "starting") fetchPrefs().then(applyPrefs);
      if (e.payload.state !== "recording" && e.payload.state !== "hands_free") setLevel(0);
    });
    const un2 = listen<{ level: number; seconds: number }>("lalia://level", (e) => {
      setLevel(e.payload.level);
      setSeconds(e.payload.seconds);
    });
    return () => {
      un1.then((f) => f());
      un2.then((f) => f());
    };
  }, []);

  const recording = p.state === "recording" || p.state === "hands_free";
  const busy = p.state === "processing" || p.state === "cleaning" || p.state === "starting";
  const error = ["failed", "mic_unavailable", "model_unavailable", "offline", "sensitive"].includes(p.state);
  const warn = ["no_speech", "cancelled", "target_changed"].includes(p.state);
  const cls = ["pill", p.state, recording ? "rec" : "", busy ? "busy" : "", error ? "err" : "", warn ? "warn" : ""].join(" ");

  if (style === "minimal") {
    // One dot with a coloured ring: red listening, amber working, green done.
    return (
      <div className={cls + " mini"} role="status" aria-live="polite" title={LABELS[lang][p.state]} data-tauri-drag-region>
        <div className="dot" />
      </div>
    );
  }

  return (
    <div className={cls} role="status" aria-live="polite" data-tauri-drag-region>
      <div className="head">
        <div className="dot" />
        <div className="title">{LABELS[lang][p.state]}{recording ? ` ${seconds.toFixed(1)}s` : ""}</div>
        {busy ? <div className="spinner" /> : null}
        {p.can_retry && (
          <button className="retry" onClick={() => invoke("pipeline_retry")} title={RETRY_TITLE[lang]}>
            ↻
          </button>
        )}
      </div>
      {recording ? <Bars level={level} active /> : null}
      {p.message && !recording && p.state !== "success" && <div className="sub">{p.message}</div>}
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Overlay />
  </React.StrictMode>,
);
