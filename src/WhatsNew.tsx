import { useEffect, useRef, useState } from "react";
import { useApp } from "./hooks";
import { api } from "./api";
import clipEn from "./assets/whats-new/storage.webm";
import clipEl from "./assets/whats-new/storage-el.webm";
import still from "./assets/whats-new/storage-still.jpg";
import { WHATS_NEW_ID } from "./whatsNewRule";
import { formatBytes, useStorageReport, WORTH_CLEANING_BYTES } from "./storage";
import { useCleanAction } from "./StorageClean";

/** The "What's new" window of 0.9.23: the "Disk space" meter in the right
 *  rail and its "Clean up" button. Shown once to people who updated (a new
 *  install starts with it seen). It shows this machine's own number and can
 *  clean from here; the button asks first, like the one in the rail.
 *
 *  The clip is a 10 second mockup made with HyperFrames
 *  (videos/fyf-storage-whatsnew, Greek copy in fyf-storage-whatsnew-el),
 *  muted and looping; with reduced motion asked for, a still of its last
 *  card shows instead. Same window, same style as the 0.9.19 one
 *  (appmic, dictating through Claude and Codex). */
export default function WhatsNew({ onClose }: { onClose: () => void }) {
  const { lang, t } = useApp();
  const clip = lang === "el" ? clipEl : clipEn;
  const [never, setNever] = useState(false);
  const title = useRef<HTMLHeadingElement>(null);
  const still_only = useRef(window.matchMedia?.("(prefers-reduced-motion: reduce)").matches ?? false);
  const { report } = useStorageReport(false);
  const { run, busy } = useCleanAction();
  const worth = !!report && report.cleanable_bytes >= WORTH_CLEANING_BYTES;

  // The title holds the focus, as on the update card: a stray Enter or Space
  // typed for another app presses nothing here.
  useEffect(() => { title.current?.focus(); }, []);

  const done = () => api.whatsNewSeen(WHATS_NEW_ID).catch(() => {});

  const later = () => {
    if (never) void done();
    onClose();
  };

  const cleanNow = async () => {
    if (!report) return;
    if (await run(report)) {
      await done();
      onClose();
    }
  };

  return (
    <div
      className="update-modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="wn-title"
      aria-describedby="wn-body"
      onKeyDown={(e) => { if (e.key === "Escape") later(); }}
    >
      <div className="update-card wn-card">
        <div className="wn-head">
          <h2 id="wn-title" ref={title} tabIndex={-1}>{t("wn2_title")}</h2>
        </div>
        <div className="wn-media">
          {still_only.current ? (
            <img src={still} alt="" />
          ) : (
            <video src={clip} poster={still} autoPlay loop muted playsInline aria-hidden="true" />
          )}
        </div>
        <p id="wn-body" className="wn-body">{t("wn2_body")}</p>
        {report && (
          <p className="wn-note">
            {worth ? <strong>{t("wn2_amount", { size: formatBytes(report.cleanable_bytes, lang) })}</strong> : t("wn2_none")}
          </p>
        )}
        {worth && (
          <label className="wn-never">
            <input type="checkbox" checked={never} onChange={(e) => setNever(e.target.checked)} />
            <span>{t("wn_never")}</span>
          </label>
        )}
        <div className="update-actions">
          {worth ? (
            <>
              <button className="primary" disabled={busy} onClick={() => { void cleanNow(); }}>{busy ? t("disk_cleaning") : t("wn2_clean")}</button>
              <button onClick={later}>{t("wn_later")}</button>
            </>
          ) : (
            <button className="primary" disabled={!report} onClick={() => { void done(); onClose(); }}>{t("wn_ok")}</button>
          )}
        </div>
      </div>
    </div>
  );
}
