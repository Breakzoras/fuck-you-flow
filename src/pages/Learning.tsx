import { KeyboardEvent, useEffect, useState } from "react";
import { api, Suggestion } from "../api";
import { useApp } from "../hooks";
import { Badge, Button, Card } from "../ui";

export default function Learning() {
  const { t, toast, settings, setSettings } = useApp();
  const [items, setItems] = useState<Suggestion[]>([]);

  const load = () => api.suggestions().then(setItems).catch((e) => toast(String(e), "err"));
  useEffect(() => { load(); }, []);

  const pending = items.filter((s) => s.status === "pending");
  const resolved = items.filter((s) => s.status !== "pending");

  const act = async (id: string, action: "accept" | "dismiss" | "ignore") => {
    try {
      await api.resolveSuggestion(id, action);
      toast(t("saved"));
      load();
    } catch (e) { toast(String(e), "err"); }
  };

  // A suggestion that is nearly right: correct either side, then accept it.
  const [editing, setEditing] = useState<{ id: string; wrong: string; correct: string } | null>(null);
  const canSave = !!editing && !!editing.wrong.trim() && !!editing.correct.trim() && editing.wrong.trim() !== editing.correct.trim();
  const saveEdited = async () => {
    if (!editing || !canSave) return;
    try {
      await api.acceptSuggestionAs(editing.id, editing.wrong, editing.correct);
      toast(t("learned_rule", { w: editing.wrong.trim(), c: editing.correct.trim() }));
      setEditing(null);
      load();
    } catch (e) { toast(String(e), "err"); }
  };
  const onEditKey = (ev: KeyboardEvent) => {
    if (ev.key === "Enter") saveEdited();
    if (ev.key === "Escape") setEditing(null);
  };

  return (
    <>
      <h1>{t("nav_learning")}</h1>
      <Card actions={<>
        <Button onClick={() => setSettings({ ...settings, privacy: { ...settings.privacy, learning_enabled: !settings.privacy.learning_enabled } })}>
          {settings.privacy.learning_enabled ? t("pause_learning") : t("resume_learning")}
        </Button>
        <Button kind="danger" onClick={async () => { await api.deleteLearningData(); load(); }}>{t("delete_learning")}</Button>
      </>}>
        {pending.length === 0 && <p className="hint">{t("suggestions_empty")}</p>}
        {pending.map((s) => (
          <div className="suggestion" key={s.id}>
            <div><strong>{s.wrong}</strong> → <strong>{s.correct}</strong> <Badge tone="warn">{t("evidence")}: {s.evidence_count}</Badge></div>
            <div className="hint">{s.reason}</div>
            {editing?.id === s.id ? (
              <div className="inline-form" style={{ marginTop: 8 }}>
                <label>
                  <span className="hint">{t("wrong")}</span>
                  <input type="text" value={editing.wrong} onKeyDown={onEditKey} onChange={(e) => setEditing({ ...editing, wrong: e.target.value })} />
                </label>
                <label>
                  <span className="hint">{t("correct")}</span>
                  <input type="text" autoFocus value={editing.correct} onKeyDown={onEditKey} onChange={(e) => setEditing({ ...editing, correct: e.target.value })} />
                </label>
                <div className="row">
                  <Button kind="primary" onClick={saveEdited} disabled={!canSave}>{t("accept_edited")}</Button>
                  <Button onClick={() => setEditing(null)}>{t("cancel")}</Button>
                </div>
              </div>
            ) : (
              <div className="row" style={{ marginTop: 8 }}>
                <Button kind="primary" onClick={() => act(s.id, "accept")}>{t("accept")}</Button>
                <Button onClick={() => setEditing({ id: s.id, wrong: s.wrong, correct: s.correct })}>{t("edit_suggestion")}</Button>
                <Button onClick={() => act(s.id, "dismiss")}>{t("dismiss")}</Button>
                <Button kind="ghost" onClick={() => act(s.id, "ignore")}>{t("ignore_forever")}</Button>
              </div>
            )}
          </div>
        ))}
      </Card>
      {resolved.length > 0 && (
        <Card title="—">
          <table>
            <tbody>
              {resolved.map((s) => (
                <tr key={s.id}><td>{s.wrong}</td><td>{s.correct}</td><td><Badge>{s.status}</Badge></td><td className="hint">{s.reason}</td></tr>
              ))}
            </tbody>
          </table>
        </Card>
      )}
    </>
  );
}
