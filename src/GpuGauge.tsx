import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, GpuGaugeInfo } from "./api";
import { useApp } from "./hooks";

// The rail on the right: how full the graphics card is, and what that means for
// the model doing the listening.
//
// It exists because of 7 September 2026, when dictation went from one second to
// thirty over the course of an afternoon while nothing in the app changed. The
// card had filled up with other programs, Windows had pushed most of the speech
// model out to system RAM, and the app had no idea: it only ever asked the card
// how big it was, never how much of it was still free. This rail asks the
// second question, every few seconds, out loud.

/// The fastest median any measured model reached, so the speed bar has a fixed
/// meaning instead of moving with whatever is installed. From eval/bench-summary.md.
const FASTEST_MS = 304;

const POLL_MS = 3000;

export default function GpuGauge() {
  const { settings, setSettings, t, toast } = useApp();
  const [g, setG] = useState<GpuGaugeInfo | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    const read = () => api.gpuGauge().then(setG).catch(() => {});
    read();
    const timer = setInterval(read, POLL_MS);
    // A dictation both finishes and changes the card, so read again right after.
    const un = listen("lalia://history-changed", read);
    return () => { clearInterval(timer); un.then((f) => f()); };
  }, []);

  if (!g) return <aside className="gauge" aria-hidden="true" />;

  const tone = g.state === "ok" ? "ok" : g.state === "tight" ? "warn" : "err";
  const word = g.state === "ok" ? t("gauge_ok") : g.state === "tight" ? t("gauge_tight") : t("gauge_spilled");

  // Four bars, each one a measurement rather than a deduction. An earlier
  // version stacked "what others hold" and "what the model wants" in one bar
  // and let the arithmetic decide whether it fitted; on 7 September 2026 that
  // arithmetic said 6027 + 1960 fits inside 8017 while Windows had in fact
  // pushed 1825 MB of the model out to system RAM. The card keeps a reserve the
  // sum does not know about, so residency is read, never inferred.
  const total = g.card?.total_mb ?? 0;
  const onCard = g.on_card_mb ?? 0;
  const cardFull = total > 0 ? Math.round(((g.card?.used_mb ?? 0) / total) * 100) : 0;
  const resident = g.needs_mb > 0 && g.on_card_mb !== null ? Math.round((onCard / g.needs_mb) * 100) : null;

  const accuracy = g.greek_errors_pct === null ? null : 100 - g.greek_errors_pct;
  const speed = g.median_ms ? Math.round((FASTEST_MS / g.median_ms) * 100) : null;

  const switchTo = async (id: string, name: string) => {
    setBusy(true);
    try {
      const next = structuredClone(settings);
      next.asr.model_id = id;
      await setSettings(next);
      toast(t("gauge_switched", { name }));
    } catch (err) {
      toast(String(err), "err");
    } finally {
      setBusy(false);
    }
  };

  return (
    <aside className="gauge" aria-label={t("gauge_title")}>
      <h2>{t("gauge_title")}</h2>

      {g.card === null ? (
        <p className="hint">{t("gauge_no_card")}</p>
      ) : (
        <>
          <p className={`gauge-word ${tone}`}>{word}</p>

          <Bar label={t("gauge_full")} pct={cardFull} note={`${cardFull}%`} tone={tone} />
          <p className="gauge-note">{t("gauge_used", { used: g.card.used_mb, total })}</p>
          <p className="gauge-note">{t("gauge_free", { free: g.card.free_mb })}</p>

          {resident !== null && (
            <>
              <Bar label={t("gauge_resident")} pct={resident} note={`${resident}%`} tone={resident < 95 ? "err" : "ok"} />
              <p className="gauge-note">{t("gauge_on_card", { on: onCard, needs: g.needs_mb })}</p>
              {g.pushed_out_mb !== null && g.pushed_out_mb > 0 && (
                <p className="gauge-note err">{t("gauge_pushed_out", { out: g.pushed_out_mb })}</p>
              )}
            </>
          )}
          {resident === null && <p className="gauge-note">{t("gauge_needs", { needs: g.needs_mb })}</p>}

          {g.state !== "ok" && <p className="hint">{g.state === "spilled" ? t("gauge_spilled_why") : t("gauge_tight_why")}</p>}
        </>
      )}

      <h2>{t("gauge_model")}</h2>
      <p className="gauge-model-name">{g.model_name}</p>

      {accuracy === null || speed === null ? (
        <p className="hint">{t("gauge_unmeasured")}</p>
      ) : (
        <>
          <Bar label={t("gauge_accuracy")} pct={accuracy} note={`${accuracy}%`} />
          <Bar label={t("gauge_speed")} pct={speed} note={`${g.median_ms} ms`} />
        </>
      )}

      {g.fits_instead && g.fits_instead_name && (
        <button className="btn primary gauge-switch" disabled={busy} onClick={() => switchTo(g.fits_instead!, g.fits_instead_name!)}>
          {t("gauge_switch", { name: g.fits_instead_name })}
        </button>
      )}
    </aside>
  );
}

function Bar({ label, pct, note, tone }: { label: string; pct: number; note: string; tone?: string }) {
  return (
    <div className="gauge-bar">
      <span className="gauge-bar-label">
        {label}
        <span className="gauge-bar-note">{note}</span>
      </span>
      <div className="gauge-bar-track" role="img" aria-label={`${label}: ${note}`}>
        <div className={tone ?? ""} style={{ width: `${Math.max(0, Math.min(100, pct))}%` }} />
      </div>
    </div>
  );
}
