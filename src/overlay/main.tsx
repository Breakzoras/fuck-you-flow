import React, { useEffect, useRef, useState } from "react";
import { tk } from "../i18n";
import ReactDOM from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
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

// The extra line under the spinner, for the afternoon when a one second
// dictation turns into thirty because the card filled up with other programs.
//
// It needs two things to be true at once, and the second one is what keeps it
// honest: the wait has to be dragging, AND the gauge has to say the model no
// longer fits on the card. A machine with no card at all reads "ok" (see the
// gauge's fallback arm in commands.rs), so a slow transcription on the
// processor never gets blamed on graphics hardware that is not doing the work.
//
// Three seconds sits in an empty gap, measured over 160 real dictations on
// history.inference_ms, which is the column for the state this timer watches.
// The healthy group runs to 2,870 ms at its very slowest, 613 ms at the median.
// The broken group starts at 7,593 ms and reaches 19,924 ms, and holds 11 of
// the 160. Nothing at all lands in between, so the threshold catches the broken
// afternoons without ever firing on an ordinary long sentence.
const STRESS_AFTER_MS = 3000;

const STRESS_LINE: Record<string, string> = {
  en: "GPU is getting stressed",
  el: "Ζορίζεται η κάρτα γραφικών",
};

// The overlay has no settings of its own: it asks the app which language the
// dashboard uses. The app may not be ready when the window first loads, so keep
// asking for a few seconds, and ask again at the start of every dictation so a
// change in Settings shows up without a restart.
async function fetchPrefs(): Promise<{ lang: string; style: string; pos: string } | null> {
  for (let attempt = 0; attempt < 40; attempt++) {
    try {
      const s = await invoke<{ general: { ui_language: string }; overlay: { style?: string; position?: string } }>("get_settings");
      return { lang: s.general.ui_language in LABELS ? s.general.ui_language : "en", style: s.overlay.style ?? "full", pos: s.overlay.position ?? "bottom_center" };
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
  const [stressed, setStressed] = useState(false);
  const applyPrefs = (pr: { lang: string; style: string; pos: string } | null) => {
    if (pr) {
      setLang(pr.lang);
      setStyle(pr.style);
      // The badge hugs whichever edge the window is anchored to.
      document.body.setAttribute("data-pos", pr.pos);
    }
  };

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

  // Time the wait, then ask the card once. One question per slow dictation,
  // asked only after the threshold, so a fast machine never pays for it.
  useEffect(() => {
    if (p.state !== "processing") {
      setStressed(false);
      return;
    }
    const timer = setTimeout(() => {
      invoke<{ state: string }>("gpu_gauge")
        .then((g) => setStressed(g.state === "tight" || g.state === "spilled"))
        // Swallowing this silently would leave a dead feature with no trace to
        // follow when someone reports that the line never shows up. It reaches
        // the webview console rather than the diagnostics file, which is thin,
        // but it is the difference between a clue and nothing.
        .catch((e) => console.error("gpu_gauge failed, stress line suppressed", e));
    }, STRESS_AFTER_MS);
    return () => clearTimeout(timer);
  }, [p.state]);

  const recording = p.state === "recording" || p.state === "hands_free";
  const busy = p.state === "processing" || p.state === "cleaning" || p.state === "starting";
  const error = ["failed", "mic_unavailable", "model_unavailable", "offline", "sensitive"].includes(p.state);
  const warn = ["no_speech", "cancelled", "target_changed"].includes(p.state);
  const cls = ["pill", p.state, recording ? "rec" : "", busy ? "busy" : "", error ? "err" : "", warn ? "warn" : ""].join(" ");
  // The state guard covers the one render between leaving "processing" and the
  // effect above clearing the flag, so the line never flashes over "Done".
  // The window is wider and taller than the badge now, so its empty area would
  // sit on top of whatever the user is trying to click. Windows can be told to
  // pass the pointer straight through, and that is switched off only while the
  // retry button is on screen and waiting to be pressed.
  useEffect(() => {
    getCurrentWindow().setIgnoreCursorEvents(!p.can_retry).catch(() => {});
  }, [p.can_retry]);

  const stress = stressed && p.state === "processing" ? STRESS_LINE[lang] : null;

  if (style === "minimal") {
    // One dot with a coloured ring: red listening, amber working, green done.
    // The dot has no room for a line, so the warning rides in the tooltip.
    return (
      <div
        className={cls + " mini"}
        role="status"
        aria-live="polite"
        title={LABELS[lang][p.state] + (stress ? ` · ${stress}` : "")}
        data-tauri-drag-region
      >
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
      {stress ? <div className="stress">{stress}</div> : null}
      {recording ? <Bars level={level} active /> : null}
      {p.message && !recording && <div className="sub">{tk(lang, p.message)}</div>}
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Overlay />
  </React.StrictMode>,
);
