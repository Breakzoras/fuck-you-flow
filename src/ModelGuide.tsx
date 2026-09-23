import { useEffect, useMemo, useState } from "react";
import { api, fmtBytes, ModelStatus } from "./api";
import { useApp } from "./hooks";
import { Badge, Button } from "./ui";
import { machineFrom, Pick, Priority, recommend, SpeakLang, speakLangFrom } from "./modelChoice";

export interface MachineProfile {
  gpus?: { name: string; vram_mb: number }[];
  vulkan_runtime?: boolean;
  cuda_driver?: boolean;
}

/// "Help me choose": two plain questions, the machine read on its own, and one
/// model with the reason it suits this person. People need different things,
/// and a table of model names told nobody which one was theirs.
export default function ModelGuide({ models, profile, currentModelId, currentBackend, onApply, onPick }: {
  models: ModelStatus[];
  profile: MachineProfile | null | undefined;
  currentModelId: string;
  currentBackend: string | undefined;
  onApply: (modelId: string, backend: string) => Promise<void>;
  onPick?: (pick: Pick) => void;
}) {
  const { settings, t, toast } = useApp();
  const [lang, setLang] = useState<SpeakLang>(() => speakLangFrom(settings.language));
  const [priority, setPriority] = useState<Priority>("accuracy");
  const [busy, setBusy] = useState(false);

  const machine = useMemo(() => machineFrom(profile), [profile]);
  const pick = useMemo(() => recommend(machine, lang, priority), [machine, lang, priority]);
  useEffect(() => { onPick?.(pick); }, [pick.modelId, pick.backend]);

  const target = models.find((m) => m.id === pick.modelId);
  const inUse = currentModelId === pick.modelId && (currentBackend ?? "auto") === pick.backend;
  const best = [...(profile?.gpus ?? [])].sort((a, b) => b.vram_mb - a.vram_mb)[0];
  const machineLine = machine.usableGpu && best
    ? `${best.name}, ${t("guide_machine_gpu", { gb: Math.round(best.vram_mb / 1024) })}`
    : t("guide_machine_nogpu");

  const apply = async () => {
    setBusy(true);
    try {
      if (target && !target.installed) await api.downloadModel(target.id);
      await onApply(pick.modelId, pick.backend);
      toast(t("guide_applied", { model: target?.display_name ?? pick.modelId }));
    } catch (e) {
      toast(String(e), "err");
    } finally {
      setBusy(false);
    }
  };

  const choice = <T extends string>(value: T, current: T, set: (v: T) => void, label: string) => (
    <Button key={value} kind={value === current ? "primary" : "default"} onClick={() => set(value)}>{label}</Button>
  );

  return (
    <div className="guide">
      <p className="hint" style={{ marginTop: 0 }}>{t("guide_intro")}</p>
      <p><strong>{t("guide_machine")}:</strong> {machineLine}</p>

      <p style={{ margin: "14px 0 6px" }}><strong>{t("guide_q_lang")}</strong></p>
      <div className="row" style={{ gap: 6, flexWrap: "wrap" }}>
        {choice<SpeakLang>("el_mixed", lang, setLang, t("guide_lang_el"))}
        {choice<SpeakLang>("english", lang, setLang, t("guide_lang_en"))}
        {choice<SpeakLang>("other", lang, setLang, t("guide_lang_other"))}
      </div>

      <p style={{ margin: "14px 0 6px" }}><strong>{t("guide_q_priority")}</strong></p>
      <div className="row" style={{ gap: 6, flexWrap: "wrap" }}>
        {choice<Priority>("accuracy", priority, setPriority, t("guide_pr_accuracy"))}
        {choice<Priority>("speed", priority, setPriority, t("guide_pr_speed"))}
        {choice<Priority>("light", priority, setPriority, t("guide_pr_light"))}
      </div>

      <div className="guide-result" style={{ marginTop: 16, padding: "12px 14px", border: "1px solid var(--line)", borderRadius: "var(--radius)", background: "var(--bg-2)" }}>
        <Badge tone="ok">★ {t("guide_result")}</Badge>
        <p style={{ margin: "8px 0 4px", fontSize: "1.15em" }}><strong>{target?.display_name ?? pick.modelId}</strong></p>
        <p style={{ margin: "0 0 10px" }}>{t(pick.whyKey as never)}</p>
        <p className="hint" style={{ margin: "0 0 10px" }}>{t("guide_runs_on")}: {t(pick.backend === "cpu" ? "backend_cpu" : "backend_auto")}</p>
        {inUse ? <Badge tone="ok">{t("guide_in_use")}</Badge> : (
          <Button kind="primary" disabled={busy} onClick={apply}>
            {busy ? t("downloading") : target && !target.installed ? t("guide_apply_download", { size: fmtBytes(target.size_bytes) }) : t("guide_apply")}
          </Button>
        )}
      </div>
    </div>
  );
}
