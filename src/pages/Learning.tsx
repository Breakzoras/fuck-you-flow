import { useEffect, useState } from "react";
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

  return (
    <>
      <h1>{t("nav_learning")}</h1>
      <Card actions={<>
        <Button onClick={() => setSettings({ ...settings, privacy: { ...settings.privacy, learning_enabled: !settings.privacy.learning_enabled } })}>
          {settings.privacy.learning_enabled ? t("pause_learning") : t("resume_learning")}
        </Button>
        <Button kind="danger" onClick={async () => { await api.deleteLearningData(); load(); }}>{t("delete_learning")}</Button>
      </>}>
        <p className="hint">{t("suggestions_empty")}</p>
        {pending.map((s) => (
          <div className="suggestion" key={s.id}>
            <div><strong>{s.wrong}</strong> → <strong>{s.correct}</strong> <Badge tone="warn">{t("evidence")}: {s.evidence_count}</Badge></div>
            <div className="hint">{s.reason}</div>
            <div className="row" style={{ marginTop: 8 }}>
              <Button kind="primary" onClick={() => act(s.id, "accept")}>{t("accept")}</Button>
              <Button onClick={() => act(s.id, "dismiss")}>{t("dismiss")}</Button>
              <Button kind="ghost" onClick={() => act(s.id, "ignore")}>{t("ignore_forever")}</Button>
            </div>
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
