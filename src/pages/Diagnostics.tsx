import { useEffect, useState } from "react";
import { api, EngineInfo } from "../api";
import { useApp } from "../hooks";
import { Button, Card } from "../ui";

export default function Diagnostics({ engine }: { engine: EngineInfo | null }) {
  const { t, toast } = useApp();
  const [diag, setDiag] = useState<Record<string, unknown> | null>(null);
  const [fg, setFg] = useState<Record<string, unknown> | null>(null);
  const [problems, setProblems] = useState<string[]>([]);
  const loadProblems = () => api.recentProblems().then(setProblems).catch(() => {});
  useEffect(() => { api.diagnostics().then(setDiag).catch(() => {}); loadProblems(); }, [engine]);

  return (
    <>
      <h1>{t("nav_diagnostics")}</h1>
      <Card title={t("problems_title")} actions={<Button onClick={loadProblems}>{t("refresh")}</Button>}>
        {problems.length === 0
          ? <p className="muted">{t("problems_empty")}</p>
          : <pre className="mono" style={{ whiteSpace: "pre-wrap", maxHeight: 320, overflow: "auto" }}>{problems.join("\n")}</pre>}
      </Card>
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
