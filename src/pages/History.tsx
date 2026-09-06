import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, dayKey, emptyRule, fmtDate, HistoryEntry, Suggestion } from "../api";
import { useApp } from "../hooks";
import { Button, Card } from "../ui";

export default function History() {
  const { t, lang, toast, settings } = useApp();
  const [items, setItems] = useState<HistoryEntry[]>([]);
  const [q, setQ] = useState("");
  const [showRaw, setShowRaw] = useState<Record<string, boolean>>({});
  const [editing, setEditing] = useState<{ id: string; text: string } | null>(null);
  const [correction, setCorrection] = useState<{ id: string; wrong: string; correct: string } | null>(null);
  // Dictionary suggestions produced by the edit the user just saved.
  const [pending, setPending] = useState<Suggestion[]>([]);

  const load = () => api.history(q || undefined).then(setItems).catch((e) => toast(String(e), "err"));
  useEffect(() => { load(); }, [q]);
  useEffect(() => {
    const un = listen("lalia://history-changed", () => load());
    return () => { un.then((f) => f()); };
  }, [q]);

  const groups: Record<string, HistoryEntry[]> = {};
  for (const h of items) {
    const k = dayKey(h.created_at, lang);
    (groups[k] ||= []).push(h);
  }

  const copy = (text: string) => navigator.clipboard.writeText(text).then(() => toast(t("copy")));

  const saveEdit = async () => {
    if (!editing) return;
    try {
      const sugg = await api.recordEdit(editing.id, editing.text);
      // A clear word correction goes straight into the Dictionary; the user
      // sees what was learned and can delete the entry there if it is wrong.
      for (const sg of sugg) {
        try { await api.resolveSuggestion(sg.id, "accept"); } catch { /* keep going */ }
      }
      setPending([]);
      toast(sugg.length ? sugg.map((sg) => t("learned_rule", { w: sg.wrong, c: sg.correct })).join("  ·  ") : t("saved"));
      setEditing(null);
      load();
    } catch (e) { toast(String(e), "err"); }
  };

  const saveCorrection = async () => {
    if (!correction) return;
    try {
      await api.saveRule({ ...emptyRule(), wrong: correction.wrong.trim(), correct: correction.correct.trim() });
      toast(t("saved"));
      setCorrection(null);
    } catch (e) { toast(String(e), "err"); }
  };

  const onSelectWrong = (h: HistoryEntry) => {
    const sel = window.getSelection()?.toString().trim() ?? "";
    setCorrection({ id: h.id, wrong: sel, correct: "" });
  };

  return (
    <>
      <h1>{t("nav_history")}</h1>
      <div className="row" style={{ marginBottom: 14 }}>
        <input type="search" placeholder={t("search")} value={q} onChange={(e) => setQ(e.target.value)} style={{ maxWidth: 420 }} />
        <Button kind="danger" onClick={async () => { if (confirm(t("confirm_delete_history"))) { await api.deleteAllHistory(); load(); } }}>{t("delete_all")}</Button>
      </div>
      {pending.length > 0 && (
        <Card title={t("add_to_dictionary")}>
          {pending.map((s) => (
            <div className="row" key={s.id} style={{ alignItems: "center", gap: 12, marginBottom: 6 }}>
              <span><b>{s.wrong}</b> → <b>{s.correct}</b></span>
              <Button kind="primary" onClick={async () => { await api.resolveSuggestion(s.id, "accept"); setPending((p) => p.filter((x) => x.id !== s.id)); toast(t("saved")); }}>{t("accept")}</Button>
              <Button onClick={async () => { await api.resolveSuggestion(s.id, "dismiss"); setPending((p) => p.filter((x) => x.id !== s.id)); }}>{t("dismiss")}</Button>
            </div>
          ))}
        </Card>
      )}
      {!settings.privacy.keep_history && <p className="hint">{t("keep_history")}: off</p>}
      {items.length === 0 && <div className="empty">{t("no_history")}</div>}
      {Object.entries(groups).map(([day, list]) => (
        <div key={day}>
          <div className="history-day">{day}</div>
          {list.map((h) => (
            <div className="history-item" key={h.id}>
              {editing?.id === h.id ? (
                <div>
                  <textarea value={editing.text} onChange={(e) => setEditing({ id: h.id, text: e.target.value })} />
                  <div className="row" style={{ marginTop: 8 }}>
                    <Button kind="primary" onClick={saveEdit}>{t("save_edit")}</Button>
                    <Button onClick={() => setEditing(null)}>{t("cancel")}</Button>
                  </div>
                </div>
              ) : (
                <div className="history-text" onMouseUp={() => {}}>{h.edited_text ?? h.final_text}</div>
              )}
              <div className="history-meta">
                <span>{fmtDate(h.created_at, lang)}</span>
                {h.app_name && <span>{t("app_of")} {h.app_name}</span>}
                <span>{h.word_count} {t("words")}</span>
                <span>{(h.audio_ms / 1000).toFixed(1)}s</span>
                <span>{t("latency")} {h.latency_ms} ms</span>
                {h.detected_language && <span>{h.detected_language}</span>}
                <span className={`status-line status-${h.status}`}>{h.status}{h.retried ? " · retry" : ""}{h.context_used ? " · ctx" : ""}</span>
              </div>
              {showRaw[h.id] && (
                <div className="history-raw">
                  <div><strong>{t("raw")}:</strong> {h.raw_text}</div>
                  <div style={{ marginTop: 4 }}><strong>{t("cleaned")}:</strong> {h.cleaned_text}</div>
                  {h.rules_applied.length > 0 && <div style={{ marginTop: 4 }}>{h.rules_applied.join(" · ")}</div>}
                </div>
              )}
              {correction?.id === h.id && (
                <div className="inline-form" style={{ marginTop: 10 }}>
                  <input type="text" placeholder={t("wrong")} value={correction.wrong} onChange={(e) => setCorrection({ ...correction, wrong: e.target.value })} />
                  <input type="text" placeholder={t("correct")} value={correction.correct} onChange={(e) => setCorrection({ ...correction, correct: e.target.value })} />
                  <div className="row">
                    <Button kind="primary" onClick={saveCorrection} disabled={!correction.wrong || !correction.correct}>{t("save")}</Button>
                    <Button onClick={() => setCorrection(null)}>{t("cancel")}</Button>
                  </div>
                </div>
              )}
              <div className="history-actions">
                <Button onClick={() => copy(h.edited_text ?? h.final_text)}>{t("copy")}</Button>
                <Button onClick={() => api.pasteHistory(h.id)}>{t("paste")}</Button>
                <Button onClick={() => setShowRaw((s) => ({ ...s, [h.id]: !s[h.id] }))}>{t("raw")}</Button>
                <Button onClick={async () => { await api.recleanHistory(h.id); load(); }}>{t("reclean")}</Button>
                <Button onClick={() => onSelectWrong(h)}>{t("add_correction")}</Button>
                <Button onClick={() => setEditing({ id: h.id, text: h.edited_text ?? h.final_text })} title={t("edit_and_learn")}>{t("edit_and_learn")}</Button>
                <Button kind="danger" onClick={async () => { await api.deleteHistory(h.id); load(); }}>{t("delete")}</Button>
              </div>
            </div>
          ))}
        </div>
      ))}
      <Card><p className="hint">{t("add_correction")}: {lang === "el" ? "μάρκαρε με το ποντίκι τη λάθος λέξη στο κείμενο και πάτα το κουμπί." : "select the wrong word with the mouse, then press the button."}</p></Card>
    </>
  );
}
