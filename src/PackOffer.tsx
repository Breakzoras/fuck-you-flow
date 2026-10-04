import { useEffect, useState } from "react";
import { api, languageName, PackStatus } from "./api";
import { useApp } from "./hooks";
import { Button, Card } from "./ui";

// The ready-made dictionary for the user's language, downloaded from the
// maker's public repository (packs.rs). "offer" is the one-time question on
// the first screen; "manage" is the line on the Dictionary page, where it can
// be fetched, checked or taken out at any time.
//
// The list on GitHub is read only when it is needed (4 October 2026; it used
// to be read on every visit to Home and Dictionary, also after "Not now"):
// the question looks once per run while it is unanswered, and the Dictionary
// page looks when the user presses its button. After a yes the daily check in
// packs.rs takes over.
let firstLook: { key: string; status: Promise<PackStatus> } | null = null;

export default function PackOffer({ mode, onChanged }: { mode: "offer" | "manage"; onChanged?: () => void }) {
  const { settings, t, toast } = useApp();
  const packs = settings.packs ?? { enabled: false, asked: false, language: "", version: 0 };
  const [status, setStatus] = useState<PackStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const asking = mode === "offer" && !packs.asked && !packs.enabled;

  useEffect(() => {
    if (!asking) return;
    let live = true;
    const key = `${settings.language.mode}|${settings.language.primary}`;
    if (!firstLook || firstLook.key !== key) firstLook = { key, status: api.packStatus() };
    firstLook.status.then((s) => live && setStatus(s)).catch(() => live && setStatus(null));
    return () => { live = false; };
  }, [asking, settings.language.mode, settings.language.primary, settings.general.language_confirmed]);

  const look = async () => {
    setBusy(true);
    try {
      setStatus(await api.packStatus());
    } catch (e) {
      toast(String(e), "err");
    } finally {
      setBusy(false);
    }
  };

  const langLabel = (code: string) => (code === "all" ? t("pack_lang_all") : languageName(code));

  const install = async () => {
    setBusy(true);
    try {
      const done = await api.packInstall();
      toast(t("pack_added", { n: done.added }));
      onChanged?.();
    } catch (e) {
      toast(String(e) === "offline" ? t("pack_offline") : t("pack_failed", { e: String(e) }), "err");
    } finally {
      setBusy(false);
    }
  };

  const decline = async () => {
    try {
      await api.packDecline();
    } catch (e) {
      toast(String(e), "err");
    }
  };

  const remove = async () => {
    setBusy(true);
    try {
      await api.packRemove();
      toast(t("pack_removed"));
      onChanged?.();
    } catch (e) {
      toast(String(e), "err");
    } finally {
      setBusy(false);
    }
  };

  if (mode === "offer") {
    // Asked once, and only when there is something to give.
    if (packs.asked || packs.enabled || !status?.language) return null;
    return (
      <Card title={t("pack_title")}>
        <p className="hint">{t("pack_offer", { lang: langLabel(status.language), n: status.rules })}</p>
        <div className="row" style={{ marginTop: 10 }}>
          <Button kind="primary" disabled={busy} onClick={install}>{t("pack_yes")}</Button>
          <Button disabled={busy} onClick={decline}>{t("pack_not_now")}</Button>
        </div>
      </Card>
    );
  }

  if (packs.enabled) {
    return (
      <Card title={t("pack_title")}>
        <p className="hint">{t("pack_on", { lang: langLabel(packs.language || status?.language || ""), v: packs.version })}</p>
        <div className="row" style={{ marginTop: 10 }}>
          <Button disabled={busy} onClick={install}>{t("pack_check")}</Button>
          <Button kind="danger" disabled={busy} onClick={remove}>{t("pack_remove")}</Button>
        </div>
      </Card>
    );
  }
  if (!status || !status.online || !status.language) {
    // Nothing is read before the button: it says what the button does, then
    // what it found.
    const said = !status ? t("pack_intro") : !status.online ? t("pack_offline") : t("pack_none");
    return (
      <Card title={t("pack_title")}>
        <p className="hint">{said}</p>
        <div className="row" style={{ marginTop: 10 }}>
          <Button disabled={busy} onClick={look}>{t("pack_look")}</Button>
        </div>
      </Card>
    );
  }
  return (
    <Card title={t("pack_title")}>
      <p className="hint">{t("pack_available", { lang: langLabel(status.language), n: status.rules })}</p>
      <div className="row" style={{ marginTop: 10 }}>
        <Button kind="primary" disabled={busy} onClick={install}>{t("pack_download")}</Button>
      </div>
    </Card>
  );
}
