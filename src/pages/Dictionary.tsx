import { useEffect, useState } from "react";
import { api, DictionaryRule, emptyRule } from "../api";
import { useApp } from "../hooks";
import { Badge, Button, Card, Field, Select, Toggle } from "../ui";

export default function Dictionary() {
  const { t, toast, lang } = useApp();
  const [rules, setRules] = useState<DictionaryRule[]>([]);
  const [q, setQ] = useState("");
  const [sort, setSort] = useState<"recent" | "usage" | "alpha">("recent");
  const [form, setForm] = useState<DictionaryRule>(emptyRule());
  const [testIn, setTestIn] = useState("");
  const [testOut, setTestOut] = useState<{ text: string; applied: string[] } | null>(null);

  const load = () => api.rules().then(setRules).catch((e) => toast(String(e), "err"));
  useEffect(() => { load(); }, []);

  const save = async () => {
    try {
      await api.saveRule(form);
      setForm(emptyRule());
      toast(t("saved"));
      load();
    } catch (e) { toast(String(e), "err"); }
  };

  const filtered = rules
    .filter((r) => !q || r.wrong.toLowerCase().includes(q.toLowerCase()) || r.correct.toLowerCase().includes(q.toLowerCase()))
    .sort((a, b) => sort === "usage" ? b.apply_count - a.apply_count : sort === "alpha" ? a.wrong.localeCompare(b.wrong) : b.updated_at.localeCompare(a.updated_at));

  const exportJson = () => {
    const blob = new Blob([JSON.stringify(rules, null, 2)], { type: "application/json" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = "lalia-dictionary.json";
    a.click();
  };
  const importJson = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.onchange = async () => {
      const f = input.files?.[0];
      if (!f) return;
      try {
        const data = JSON.parse(await f.text());
        const n = await api.importRules(Array.isArray(data) ? data : data.dictionary ?? []);
        toast(`${t("import")}: ${n}`);
        load();
      } catch (e) { toast(String(e), "err"); }
    };
    input.click();
  };

  return (
    <>
      <h1>{t("nav_dictionary")}</h1>
      <Card title={form.id ? t("save") : t("add_rule")}>
        <div className="grid2">
          <Field label={t("wrong")} hint={lang === "el" ? "π.χ. Λούραμ" : "e.g. Loo-ram"}>
            <input type="text" value={form.wrong} onChange={(e) => setForm({ ...form, wrong: e.target.value })} />
          </Field>
          <Field label={t("correct")} hint={lang === "el" ? "π.χ. Luram" : "e.g. Luram"}>
            <input type="text" value={form.correct} onChange={(e) => setForm({ ...form, correct: e.target.value })} />
          </Field>
          <Field label={t("match_mode")}>
            <Select value={form.match_mode} onChange={(v) => setForm({ ...form, match_mode: v })} options={[
              { value: "whole_word", label: t("whole_word") }, { value: "phrase", label: t("phrase") }, { value: "exact", label: t("exact") },
            ]} />
          </Field>
          <Field label={t("language_scope")}>
            <Select value={form.language ?? "global"} onChange={(v) => setForm({ ...form, language: v === "global" ? null : v })} options={[
              { value: "global", label: t("global") }, { value: "el", label: t("greek") }, { value: "en", label: t("english") },
            ]} />
          </Field>
        </div>
        <Toggle label={t("case_sensitive")} checked={form.case_sensitive} onChange={(v) => setForm({ ...form, case_sensitive: v })} />
        <Toggle label={t("use_as_hint")} checked={form.use_as_hint} onChange={(v) => setForm({ ...form, use_as_hint: v })} />
        <div className="row">
          <Button kind="primary" onClick={save} disabled={!form.wrong.trim() || !form.correct.trim()}>{t("save")}</Button>
          {form.id && <Button onClick={() => setForm(emptyRule())}>{t("cancel")}</Button>}
        </div>
      </Card>

      <Card title={t("test_text")}>
        <div className="row">
          <input type="text" value={testIn} onChange={(e) => setTestIn(e.target.value)} style={{ flex: 1 }} />
          <Button onClick={() => api.testRules(testIn).then(setTestOut)}>{t("verify")}</Button>
        </div>
        {testOut && <p><strong>{testOut.text}</strong> <span className="hint">{testOut.applied.join(" · ") || "-"}</span></p>}
      </Card>

      <Card
        title={`${t("nav_dictionary")} (${rules.length})`}
        actions={<>
          <input type="search" placeholder={t("search")} value={q} onChange={(e) => setQ(e.target.value)} style={{ width: 220 }} />
          <Select value={sort} onChange={setSort} options={[{ value: "recent", label: "recent" }, { value: "usage", label: "usage" }, { value: "alpha", label: "A-Ω" }]} />
          <Button onClick={importJson}>{t("import")}</Button>
          <Button onClick={exportJson}>{t("export")}</Button>
        </>}
      >
        {filtered.length === 0 && <div className="empty">-</div>}
        {filtered.length > 0 && (
          <table>
            <thead><tr><th>{t("wrong")}</th><th>{t("correct")}</th><th>{t("match_mode")}</th><th>{t("language_scope")}</th><th></th><th></th></tr></thead>
            <tbody>
              {filtered.map((r) => (
                <tr key={r.id} style={{ opacity: r.enabled ? 1 : 0.5 }}>
                  <td>{r.wrong}</td>
                  <td><strong>{r.correct}</strong></td>
                  <td>{r.match_mode === "whole_word" ? t("whole_word") : r.match_mode === "phrase" ? t("phrase") : t("exact")}{r.case_sensitive ? " · Aa" : ""}</td>
                  <td>{r.language ?? t("global")}{r.source === "suggested" ? " · " : ""}{r.source === "suggested" && <Badge>auto</Badge>}</td>
                  <td className="hint">{t("applied_times", { n: r.apply_count })}<br />{r.last_applied_at ? r.last_applied_at.slice(0, 10) : t("never")}</td>
                  <td>
                    <div className="row" style={{ gap: 4 }}>
                      <Button onClick={() => api.saveRule({ ...r, enabled: !r.enabled }).then(load)}>{r.enabled ? "⏸" : "▶"}</Button>
                      <Button onClick={() => setForm(r)}>✎</Button>
                      <Button kind="danger" onClick={() => api.deleteRule(r.id).then(load)}>✕</Button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Card>
    </>
  );
}
