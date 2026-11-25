import { useState } from "react";
import { useApp } from "./hooks";
import { StorageReport } from "./api";
import { cleanStorage, formatBytes, WORTH_CLEANING_BYTES } from "./storage";

/** What one press of "Clean up" deletes, in words, for the confirm box. */
export function describeItems(r: StorageReport, t: ReturnType<typeof useApp>["t"]): string {
  const parts: string[] = [];
  for (const i of r.items.filter((i) => i.kind === "model")) parts.push(t("disk_kind_model", { name: i.name }));
  if (r.items.some((i) => i.kind === "old_program")) parts.push(t("disk_kind_old_program"));
  if (r.items.some((i) => i.kind === "upgrade_copy")) parts.push(t("disk_kind_upgrade_copy"));
  return parts.join(", ");
}

/** The press itself, shared by the rail, Settings and the "What's new" card:
 *  asks first (naming what goes), then reports what was freed. */
export function useCleanAction() {
  const { t, lang, toast } = useApp();
  const [busy, setBusy] = useState(false);
  const run = async (r: StorageReport, ask = true): Promise<boolean> => {
    if (ask && !confirm(t("disk_confirm", { list: describeItems(r, t), size: formatBytes(r.cleanable_bytes, lang) }))) return false;
    setBusy(true);
    try {
      const res = await cleanStorage(r.items.map((i) => i.key));
      const size = formatBytes(res.freed_bytes, lang);
      toast(res.failed > 0 ? t("disk_partly", { size, n: res.failed }) : t("disk_done", { size }), res.failed > 0 && res.removed === 0 ? "err" : "ok");
      return true;
    } catch (err) {
      toast(String(err), "err");
      return false;
    } finally {
      setBusy(false);
    }
  };
  return { run, busy };
}

/** The block under the card memory in the right-hand rail. */
export function StorageRail({ report }: { report: StorageReport | null }) {
  const { t, lang } = useApp();
  const { run, busy } = useCleanAction();
  if (!report) return null;
  const worth = report.cleanable_bytes >= WORTH_CLEANING_BYTES;
  // The bar is the share of the app's own files that is left over.
  const pct = report.total_bytes > 0 ? Math.round((report.cleanable_bytes / report.total_bytes) * 100) : 0;
  const size = formatBytes(report.cleanable_bytes, lang);
  return (
    <section className="gauge-disk" aria-labelledby="gauge-disk-title">
      <h2 id="gauge-disk-title">{t("disk_title")}</h2>
      {worth ? (
        <>
          <div className="gauge-bar">
            <span className="gauge-bar-label">
              {t("disk_cleanable")}
              <span className="gauge-bar-note">{size}</span>
            </span>
            <div className="gauge-bar-track" role="img" aria-label={`${t("disk_cleanable")}: ${size}`}>
              <div className="warn" style={{ width: `${Math.max(2, Math.min(100, pct))}%` }} />
            </div>
          </div>
          <button className="btn primary gauge-switch" disabled={busy} onClick={() => { void run(report); }}>
            {busy ? t("disk_cleaning") : t("disk_clean")}
          </button>
        </>
      ) : (
        <p className="gauge-note">{t("disk_nothing")}</p>
      )}
      {report.models_busy && <p className="gauge-note">{t("disk_busy")}</p>}
    </section>
  );
}
