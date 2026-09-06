import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, AppStyle, DeviceInfo, DownloadProgress, emptyStyle, EngineInfo, fmtBytes, ModelStatus, RuntimeStatus, Settings, LanguageMode } from "../api";
import { useApp } from "../hooks";
import { Badge, Button, Card, Field, Select, Toggle } from "../ui";

type Tab = "general" | "mic" | "language" | "models" | "shortcuts" | "cleanup" | "insertion" | "overlay" | "privacy" | "styles" | "cloud";

export default function SettingsPage({ engine }: { engine: EngineInfo | null }) {
  const { settings, setSettings, t, toast } = useApp();
  const [tab, setTab] = useState<Tab>("general");
  const [draft, setDraft] = useState<Settings>(settings);
  useEffect(() => setDraft(settings), [settings]);

  const patch = (p: (s: Settings) => Settings) => setDraft((d) => p(structuredClone(d)));
  const save = async () => {
    try {
      await setSettings(draft);
      toast(t("saved"));
    } catch { /* toast shown by ctx */ }
  };
  const dirty = JSON.stringify(draft) !== JSON.stringify(settings);

  const tabs: [Tab, string][] = [
    ["general", t("s_general")], ["mic", t("s_mic")], ["language", t("s_language")], ["models", t("s_models")], ["shortcuts", t("s_shortcuts")],
    ["cleanup", t("s_cleanup")], ["insertion", t("s_insertion")], ["overlay", t("s_overlay")], ["privacy", t("s_privacy")], ["styles", t("s_styles")], ["cloud", t("s_cloud")],
  ];

  return (
    <>
      <div className="row" style={{ justifyContent: "space-between" }}>
        <h1>{t("nav_settings")}</h1>
        <Button kind="primary" onClick={save} disabled={!dirty}>{t("save")}</Button>
      </div>
      <div className="tabs">{tabs.map(([id, label]) => <button key={id} className={tab === id ? "active" : ""} onClick={() => setTab(id)}>{label}</button>)}</div>

      {tab === "general" && (
        <Card>
          <Field label={t("theme")}>
            <Select value={draft.general.theme} onChange={(v) => patch((s) => { s.general.theme = v; return s; })} options={[{ value: "system", label: t("theme_system") }, { value: "dark", label: t("theme_dark") }, { value: "light", label: t("theme_light") }]} />
          </Field>
          <Toggle label={t("autostart")} checked={draft.general.autostart} onChange={(v) => patch((s) => { s.general.autostart = v; return s; })} />
        </Card>
      )}

      {tab === "mic" && <MicTab draft={draft} patch={patch} />}

      {tab === "language" && (
        <Card title={t("lang_section")}>
          <Field label={t("ui_language")} hint={t("lang_ui_hint")}>
            <Select value={draft.general.ui_language} onChange={(v) => patch((s) => { s.general.ui_language = v; return s; })} options={[{ value: "en", label: "English" }, { value: "el", label: "Ελληνικά" }]} />
          </Field>
          <Field label={t("lang_mode")} hint={t("lang_mode_hint")}>
            <Select value={draft.language.mode} onChange={(v) => patch((s) => { s.language.mode = v; return s; })} options={dictationOptions(draft.general.ui_language, t)} />
          </Field>
        </Card>
      )}

      {tab === "models" && <ModelsTab draft={draft} patch={patch} engine={engine} />}

      {tab === "shortcuts" && <ShortcutsTab draft={draft} patch={patch} />}

      {tab === "cleanup" && (
        <Card>
          <Field label={t("intensity")}>
            <Select value={draft.cleanup.intensity} onChange={(v) => patch((s) => { s.cleanup.intensity = v; return s; })} options={[
              { value: "off", label: t("int_off") }, { value: "light", label: t("int_light") }, { value: "normal", label: t("int_normal") }, { value: "strong", label: t("int_strong") },
            ]} />
          </Field>
          <Toggle label={t("remove_fillers")} checked={draft.cleanup.remove_fillers} onChange={(v) => patch((s) => { s.cleanup.remove_fillers = v; return s; })} />
          <Toggle label={t("self_corr")} checked={draft.cleanup.resolve_self_corrections} onChange={(v) => patch((s) => { s.cleanup.resolve_self_corrections = v; return s; })} />
          <Toggle label={t("intonation_q")} hint={t("intonation_q_hint")} checked={draft.cleanup.intonation_questions ?? true} onChange={(v) => patch((s) => { s.cleanup.intonation_questions = v; return s; })} />
          <Toggle label={t("auto_punct")} checked={draft.cleanup.auto_punctuate} onChange={(v) => patch((s) => { s.cleanup.auto_punctuate = v; return s; })} />
          <Toggle label={t("auto_cap")} checked={draft.cleanup.auto_capitalize} onChange={(v) => patch((s) => { s.cleanup.auto_capitalize = v; return s; })} />
          <Toggle label={t("hints")} checked={draft.asr.hints_from_dictionary} onChange={(v) => patch((s) => { s.asr.hints_from_dictionary = v; return s; })} />
        </Card>
      )}

      {tab === "insertion" && (
        <Card>
          <Field label={t("ins_method")}>
            <Select value={draft.insertion.method} onChange={(v) => patch((s) => { s.insertion.method = v; return s; })} options={[
              { value: "auto", label: t("ins_auto") }, { value: "paste", label: t("ins_paste") }, { value: "type", label: t("ins_type") }, { value: "copy_only", label: t("ins_copy") },
            ]} />
          </Field>
          <Toggle label={t("restore_clip")} checked={draft.insertion.restore_clipboard} onChange={(v) => patch((s) => { s.insertion.restore_clipboard = v; return s; })} />
          <Toggle label={t("trailing_space")} checked={draft.insertion.trailing_space} onChange={(v) => patch((s) => { s.insertion.trailing_space = v; return s; })} />
          <Field label={t("settle")}>
            <input type="number" min={40} max={2000} value={draft.insertion.paste_settle_ms} onChange={(e) => patch((s) => { s.insertion.paste_settle_ms = Number(e.target.value); return s; })} style={{ maxWidth: 160 }} />
          </Field>
        </Card>
      )}

      {tab === "overlay" && (
        <Card>
          <Field label={t("position")}>
            <Select value={draft.overlay.position} onChange={(v) => patch((s) => { s.overlay.position = v; return s; })} options={[
              { value: "bottom_center", label: t("pos_bc") }, { value: "top_center", label: t("pos_tc") }, { value: "bottom_right", label: t("pos_br") }, { value: "bottom_left", label: t("pos_bl") },
            ]} />
          </Field>
          <Field label={t("overlay_style")}>
            <Select value={draft.overlay.style ?? "full"} onChange={(v) => patch((s) => { s.overlay.style = v; return s; })} options={[{ value: "full", label: t("ov_full") }, { value: "minimal", label: t("ov_minimal") }]} />
          </Field>
          <Toggle label={t("hide_idle")} checked={draft.overlay.hide_when_idle} onChange={(v) => patch((s) => { s.overlay.hide_when_idle = v; return s; })} />
        </Card>
      )}

      {tab === "privacy" && <PrivacyTab draft={draft} patch={patch} />}

      {tab === "styles" && <StylesTab />}

      {tab === "cloud" && <CloudTab draft={draft} patch={patch} />}
    </>
  );
}

