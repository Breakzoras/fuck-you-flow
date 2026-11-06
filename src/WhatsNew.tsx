import { useEffect, useRef, useState } from "react";
import { useApp } from "./hooks";
import { api } from "./api";
import BrandMarks from "./BrandMarks";
import clipEn from "./assets/whats-new/appmic.webm";
import clipEl from "./assets/whats-new/appmic-el.webm";
import still from "./assets/whats-new/appmic-still.jpg";
import { WHATS_NEW_ID } from "./whatsNewRule";

/** The "What's new" window of 0.9.19: the dictation key presses the Claude
 *  or Codex microphone. Shown once to people who updated (a new install
 *  starts with it seen). The clip is a 10 second mockup made with HyperFrames
 *  (videos/fyf-appmic-whatsnew, Greek copy in fyf-appmic-whatsnew-el), muted and looping; with reduced motion asked
 *  for, a still of its last card shows instead. */
export default function WhatsNew({ onClose }: { onClose: () => void }) {
  const { settings, setSettings, lang, t, toast } = useApp();
  // The words typed in the clip follow the interface language.
  const clip = lang === "el" ? clipEl : clipEn;
  const [never, setNever] = useState(false);
  const [busy, setBusy] = useState(false);
  const title = useRef<HTMLHeadingElement>(null);
  const still_only = useRef(window.matchMedia?.("(prefers-reduced-motion: reduce)").matches ?? false);
  const alreadyOn = !!settings.asr.app_mic;

  // The title holds the focus, as on the update card: a stray Enter or Space
  // typed for another app presses nothing here.
  useEffect(() => { title.current?.focus(); }, []);

  const done = () => api.whatsNewSeen(WHATS_NEW_ID).catch(() => {});

  const later = () => {
    if (never) void done();
    onClose();
  };

  const turnOn = async () => {
    setBusy(true);
    try {
      await setSettings({ ...settings, asr: { ...settings.asr, app_mic: true } });
      await done();
      toast(t("wn_on_done"));
      onClose();
    } catch {
      setBusy(false);
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
          <BrandMarks size={28} />
          <h2 id="wn-title" ref={title} tabIndex={-1}>{t("wn_title")}</h2>
        </div>
        <div className="wn-media">
          {still_only.current ? (
            <img src={still} alt="" />
          ) : (
            <video src={clip} poster={still} autoPlay loop muted playsInline aria-hidden="true" />
          )}
        </div>
        <p id="wn-body" className="wn-body">{t("wn_body", { key: settings.hotkeys.hands_free })}</p>
        <p className="wn-note">{t(alreadyOn ? "wn_note_on" : "wn_note")}</p>
        <label className="wn-never">
          <input type="checkbox" checked={never} onChange={(e) => setNever(e.target.checked)} />
          <span>{t("wn_never")}</span>
        </label>
        <div className="update-actions">
          {alreadyOn ? (
            <button className="primary" onClick={() => { void done(); onClose(); }}>{t("wn_ok")}</button>
          ) : (
            <>
              <button className="primary" disabled={busy} onClick={() => { void turnOn(); }}>{t("wn_turn_on")}</button>
              <button onClick={later}>{t("wn_later")}</button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
