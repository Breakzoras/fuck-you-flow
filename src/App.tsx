import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow, UserAttentionType } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api, EngineInfo, PipelineSnapshot, Settings, UpdateInfo } from "./api";
import { AppCtx, makeT, makeTk } from "./hooks";
import Home from "./pages/Home";
import History from "./pages/History";
import Dictionary from "./pages/Dictionary";
import Snippets from "./pages/Snippets";
import Learning from "./pages/Learning";
import Stats from "./pages/Stats";
import SettingsPage from "./pages/SettingsPage";
import Diagnostics from "./pages/Diagnostics";
import GpuGauge from "./GpuGauge";
import ReportDialog from "./ReportDialog";
import { withTimeout } from "./async";
import { testBuildStamp } from "./testBuild";
import logoOnDark from "./assets/luram-logo-on-dark.png";
import logoOnLight from "./assets/luram-logo-on-light.png";

const LURAM_URL = "https://luram.gr/?utm_source=fuflow&utm_medium=app&utm_campaign=credits";
const GITHUB_URL = "https://github.com/Breakzoras/fuck-you-flow";
// Set by the local build script alone; a public build has none (see testBuild.ts).
const TEST_BUILD = testBuildStamp(import.meta.env.VITE_FYF_TEST_BUILD);

// The same rule the .deb installs, for the AppImage or a missed step.
const PERM_CMD = "printf '%s\\n' 'KERNEL==\"uinput\", SUBSYSTEM==\"misc\", TAG+=\"uaccess\", OPTIONS+=\"static_node=uinput\"' 'SUBSYSTEM==\"input\", KERNEL==\"event*\", ENV{ID_INPUT_KEYBOARD}==\"1\", TAG+=\"uaccess\"' 'SUBSYSTEM==\"input\", KERNEL==\"event*\", ENV{ID_INPUT_MOUSE}==\"1\", TAG+=\"uaccess\"' | sudo tee /etc/udev/rules.d/70-fuckyouflow.rules && sudo modprobe uinput && sudo udevadm control --reload-rules && sudo udevadm trigger";

type Page = "home" | "history" | "dictionary" | "snippets" | "learning" | "stats" | "settings" | "diagnostics";