function MicTab({ draft, patch }: { draft: Settings; patch: (p: (s: Settings) => Settings) => void }) {
  const { t, toast } = useApp();
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [level, setLevel] = useState(0);
  const [open, setOpen] = useState(false);
  useEffect(() => {
    api.microphones().then(setDevices).catch(() => {});
    const iv = setInterval(() => api.micLevel().then((m) => { setLevel(m.level); setOpen(m.open && m.alive); }).catch(() => {}), 100);
    return () => clearInterval(iv);
  }, []);
  return (
    <Card>
      <Field label={t("device")}>
        <Select value={draft.audio.device_name ?? ""} onChange={(v) => patch((s) => { s.audio.device_name = v || null; return s; })} options={[
          { value: "", label: t("system_default") }, ...devices.map((d) => ({ value: d.name, label: d.name + (d.is_default ? " ★" : "") })),
        ]} />
      </Field>
      <div className="row" style={{ margin: "12px 0" }}>
        <Button onClick={() => api.micTestOpen().then(() => toast(t("saved"))).catch((e) => toast(String(e), "err"))}>{t("mic_test")}</Button>
        <Badge tone={open ? "ok" : "warn"}>{open ? t("installed") : "-"}</Badge>
      </div>
      <div className="meter" aria-label={t("level")}><div style={{ transform: `scaleX(${Math.min(1, Math.sqrt(level)).toFixed(3)})` }} /></div>
      <Toggle label={t("keep_warm")} hint={t("keep_warm_note")} checked={draft.audio.keep_stream_warm} onChange={(v) => patch((s) => { s.audio.keep_stream_warm = v; return s; })} />
      <div className="grid2">
        <Field label={t("preroll")}><input type="number" min={0} max={1000} value={draft.audio.preroll_ms} onChange={(e) => patch((s) => { s.audio.preroll_ms = Number(e.target.value); return s; })} /></Field>
        <Field label={t("min_speech")}><input type="number" min={0} max={2000} value={draft.audio.min_speech_ms} onChange={(e) => patch((s) => { s.audio.min_speech_ms = Number(e.target.value); return s; })} /></Field>
      </div>
    </Card>
  );
}

