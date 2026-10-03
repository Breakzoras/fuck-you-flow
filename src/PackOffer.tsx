import { useEffect, useState } from "react";
import { api, languageName, PackStatus } from "./api";
import { useApp } from "./hooks";
import { Button, Card } from "./ui";

// The ready-made dictionary for the user's language, downloaded from the
// maker's public repository (packs.rs). "offer" is the one-time question on
// the first screen; "manage" is the line on the Dictionary page, where it can
// be fetched, checked or taken out at any time.
export default function PackOffer({ mode, onChanged }: { mode: "offer" | "manage"; onChanged?: () => void }) {
  const { settings, t, toast } = useApp();
  const packs = settings.packs ?? { enabled: false, asked: false, language: "", version: 0 };
  const [status, setStatus] = useState<PackStatus | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let live = true;
    api.packStatus().then((s) => live && setStatus(s)).catch(() => live && setStatus(null));
    return () => { live = false; };
  }, [settings.language.mode, settings.language.primary, settings.general.language_confirmed, packs.enabled]);

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
  if (status && !status.online) {
    return (
      <Card title={t("pack_title")}>
        <p className="hint">{t("pack_offline")}</p>
      </Card>
    );
  }
  if (!status?.language) return null;
  return (
    <Card title={t("pack_title")}>
      <p className="hint">{t("pack_available", { lang: langLabel(status.language), n: status.rules })}</p>
      <div className="row" style={{ marginTop: 10 }}>
        <Button kind="primary" disabled={busy} onClick={install}>{t("pack_download")}</Button>
      </div>
    </Card>
  );
}