export default function App() {
  const [settings, setSettingsState] = useState<Settings | null>(null);
  const [page, setPage] = useState<Page>("home");
  const [reportOpen, setReportOpen] = useState(false);
  const [engine, setEngine] = useState<EngineInfo | null>(null);
  const [snap, setSnap] = useState<PipelineSnapshot | null>(null);
  const [toastMsg, setToastMsg] = useState<{ msg: string; kind: "ok" | "err" } | null>(null);
  const [loadFailed, setLoadFailed] = useState(false);
  const [loadAttempt, setLoadAttempt] = useState(0);
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  // Linux: the keyboard permission comes from a udev rule. Without it the app
  // runs and Right Alt stays silent, so the missing permission is said here.
  const [inputMissing, setInputMissing] = useState(false);
  useEffect(() => {
    let alive = true;
    const poll = () => api.linuxInputStatus().then((s) => { if (alive) setInputMissing(s.linux && (!s.keyboard || !s.virtual_keyboard)); }).catch(() => {});
    poll();
    const iv = setInterval(poll, 5000);
    return () => { alive = false; clearInterval(iv); };
  }, []);
  // The big card in the middle. "Not now" closes it and leaves the bar on top.
  const [prompt, setPrompt] = useState(false);
  const [updating, setUpdating] = useState<{ received: number; total: number; installing?: boolean } | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoadFailed(false);
    // Bound startup recovery even when a backend call stalls.
    const load = async () => {
      const deadline = Date.now() + 15000;
      while (!cancelled && Date.now() < deadline) {
        try {
          const s = await withTimeout(api.getSettings(), deadline - Date.now());
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
    // The notepad's "do not show again" box writes the settings from the
    // program side. Without this the open Settings page would still hold the
    // old copy and put the window back on the next save.
    const un3 = listen("lalia://settings-changed", () => api.getSettings().then(setSettingsState).catch(() => {}));
    const iv = setInterval(() => api.snapshot().then(setSnap).catch(() => {}), 2000);
    return () => {
      cancelled = true;
      un1.then((f) => f());
      un2.then((f) => f());
      un3.then((f) => f());
      clearInterval(iv);
    };
  }, [loadAttempt]);

  useEffect(() => {
    if (!settings) return;
    const theme = settings.general.theme;
    const root = document.documentElement;
    const apply = () => {
      const dark = theme === "dark" || (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
      root.setAttribute("data-theme", dark ? "dark" : "light");
      root.setAttribute("data-skin", settings.general.skin === "classic" ? "classic" : "carbon");
      root.setAttribute("data-os", /linux/i.test(navigator.userAgent) ? "linux" : "other");
    };
    apply();
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    mq.addEventListener("change", apply);
    return () => mq.removeEventListener("change", apply);
  }, [settings?.general.theme, settings?.general.skin]);

  const toast = useCallback((msg: string, kind: "ok" | "err" = "ok") => {
    setToastMsg({ msg, kind });
    setTimeout(() => setToastMsg(null), 2600);
  }, []);

  const setSettings = useCallback(async (s: Settings) => {
    try {
      const saved = await api.saveSettings(s);
      setSettingsState(saved);
      // The engine may correct what it was given, for example when the user
      // picks the graphics card by hand. Returning the corrected object lets
      // the page match it, which is what stops the Save button staying lit
      // after a successful save.
      return saved;
    } catch (e) {
      // A rejected shortcut is reported by the page that owns the field, in the
      // user's language. Showing the raw line here as well would say the same
      // thing twice, once unreadably.
      if (!String(e).includes("bad_shortcut|")) {
        toast(String(e), "err");
      }
      throw e;
    }
  }, [toast]);

  const lang = settings?.general.ui_language ?? "en";
  const t = useMemo(() => makeT(lang), [lang]);
  const tk = useMemo(() => makeTk(lang), [lang]);
  // index.html says lang="en" for the first paint only. A screen reader picks
  // its pronunciation from this attribute, so a Greek interface left at "en"
  // was read aloud with English rules.
  useEffect(() => {
    document.documentElement.lang = lang;
  }, [lang]);

  // Ask the update server once, a few seconds after the app opens. Nothing is
  // downloaded here. Nothing newer: not a word. Something newer: the dashboard
  // comes to the front with the update in the middle of it (Lu, 11 September
  // 2026). The app usually starts hidden in the tray, so the bar alone sat in
  // a window nobody opened and installed copies stayed old for days.
  // The check that runs by itself shows the window and flashes its taskbar
  // button, and the keyboard stays where it was. Someone typing in another app
  // a few seconds after logon would otherwise send the next Space or Enter into
  // the card and start an install nobody asked for (pre-release review,
  // 11 September 2026). "Check for updates" in the tray is a request, so that
  // one takes the keyboard. A card that turns up during a dictation still waits
  // until the dictation has finished.
  const wantFront = useRef(false);
  const bringFront = useCallback(async (takeKeyboard: boolean) => {
    const w = getCurrentWindow();
    await w.unminimize().catch(() => {});
    await w.show().catch(() => {});
    if (takeKeyboard) await w.setFocus().catch(() => {});
    else await w.requestUserAttention(UserAttentionType.Informational).catch(() => {});
  }, []);
  const lookForUpdate = useCallback(async (manual: boolean) => {
    try {
      const u = await api.checkForUpdate();
      if (!u.available) {
        if (manual) toast(t("update_none"));
        return;
      }
      setUpdate(u);
      setPrompt(true);
      // The tray menu already brought the window up; a manual check is never mid-dictation.
      const now = manual ? null : await api.snapshot().catch(() => null);
      if (manual || now?.phase === "idle") await bringFront(manual);
      else wantFront.current = true;
    } catch (err) {
      if (manual) toast(`${t("update_failed")}: ${err}`, "err");
    }
  }, [t, toast, bringFront]);
  // The card's title holds the focus, so a stray Enter or Space presses nothing
  // and Escape still reaches the card. React's autoFocus moves the focus only
  // for form fields and buttons, so the title is focused here.
  const updateTitle = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    if (update && prompt) updateTitle.current?.focus();
  }, [update, prompt]);
  useEffect(() => {
    if (wantFront.current && snap?.phase === "idle") {
      wantFront.current = false;
      void bringFront(false);
    }
  }, [snap, bringFront]);
  // The listeners below live for the whole session; the ref hands them the
  // latest function, so the tray answer comes in the language chosen since.
  const lookRef = useRef(lookForUpdate);
  lookRef.current = lookForUpdate;

  useEffect(() => {
    const timer = setTimeout(() => { void lookRef.current(false); }, 8000);
    const un = listen<{ received: number; total: number; installing?: boolean }>("lalia://update-progress", (e) => setUpdating(e.payload));
    // "Check for updates" in the tray menu.
    const unCheck = listen("lalia://check-update", () => { void lookRef.current(true); });
    return () => { clearTimeout(timer); un.then((f) => f()); unCheck.then((f) => f()); };
  }, []);

  const startInstall = () => {
    setUpdating({ received: 0, total: 0 });
    api
      .installUpdate()
      // The installer normally ends this process, so the line below runs only
      // when it did not. Then the buttons come back and the card stays usable.
      .then(() => setUpdating(null))
      .catch((err) => {
        toast(String(err), "err");
        setUpdating(null);
      });
  };

  if (!settings) {
    return (
      <div className="main">
        <p className="muted">
          {loadFailed
            ? "Η εφαρμογή άργησε να απαντήσει. Δοκίμασε ξανά. / The app took too long to respond. Try again."
            : "Loading…"}
        </p>
        {loadFailed && <button onClick={() => setLoadAttempt((n) => n + 1)}>Δοκιμή ξανά / Try again</button>}
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
    <AppCtx.Provider value={{ settings, setSettings, lang, t, tk, toast }}>
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
            {TEST_BUILD && (
              <span className="test-build">
                <b>{t("test_build")}</b>
                {TEST_BUILD}
              </span>
            )}
          </div>
          {nav.map(([id, label]) => (
            <button key={id} className={page === id ? "active" : ""} onClick={() => setPage(id)}>{label}</button>
          ))}
          <div className="spacer" />
          {/* Asks for a star on GitHub until the card is pressed once. GitHub
              tells no program who starred, so the press is what counts. */}
          {!settings.general.rated_on_github && (
            <button
              type="button"
              className="rate-card"
              title={`${t("rate_body")} ${t("rate_note")}`}
              onClick={() => {
                openUrl(GITHUB_URL)
                  .then(() => api.markRatedOnGithub())
                  .then(setSettingsState)
                  .catch(() => {});
              }}
            >
              <span className="rate-head">
                {/* Thumbs up, after the Lucide icon (ISC licence). */}
                <svg className="rate-thumb" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                  <path d="M7 10v12" />
                  <path d="M15 5.88 14 10h5.83a2 2 0 0 1 1.92 2.56l-2.33 8A2 2 0 0 1 17.5 22H4a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2h2.76a2 2 0 0 0 1.79-1.11L12 2a3.13 3.13 0 0 1 3 3.88Z" />
                </svg>
                <b>{t("rate_title")}</b>
              </span>
              <span className="rate-note">{t("rate_note")}</span>
            </button>
          )}
          <div className={`status-chip ${snap?.phase === "recording" ? "recording" : engineClass}`} title={engine?.message ?? ""}>
            <span className="dot" />
            <span>{snap?.phase && snap.phase !== "idle" ? phaseLabel : engineLabel}</span>
          </div>
          <button type="button" className="report-open" onClick={() => setReportOpen(true)}>{t("report_open")}</button>
          {/* The maker's credit, on every page. The link carries utm tags so the
              agency site can count the visitors that came from the app. */}
          <button type="button" className="credit" title="luram.gr" onClick={() => { openUrl(LURAM_URL).catch(() => {}); }}>
            <span className="credit-by">Developed by<b>Luram AI Agency</b></span>
            <img className="credit-logo on-dark" src={logoOnDark} alt="Luram" />
            <img className="credit-logo on-light" src={logoOnLight} alt="Luram" />
          </button>
        </nav>
        <main className="main">
          {inputMissing && (
            <div className="update-bar" role="alert">
              <strong>{t("perm_title")}</strong>
              <span>{t("perm_body")}</span>
              <code className="perm-cmd">{PERM_CMD}</code>
              <button className="primary" onClick={() => { navigator.clipboard.writeText(PERM_CMD).then(() => toast(t("perm_copied"))).catch(() => {}); }}>{t("perm_copy")}</button>
            </div>
          )}
          {update && (
            <div className="update-bar" role="status">
              <strong>{t("update_title")}</strong>
              <span>{t("update_ready").replace("{v}", update.version).replace("{c}", update.current)}</span>
              {!update.small_download && <span className="muted">{t("update_big")}</span>}
              {!update.small_download ? (
                <>
                  <span className="grow" />
                  <button className="primary" onClick={() => { openUrl("https://fuckyouflow.app"); }}>{t("update_site")}</button>
                  <button onClick={() => setUpdate(null)}>{t("update_later")}</button>
                </>
              ) : updating ? (
                <span className="grow">
                  {updating.installing
                    ? t("update_installing")
                    : `${t("update_downloading")} ${updating.total > 0 ? Math.round((updating.received / updating.total) * 100) + "%" : ""}`}
                </span>
              ) : (
                <>
                  <span className="grow" />
                  <button className="primary" onClick={startInstall}>
                    {t("update_now")}
                  </button>
                  <button onClick={() => setUpdate(null)}>{t("update_later")}</button>
                </>
              )}
            </div>
          )}
          {update && prompt && (
            <div
              className="update-modal"
              role="dialog"
              aria-modal="true"
              aria-labelledby="update-modal-title"
              onKeyDown={(e) => { if (e.key === "Escape" && !updating) setPrompt(false); }}
            >
              <div className="update-card">
                <h2 id="update-modal-title" ref={updateTitle} tabIndex={-1}>{t("update_title")}</h2>
                <p className="update-lead">{t("update_ready").replace("{v}", update.version).replace("{c}", update.current)}</p>
                {update.notes && <p className="update-notes">{update.notes}</p>}
                {!update.small_download && <p className="muted">{t("update_big")}</p>}
                <div className="update-actions">
                  {updating ? (
                    <span className="update-progress">
                      {updating.installing
                        ? t("update_installing")
                        : `${t("update_downloading")} ${updating.total > 0 ? Math.round((updating.received / updating.total) * 100) + "%" : ""}`}
                    </span>
                  ) : (
                    <>
                      {update.small_download ? (
                        <button className="primary" onClick={startInstall}>{t("update_now")}</button>
                      ) : (
                        <button className="primary" onClick={() => { openUrl("https://fuckyouflow.app"); }}>{t("update_site")}</button>
                      )}
                      <button onClick={() => setPrompt(false)}>{t("update_later")}</button>
                    </>
                  )}
                </div>
              </div>
            </div>
          )}
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
        <GpuGauge />
        {reportOpen && <ReportDialog onClose={() => setReportOpen(false)} />}
        {toastMsg && <div className={`toast ${toastMsg.kind}`} role="status">{toastMsg.msg}</div>}
      </div>
    </AppCtx.Provider>
  );
}
