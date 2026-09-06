import { useEffect, useState } from "react";
import { api, StatsSummary } from "../api";
import { useApp } from "../hooks";
import { Card, Stat } from "../ui";

export default function Stats() {
  const { t, toast } = useApp();
  const [s, setS] = useState<StatsSummary | null>(null);
  useEffect(() => { api.stats().then(setS).catch((e) => toast(String(e), "err")); }, []);
  if (!s) return <h1>{t("nav_stats")}</h1>;

  const mins = (m: number) => (m >= 120 ? `${(m / 60).toFixed(1)} ${t("hours")}` : `${m.toFixed(0)} ${t("minutes")}`);
  const days = s.daily.slice(-30);
  const max = Math.max(1, ...days.map((d) => d.words));

  return (
    <>
      <h1>{t("nav_stats")}</h1>
      <div className="grid4">
        <Stat label={t("total_words")} value={s.total_words.toLocaleString()} />
        <Stat label={t("total_minutes")} value={mins(s.total_minutes)} />
        <Stat label={t("avg_wpm")} value={s.avg_wpm.toFixed(0)} />
        <Stat label={t("dictations")} value={s.dictations.toLocaleString()} />
        <Stat label={t("time_saved")} value={mins(s.time_saved_minutes)} sub={`${t("formula")}: ${s.time_saved_formula}`} />
        <Stat label={t("streak")} value={String(s.current_streak)} sub={`${t("longest_streak")}: ${s.longest_streak}`} />
        <Stat label={t("corrections")} value={String(s.corrections_applied)} />
        <Stat label={`${t("retries")} / ${t("edits")}`} value={`${s.retries} / ${s.edits}`} />
        <Stat label={t("p50")} value={`${s.p50_latency_ms} ms`} />
        <Stat label={t("p95")} value={`${s.p95_latency_ms} ms`} />
      </div>
      <Card title={t("last_days")}>
        <div className="bars-chart" style={{ marginBottom: 26 }}>
          {days.map((d) => (
            <div key={d.day} style={{ height: `${Math.max(3, (d.words / max) * 100)}%` }} title={`${d.day}: ${d.words} ${t("words")}, ${d.dictations} ${t("dictations")}`}>
              <span>{d.day.slice(8)}</span>
            </div>
          ))}
        </div>
      </Card>
      {s.by_category.length > 0 && (
        <Card title={t("by_category")}>
          <table><tbody>{s.by_category.map(([c, n]) => <tr key={c}><td>{c}</td><td>{n}</td></tr>)}</tbody></table>
        </Card>
      )}
    </>
  );
}
