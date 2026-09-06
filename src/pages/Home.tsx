import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, DownloadProgress, EngineInfo, fmtBytes, HistoryEntry, ModelStatus, PipelineSnapshot, RuntimeStatus, StatsSummary } from "../api";
import { useApp } from "../hooks";
import { Badge, Button, Card } from "../ui";

// Home in the Mono style: a state word, a few lines of facts, the last
// transcripts as a log. Setup only appears while something is missing.
export default function Home({ engine, snap, goSettings }: { engine: EngineInfo | null; snap: PipelineSnapshot | null; goSettings: () => void }) {
  const { settings, setSettings, t, tk, toast } = useApp();
  const [runtime, setRuntime] = useState<RuntimeStatus | null>(null);
  const [models, setModels] = useState<ModelStatus[]>([]);
  const [progress, setProgress] = useState<Record<string, DownloadProgress>>({});
  const [busy, setBusy] = useState<string | null>(null);
  const [recent, setRecent] = useState<HistoryEntry[]>([]);
  const [stats, setStats] = useState<StatsSummary | null>(null);

  const refresh = () => {
    api.runtimeStatus().then(setRuntime).catch(() => {});
    api.models().then(setModels).catch(() => {});
  };
  const refreshLog = () => {
    api.history(undefined).then((h) => setRecent(h.slice(0, 6))).catch(() => {});
    api.stats().then(setStats).catch(() => {});
  };
  useEffect(() => {
    refresh();
    refreshLog();
    const un = listen<DownloadProgress>("lalia://download", (e) => {
      setProgress((p) => ({ ...p, [e.payload.id]: e.payload }));
      if (e.payload.phase === "done" || e.payload.phase === "error") setTimeout(refresh, 300);
    });
    const un2 = listen("lalia://history-changed", () => refreshLog());
    return () => { un.then((f) => f()); un2.then((f) => f()); };
  }, []);

  const current = models.find((m) => m.id === settings.asr.model_id);
  const setupDone = !!runtime?.installed && !!current?.installed && !!runtime?.vad_model;
  const showSetup = !settings.general.first_run_done || !setupDone;

  const run = async (id: string, fn: () => Promise<unknown>) => {
    setBusy(id);
    try {
      await fn();
      toast(t("saved"));
    } catch (e) {
      toast(String(e), "err");
    } finally {
      setBusy(null);
      refresh();
    }
  };

  const pct = (id: string) => {
    const p = progress[id];
    if (!p || p.phase === "done") return null;
    const v = p.total > 0 ? Math.round((p.received / p.total) * 100) : 0;
    return { v, phase: p.phase, msg: p.message };
  };

  const ProgressBar = ({ id }: { id: string }) => {
    const p = pct(id);
    if (!p) return null;
    return (
      <div>
        <span className="hint">{p.phase === "verifying" ? t("verifying") : p.phase === "error" ? `${t("error")}: ${p.msg}` : `${t("downloading")} ${p.v}%`}</span>
        <div className="progress"><div style={{ transform: `scaleX(${(p.v / 100).toFixed(3)})` }} /></div>
      </div>
    );
  };

  const hk = settings.hotkeys;
  const state =
    snap?.phase === "recording" ? t("status_recording")
    : snap?.phase === "processing" ? t("status_processing")
    : engine?.status === "ready" ? t("status_ready")
    : engine?.status === "starting" ? t("engine_starting")
    : t("engine_missing");
  const engineLine = engine?.status === "ready"
    ? <><b>{engine.model_id}</b> {engine.gpu ? t("gpu_on") : t("gpu_off")}{engine.warm_ms ? `, ${t("warm_in", { s: (engine.warm_ms / 1000).toFixed(1) })}` : ""}</>
    : <span>{engine?.message ?? state}</span>;
  const today = stats?.daily?.length ? stats.daily[stats.daily.length - 1] : null;
  const fmtTime = (iso: string) => { const d = new Date(iso); return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`; };
  const statusClass = (s: string) => (s === "success" ? "ok" : s === "copied" ? "warn" : "err");

  return (
    <>
      <h1>{state.toLowerCase()}</h1>
      {showSetup && (
        <Card title={t("setup_title")}>
          <div className="setup-item">
            <div>
              <strong>{t("setup_runtime")}</strong>
              <span className="hint">{runtime?.spec.version} · {fmtBytes(runtime?.spec.size_bytes ?? 0)} · {t("cuda_driver")}: {runtime?.cuda_driver ? t("present") : t("absent")}</span>
              <ProgressBar id="whisper-runtime" />
            </div>
            {runtime?.installed ? <Badge tone="ok">{t("installed")}</Badge> : (
              <Button kind="primary" disabled={busy !== null} onClick={() => run("rt", () => api.installRuntime())}>{t("install")}</Button>
            )}
          </div>
          <div className="setup-item">
            <div>
              <strong>{t("setup_model")}: {current?.display_name ?? settings.asr.model_id}</strong>
              <span className="hint">{current ? `${fmtBytes(current.size_bytes)} · ${t("vram")} ~${current.vram_mb} MB · ${tk(current.notes_key)}` : ""}</span>
              <ProgressBar id={settings.asr.model_id} />
            </div>
            {current?.installed ? <Badge tone="ok">{t("installed")}</Badge> : (
              <Button kind="primary" disabled={busy !== null} onClick={() => run("m", () => api.downloadModel(settings.asr.model_id))}>{t("download")}</Button>
            )}
          </div>
          <div className="setup-item">
            <div>
              <strong>{t("setup_vad")}</strong>
              <span className="hint">Silero VAD · 0.9 MB</span>
              <ProgressBar id="silero-vad" />
            </div>
            {runtime?.vad_model ? <Badge tone="ok">{t("installed")}</Badge> : (
              <Button kind="primary" disabled={busy !== null} onClick={() => run("v", () => api.downloadModel("silero-vad"))}>{t("download")}</Button>
            )}
          </div>
          <div className="setup-item">
            <div>
              <strong>{t("setup_mic")}</strong>
              <span className="hint">{settings.audio.device_name ?? t("system_default")}</span>
            </div>
            <Button onClick={goSettings}>{t("nav_settings")}</Button>
          </div>
          {setupDone && !settings.general.first_run_done && (
            <div className="row" style={{ marginTop: 12 }}>
              <Button kind="primary" onClick={async () => {
                await setSettings({ ...settings, general: { ...settings.general, first_run_done: true } });
                await api.engineRestart();
              }}>{t("finish_setup")}</Button>
            </div>
          )}
          {setupDone && settings.general.first_run_done && engine?.status !== "ready" && (
            <div className="row" style={{ marginTop: 12 }}>
              <Button kind="primary" onClick={() => api.engineRestart()}>{t("restart_engine")}</Button>
            </div>
          )}
        </Card>
      )}

      <div className="facts">
        <div className="fact"><span className="k">{t("f_engine")}</span><span className="v">{engineLine}</span></div>
        <div className="fact"><span className="k">{t("f_mic")}</span><span className="v">{settings.audio.device_name ?? t("system_default")}{snap?.mic_open ? "" : ` · ${t("mic_closed")}`}</span></div>
        <div className="fact"><span className="k">{t("f_key")}</span><span className="v"><span className="kbd">{hk.hands_free}</span> {t("f_key_start")} · <span className="kbd">{hk.hands_free}</span> {t("f_key_stop")} · <span className="kbd">Esc</span> {t("f_key_cancel")}</span></div>
        <div className="fact"><span className="k">{t("f_today")}</span><span className="v">
          {today ? <><b>{today.words}</b> {t("words")} · {t("median_wait")} <b>{stats && stats.p50_latency_ms ? (stats.p50_latency_ms / 1000).toFixed(2) + " s" : "-"}</b> · <b>{stats ? Math.round(stats.time_saved_minutes) : 0} min</b> {t("saved_short")}</> : <span className="muted">{t("no_history")}</span>}
        </span></div>
      </div>

      <div className="row" style={{ margin: "14px 0 26px" }}>
        {snap?.phase === "recording" ? (
          <>
            <Button kind="primary" onClick={() => api.toggle()}>{t("stop_dictation")}</Button>
            <Button kind="danger" onClick={() => api.cancel()}>{t("cancel")}</Button>
          </>
        ) : (
          <Button kind="primary" onClick={() => api.toggle()} disabled={engine?.status !== "ready"}>{t("test_dictation")}</Button>
        )}
        <Button onClick={() => api.pasteLast()}>{t("paste_last")}</Button>
      </div>

      <h2>{t("last_transcripts")}</h2>
      <div className="log" style={{ marginTop: 8 }}>
        {recent.length === 0 && <div className="muted">{t("no_history")}</div>}
        {recent.map((h) => (
          <div key={h.id}>
            <span className="t">{fmtTime(h.created_at)}</span>{" "}
            <span className={statusClass(h.status)}>{h.status === "success" ? "ok" : h.status}</span>{" "}
            <span className="t">{h.word_count}w {(h.latency_ms / 1000).toFixed(2)}s</span>{"  "}
            <span className="body">{(h.edited_text ?? h.final_text).slice(0, 110)}{(h.edited_text ?? h.final_text).length > 110 ? "…" : ""}</span>
          </div>
        ))}
      </div>
    </>
  );
}
