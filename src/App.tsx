import { useCallback, useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, EngineInfo, PipelineSnapshot, Settings } from "./api";
import { AppCtx, makeT } from "./hooks";
import Home from "./pages/Home";
import History from "./pages/History";
import Dictionary from "./pages/Dictionary";
import Snippets from "./pages/Snippets";
import Learning from "./pages/Learning";
import Stats from "./pages/Stats";
import SettingsPage from "./pages/SettingsPage";
import Diagnostics from "./pages/Diagnostics";

type Page = "home" | "history" | "dictionary" | "snippets" | "learning" | "stats" | "settings" | "diagnostics";

export default function App() {
  const [settings, setSettingsState] = useState<Settings | null>(null);
  const [page, setPage] = useState<Page>("home");
  const [engine, setEngine] = useState<EngineInfo | null>(null);
  const [snap, setSnap] = useState<PipelineSnapshot | null>(null);
  const [toastMsg, setToastMsg] = useState<{ msg: string; kind: "ok" | "err" } | null>(null);
  const [loadFailed, setLoadFailed] = useState(false);

  useEffect(() => {
    let cancelled = false;
    // The page loads before the backend has registered its state, so the first
    // calls can be rejected. Keep asking for a few seconds instead of staying on
    // the placeholder forever.
    const load = async () => {
      for (let attempt = 0; attempt < 60 && !cancelled; attempt++) {
        try {
          const s = await api.getSettings();
          if (cancelled) return;
          setSettingsState(s);
          api.engineInfo().then(setEngine).catch(() => {});
          return;
        } catch {
          await new Promise((r) => setTimeout(r, 250));
        }
      }
      if (!cancelled) setLoadFailed(true);
    };
    load();
    const un1 = listen<EngineInfo>("lalia://engine", (e) => setEngine(e.payload));
    const un2 = listen("lalia://overlay", () => api.snapshot().then(setSnap).catch(() => {}));
    const iv = setInterval(() => api.snapshot().then(setSnap).catch(() => {}), 2000);
    return () => {
      cancelled = true;
      un1.then((f) => f());
      un2.then((f) => f());
      clearInterval(iv);
    };
  }, []);

  useEffect(() => {
    if (!settings) return;
    const theme = settings.general.theme;
    const root = document.documentElement;
    const apply = () => {
      const dark = theme === "dark" || (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
      root.setAttribute("data-theme", dark ? "dark" : "light");
    };
    apply();
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    mq.addEventListener("change", apply);
    return () => mq.removeEventListener("change", apply);
  }, [settings?.general.theme]);

  const toast = useCallback((msg: string, kind: "ok" | "err" = "ok") => {
    setToastMsg({ msg, kind });
    setTimeout(() => setToastMsg(null), 2600);
  }, []);

  const setSettings = useCallback(async (s: Settings) => {
    try {
      const saved = await api.saveSettings(s);
      setSettingsState(saved);
    } catch (e) {
      toast(String(e), "err");
      throw e;
    }
  }, [toast]);

  const lang = settings?.general.ui_language ?? "el";
  const t = useMemo(() => makeT(lang), [lang]);

  if (!settings) {
    return (
      <div className="main">
        <p className="muted">
          {loadFailed
            ? "The app did not respond. Close this window and open it again from the tray icon."
            : "Loading…"}
        </p>
      </div>
    );
  }

  const nav: [Page, string][] = [
    ["home", t("nav_home")],
    ["history", t("nav_history")],
    ["dictionary", t("nav_dictionary")],
    ["snippets", t("nav_snippets")],
    ["learning", t("nav_learning")],
    ["stats", t("nav_stats")],
    ["settings", t("nav_settings")],
    ["diagnostics", t("nav_diagnostics")],
  ];

  const engineClass = engine?.status ?? "missing";
  const engineLabel =
    engine?.status === "ready" ? `${t("engine_ready")} (${engine.gpu ? t("gpu_on") : t("gpu_off")})`
    : engine?.status === "starting" ? t("engine_starting")
    : engine?.status === "failed" ? t("engine_failed")
    : t("engine_missing");
  const phaseLabel = snap?.phase === "recording" ? t("status_recording") : snap?.phase === "processing" ? t("status_processing") : t("status_idle");

  return (
    <AppCtx.Provider value={{ settings, setSettings, lang, t, toast }}>
      <div className="layout">
        <nav className="nav" aria-label="Main">
          <div className="brand">
            <div className="brand-mark" aria-hidden="true">
              <i style={{ height: 10 }} /><i style={{ height: 18 }} /><i style={{ height: 24 }} /><i style={{ height: 14 }} />
            </div>
            <span className="brand-name">
              {t("app").split(" ").map((w, i) => (
                <span key={i} className={i === 1 ? "accent" : ""}>{w}{i < t("app").split(" ").length - 1 ? " " : ""}</span>
              ))}
            </span>
          </div>
          {nav.map(([id, label]) => (
            <button key={id} className={page === id ? "active" : ""} onClick={() => setPage(id)}>{label}</button>
          ))}
          <div className="spacer" />
          <div className={`status-chip ${snap?.phase === "recording" ? "recording" : engineClass}`} title={engine?.message ?? ""}>
            <span className="dot" />
            <span>{snap?.phase && snap.phase !== "idle" ? phaseLabel : engineLabel}</span>
          </div>
        </nav>
        <main className="main">
          <div className="page">
            {page === "home" && <Home engine={engine} snap={snap} goSettings={() => setPage("settings")} />}
            {page === "history" && <History />}
            {page === "dictionary" && <Dictionary />}
            {page === "snippets" && <Snippets />}
            {page === "learning" && <Learning />}
            {page === "stats" && <Stats />}
            {page === "settings" && <SettingsPage engine={engine} />}
            {page === "diagnostics" && <Diagnostics engine={engine} />}
          </div>
        </main>
        {toastMsg && <div className={`toast ${toastMsg.kind}`} role="status">{toastMsg.msg}</div>}
      </div>
    </AppCtx.Provider>
  );
}