/// The dictation languages, most useful first for whoever is reading the menu.
/// Someone running the app in English is dictating English and has no reason to
/// meet Greek at the top of a list; someone running it in Greek almost always
/// mixes English words into Greek sentences, so the bilingual mode leads there.
function dictationOptions(uiLanguage: string, t: (k: "lang_multi" | "lang_greek" | "lang_english" | "lang_auto") => string): { value: LanguageMode; label: string }[] {
  const multi = { value: "multi" as const, label: t("lang_multi") };
  const greek = { value: "greek" as const, label: t("lang_greek") };
  const english = { value: "english" as const, label: t("lang_english") };
  const auto = { value: "auto" as const, label: t("lang_auto") };
  return uiLanguage === "el" ? [multi, greek, english, auto] : [english, auto, multi, greek];
}

function ModelsTab({ draft, patch, engine }: { draft: Settings; patch: (p: (s: Settings) => Settings) => void; engine: EngineInfo | null }) {
  const { t, tk, toast } = useApp();
  const [models, setModels] = useState<ModelStatus[]>([]);
  const [runtime, setRuntime] = useState<RuntimeStatus | null>(null);
  const [progress, setProgress] = useState<Record<string, DownloadProgress>>({});
  const refresh = () => { api.models().then(setModels).catch(() => {}); api.runtimeStatus().then(setRuntime).catch(() => {}); };
  useEffect(() => {
    refresh();
    const un = listen<DownloadProgress>("lalia://download", (e) => {
      setProgress((p) => ({ ...p, [e.payload.id]: e.payload }));
      if (e.payload.phase === "done" || e.payload.phase === "error") setTimeout(refresh, 300);
    });
    return () => { un.then((f) => f()); };
  }, []);
  const pr = (id: string) => {
    const p = progress[id];
    if (!p || p.phase === "done") return null;
    const v = p.total > 0 ? Math.round((p.received / p.total) * 100) : 0;
    return <div><span className="hint">{p.phase === "verifying" ? t("verifying") : p.phase === "error" ? `${t("error")}: ${p.message}` : `${t("downloading")} ${v}%`}</span><div className="progress"><div style={{ transform: `scaleX(${(v / 100).toFixed(3)})` }} /></div></div>;
  };

  const machine = (runtime as unknown as { machine?: { gpus?: { name: string; vram_mb: number }[]; logical_cores?: number; vulkan_runtime?: boolean; cuda_driver?: boolean } } | null)?.machine;
  const gpus = machine?.gpus ?? [];
  const onGpu = engine?.gpu ?? false;

  return (
    <>
      <Card
        title={t("setup_runtime")}
        actions={<Button onClick={() => api.engineRestart().then(() => toast(t("saved")))}>{t("restart_engine")}</Button>}
      >
        <p className="hint" style={{ marginTop: 0 }}>{t("engine_what")}</p>

        <p>
          <strong>{t("engine_running_on")} {onGpu ? t("engine_on_gpu") : t("engine_on_cpu")}</strong>
          {engine ? <> · {engine.model_id} · {engine.status}{engine.warm_ms ? ` · ${engine.warm_ms} ms` : ""}</> : null}
          {engine?.message ? <> · {engine.message}</> : null}
        </p>
        <p className="hint">
          {t("machine")}: {gpus.length ? gpus.map((g) => `${g.name} (${g.vram_mb} MB)`).join(", ") : t("no_gpu")}
          {" · "}{machine?.logical_cores} {t("cores")}
          {" · Vulkan "}{machine?.vulkan_runtime ? "✓" : "✗"}
          {" · CUDA "}{machine?.cuda_driver ? "✓" : "✗"}
        </p>

        <h3 style={{ margin: "18px 0 6px", fontSize: 15, opacity: 0.75 }}>{t("engine_gpu_heavy")}</h3>
        <Field label={t("backend")}>
          <Select value={draft.asr.backend ?? "auto"} onChange={(v) => patch((s) => { s.asr.backend = v; return s; })} options={[
            { value: "auto", label: t("backend_auto") }, { value: "vulkan", label: t("backend_vulkan") }, { value: "cuda", label: t("backend_cuda") }, { value: "cpu", label: t("backend_cpu") },
          ]} />
        </Field>
        <Toggle label={t("use_gpu")} checked={draft.asr.use_gpu} onChange={(v) => patch((s) => { s.asr.use_gpu = v; return s; })} />

        <h3 style={{ margin: "18px 0 6px", fontSize: 15, opacity: 0.75 }}>{t("engine_cpu_heavy")}</h3>
        <div className="grid2">
          <Field label={t("threads")} hint={t("threads_hint")}><input type="number" min={1} max={32} value={draft.asr.threads} onChange={(e) => patch((s) => { s.asr.threads = Number(e.target.value); return s; })} /></Field>
          <Field label={t("beam")} hint={t("beam_hint")}><input type="number" min={1} max={8} value={draft.asr.beam_size} onChange={(e) => patch((s) => { s.asr.beam_size = Number(e.target.value); return s; })} /></Field>
        </div>
        <Toggle label={t("vad")} checked={draft.asr.vad} onChange={(v) => patch((s) => { s.asr.vad = v; return s; })} hint={runtime?.vad_model ? t("installed") : undefined} />
        {!runtime?.vad_model && <Button onClick={() => api.downloadModel("silero-vad").catch((e) => toast(String(e), "err"))}>{t("download")} Silero VAD</Button>}
        {pr("silero-vad")}

        <p className="hint" style={{ marginTop: 18 }}>
          {t("runtime_version")}: {runtime?.spec.version} · {t("size")}: {fmtBytes(runtime?.spec.size_bytes ?? 0)}
          {" · "}{runtime?.installed ? <Badge tone="ok">{t("installed")}</Badge> : <Button kind="primary" onClick={() => api.installRuntime().catch((e) => toast(String(e), "err"))}>{t("install")}</Button>}
        </p>
        {pr("whisper-runtime")}
      </Card>

      <Card title={t("s_models")}>
        <p className="hint" style={{ marginTop: 0 }}>{t("models_what")}</p>
        <table>
          <thead><tr><th></th><th>{t("size")}</th><th>{t("vram")}</th><th></th><th></th></tr></thead>
          <tbody>
            {models.map((m) => (
              <tr key={m.id}>
                <td><strong>{m.display_name}</strong>{m.recommended && <> <Badge tone="ok">{t("recommended")}</Badge></>}<br /><span className="hint">{tk(m.languages_key)} · {tk(m.notes_key)}</span>{pr(m.id)}</td>
                <td>{fmtBytes(m.size_bytes)}</td>
                <td>~{m.vram_mb} MB</td>
                <td>{m.installed ? <Badge tone={m.verified ? "ok" : "warn"}>{t("installed")}{m.verified ? " ✓" : ""}</Badge> : <Badge>-</Badge>}</td>
                <td>
                  <div className="row" style={{ gap: 4 }}>
                    {!m.installed && <Button kind="primary" onClick={() => api.downloadModel(m.id).catch((e) => toast(String(e), "err"))}>{t("download")}</Button>}
                    {m.installed && draft.asr.model_id !== m.id && <Button onClick={() => patch((s) => { s.asr.model_id = m.id; return s; })}>{t("use_this")}</Button>}
                    {draft.asr.model_id === m.id && <Badge tone="ok">{t("in_use")}</Badge>}
                    {m.installed && <Button onClick={() => api.verifyModel(m.id).then((ok) => { toast(ok ? "✓" : "✗", ok ? "ok" : "err"); refresh(); })}>{t("verify")}</Button>}
                    {m.installed && draft.asr.model_id !== m.id && <Button kind="danger" onClick={() => api.removeModel(m.id).then(refresh)}>{t("remove")}</Button>}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </Card>
    </>
  );
}

function ShortcutsTab({ draft, patch }: { draft: Settings; patch: (p: (s: Settings) => Settings) => void }) {
  const { t, toast } = useApp();
  const [recording, setRecording] = useState<string | null>(null);
  const rec = async (key: "push_to_talk" | "hands_free" | "paste_last") => {
    setRecording(key);
    try {
      const chord = await api.recordShortcut();
      patch((s) => { s.hotkeys[key] = chord; return s; });
    } catch (e) { toast(String(e), "err"); } finally { setRecording(null); }
  };
  const Row = ({ k, label }: { k: "push_to_talk" | "hands_free" | "paste_last"; label: string }) => (
    <Field label={label}>
      <div className="row">
        <input type="text" value={draft.hotkeys[k]} onChange={(e) => patch((s) => { s.hotkeys[k] = e.target.value; return s; })} style={{ maxWidth: 260 }} />
        <Button onClick={() => rec(k)} disabled={recording !== null}>{recording === k ? t("listening_keys") : t("record_shortcut")}</Button>
      </div>
    </Field>
  );
  return (
    <Card>
      <Row k="push_to_talk" label={t("ptt")} />
      <Row k="hands_free" label={t("hf")} />
      <Row k="paste_last" label={t("pl")} />
      <Toggle label={t("tap_toggle")} checked={draft.hotkeys.tap_toggles_hands_free} onChange={(v) => patch((s) => { s.hotkeys.tap_toggles_hands_free = v; return s; })} />
      <p className="hint">Ctrl, Shift, Alt, Win, RCtrl, RShift, RAlt, F1-F24, CapsLock, ScrollLock, Pause, A-Z, 0-9. Escape = {t("cancel")}.</p>
    </Card>
  );
}

function PrivacyTab({ draft, patch }: { draft: Settings; patch: (p: (s: Settings) => Settings) => void }) {
  const { t, toast } = useApp();
  const exportAll = async () => {
    const json = await api.exportAll();
    const blob = new Blob([json], { type: "application/json" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = `lalia-export-${new Date().toISOString().slice(0, 10)}.json`;
    a.click();
  };
  return (
    <Card>
      <Toggle label={t("keep_history")} checked={draft.privacy.keep_history} onChange={(v) => patch((s) => { s.privacy.keep_history = v; return s; })} />
      <Field label={t("retention")}>
        <input type="number" min={1} value={draft.privacy.retention_days ?? ""} onChange={(e) => patch((s) => { s.privacy.retention_days = e.target.value ? Number(e.target.value) : null; return s; })} style={{ maxWidth: 160 }} />
      </Field>
      <Toggle label={t("keep_audio")} checked={draft.privacy.keep_audio} onChange={(v) => patch((s) => { s.privacy.keep_audio = v; return s; })} />
      <Toggle label={t("context")} checked={draft.privacy.context_awareness} onChange={(v) => patch((s) => { s.privacy.context_awareness = v; return s; })} />
      <Toggle label={t("learning")} checked={draft.privacy.learning_enabled} onChange={(v) => patch((s) => { s.privacy.learning_enabled = v; return s; })} />
      <Toggle label={t("redact")} checked={draft.privacy.redact_logs} onChange={(v) => patch((s) => { s.privacy.redact_logs = v; return s; })} />
      <div className="row" style={{ marginTop: 14 }}>
        <Button onClick={exportAll}>{t("export_all")}</Button>
        <Button onClick={() => api.openDataFolder()}>{t("open_folder")}</Button>
        <Button kind="danger" onClick={async () => { if (confirm(t("confirm_delete_all"))) { await api.deleteAll(); toast(t("saved")); } }}>{t("delete_everything")}</Button>
      </div>
    </Card>
  );
}

function StylesTab() {
  const { t, toast } = useApp();
  const [items, setItems] = useState<AppStyle[]>([]);
  const [form, setForm] = useState<AppStyle>(emptyStyle());
  const load = () => api.appStyles().then(setItems).catch(() => {});
  useEffect(() => { load(); }, []);
  const cats = ["chat", "email", "document", "code", "ai_chat", "browser", "terminal", "unknown"];
  return (
    <>
      <Card title={form.id ? t("save") : t("add_style")}>
        <div className="grid2">
          <Field label={t("style_name")}><input type="text" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></Field>
          <Field label={t("style_process")}><input type="text" value={form.process_match} onChange={(e) => setForm({ ...form, process_match: e.target.value })} placeholder="telegram" /></Field>
          <Field label={t("style_category")}>
            <Select value={form.category} onChange={(v) => setForm({ ...form, category: v })} options={cats.map((c) => ({ value: c, label: c }))} />
          </Field>
        </div>
        <Toggle label={t("style_trailing")} checked={form.trailing_punctuation} onChange={(v) => setForm({ ...form, trailing_punctuation: v })} />
        <Toggle label={t("style_cap")} checked={form.capitalize_first} onChange={(v) => setForm({ ...form, capitalize_first: v })} />
        <div className="row">
          <Button kind="primary" onClick={() => api.saveAppStyle(form).then(() => { setForm(emptyStyle()); toast(t("saved")); load(); }).catch((e) => toast(String(e), "err"))} disabled={!form.name && !form.process_match}>{t("save")}</Button>
          {form.id && <Button onClick={() => setForm(emptyStyle())}>{t("cancel")}</Button>}
        </div>
      </Card>
      <Card>
        {items.length === 0 && <div className="empty">-</div>}
        {items.length > 0 && (
          <table>
            <tbody>
              {items.map((s) => (
                <tr key={s.id}><td><strong>{s.name}</strong></td><td>{s.process_match}</td><td>{s.category}</td><td>{s.trailing_punctuation ? "." : "-"} {s.capitalize_first ? "Aa" : "aa"}</td>
                  <td><div className="row" style={{ gap: 4 }}><Button onClick={() => setForm(s)}>✎</Button><Button kind="danger" onClick={() => api.deleteAppStyle(s.id).then(load)}>✕</Button></div></td></tr>
              ))}
            </tbody>
          </table>
        )}
      </Card>
    </>
  );
}

function CloudTab({ draft, patch }: { draft: Settings; patch: (p: (s: Settings) => Settings) => void }) {
  const { t, toast } = useApp();
  const [key, setKey] = useState("");
  const [has, setHas] = useState(false);
  useEffect(() => { api.hasCloudKey().then(setHas).catch(() => {}); }, []);
  return (
    <Card>
      <Field label={t("provider")}>
        <Select value={draft.asr.provider} onChange={(v) => patch((s) => { s.asr.provider = v; return s; })} options={[{ value: "whisper_local", label: t("prov_local") }, { value: "openai_compatible", label: t("prov_cloud") }]} />
      </Field>
      <Field label={t("base_url")}><input type="text" value={draft.asr.openai_base_url} onChange={(e) => patch((s) => { s.asr.openai_base_url = e.target.value; return s; })} /></Field>
      <Field label={t("cloud_model")}><input type="text" value={draft.asr.openai_model} onChange={(e) => patch((s) => { s.asr.openai_model = e.target.value; return s; })} /></Field>
      <Field label={t("api_key")}>
        <div className="row">
          <input type="password" value={key} onChange={(e) => setKey(e.target.value)} style={{ maxWidth: 420 }} />
          <Button onClick={() => api.setCloudKey(key).then(() => { toast(t("key_saved")); setKey(""); api.hasCloudKey().then(setHas); }).catch((e) => toast(String(e), "err"))}>{t("save")}</Button>
          <Badge tone={has ? "ok" : "warn"}>{has ? t("key_present") : t("key_absent")}</Badge>
        </div>
      </Field>
    </Card>
  );
}
