import { useEffect, useRef, useState } from "react";
import { api } from "./api";
import { useApp } from "./hooks";

// A problem or an idea, sent to the maker from inside the app. Nothing leaves
// the machine until Send is pressed, and the diagnostic log only with its box
// ticked. The user can read the log before sending it.
export default function ReportDialog({ onClose }: { onClose: () => void }) {
  const { t, toast } = useApp();
  const [kind, setKind] = useState<"problem" | "idea">("problem");
  const [message, setMessage] = useState("");
  const [email, setEmail] = useState("");
  const [includeLog, setIncludeLog] = useState(true);
  const [preview, setPreview] = useState<string | null>(null);
  const [sending, setSending] = useState(false);
  const title = useRef<HTMLHeadingElement>(null);

  useEffect(() => { title.current?.focus(); }, []);

  const togglePreview = () => {
    if (preview !== null) { setPreview(null); return; }
    api.reportPreview().then(setPreview).catch((e) => toast(String(e), "err"));
  };

  const send = () => {
    if (!message.trim() || sending) return;
    setSending(true);
    api.sendReport(kind, message, email, includeLog)
      .then(() => { toast(t("report_sent")); onClose(); })
      .catch((e) => {
        const why = String(e);
        toast(t(why === "offline" ? "report_offline" : why === "too_many" ? "report_too_many" : "report_failed"), "err");
        setSending(false);
      });
  };

  return (
    <div className="update-modal" role="dialog" aria-modal="true" aria-labelledby="report-title"
      onKeyDown={(e) => { if (e.key === "Escape" && !sending) onClose(); }}>
      <div className="update-card report-card">
        <h2 id="report-title" ref={title} tabIndex={-1}>{t("report_title")}</h2>
        <div className="report-kind" role="radiogroup" aria-label={t("report_title")}>
          {(["problem", "idea"] as const).map((k) => (
            <button key={k} type="button" role="radio" aria-checked={kind === k} className={kind === k ? "on" : ""} onClick={() => setKind(k)}>
              {t(k === "problem" ? "report_kind_problem" : "report_kind_idea")}
            </button>
          ))}
        </div>
        <textarea className="report-text" rows={4} maxLength={5000} value={message} aria-label={t("report_title")}
          placeholder={t(kind === "problem" ? "report_message_problem" : "report_message_idea")}
          onChange={(e) => setMessage(e.target.value)} />
        <label className="report-field">
          <span>{t("report_email")}</span>
          <input type="email" value={email} maxLength={254} autoComplete="email" onChange={(e) => setEmail(e.target.value)} />
        </label>
        <label className="report-check">
          <input type="checkbox" checked={includeLog} onChange={(e) => setIncludeLog(e.target.checked)} />
          <span>{t("report_log")}<small>{t("report_log_hint")}</small></span>
        </label>
        {includeLog && (
          <button type="button" className="report-peek" onClick={togglePreview}>{t(preview === null ? "report_show" : "report_hide")}</button>
        )}
        {includeLog && preview !== null && <pre className="report-preview" tabIndex={0}>{preview}</pre>}
        <p className="muted report-where">{t("report_where")}</p>
        <div className="update-actions">
          <button className="primary" onClick={send} disabled={sending || !message.trim()}>{t(sending ? "report_sending" : "report_send")}</button>
          <button onClick={onClose} disabled={sending}>{t("report_cancel")}</button>
        </div>
      </div>
    </div>
  );
}
