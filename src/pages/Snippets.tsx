import { useEffect, useState } from "react";
import { api, emptySnippet, Snippet } from "../api";
import { useApp } from "../hooks";
import { Button, Card, Field } from "../ui";

export default function Snippets() {
  const { t, toast, lang } = useApp();
  const [items, setItems] = useState<Snippet[]>([]);
  const [form, setForm] = useState<Snippet>(emptySnippet());
  const [q, setQ] = useState("");

  const load = () => api.snippets().then(setItems).catch((e) => toast(String(e), "err"));
  useEffect(() => { load(); }, []);

  const save = async () => {
    try {
      await api.saveSnippet(form);
      setForm(emptySnippet());
      toast(t("saved"));
      load();
    } catch (e) { toast(String(e), "err"); }
  };

  const exportJson = () => {
    const blob = new Blob([JSON.stringify(items, null, 2)], { type: "application/json" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = "lalia-snippets.json";
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
        const data = JSON.parse(await f.text()) as Snippet[];
        for (const s of data) await api.saveSnippet({ ...emptySnippet(), trigger: s.trigger, expansion: s.expansion, enabled: s.enabled ?? true });
        toast(t("import"));
        load();
      } catch (e) { toast(String(e), "err"); }
    };
    input.click();
  };

  const filtered = items.filter((s) => !q || s.trigger.toLowerCase().includes(q.toLowerCase()) || s.expansion.toLowerCase().includes(q.toLowerCase()));

  return (
    <>
      <h1>{t("nav_snippets")}</h1>
      <Card title={form.id ? t("save") : t("add_snippet")}>
        <Field label={t("trigger")} hint={lang === "el" ? "π.χ. «η υπογραφή μου»" : "e.g. 'my email signature'"}>
          <input type="text" value={form.trigger} onChange={(e) => setForm({ ...form, trigger: e.target.value })} />
        </Field>
        <Field label={t("expansion")}>
          <textarea value={form.expansion} onChange={(e) => setForm({ ...form, expansion: e.target.value })} />
        </Field>
        <div className="row">
          <Button kind="primary" onClick={save} disabled={!form.trigger.trim() || !form.expansion}>{t("save")}</Button>
          {form.id && <Button onClick={() => setForm(emptySnippet())}>{t("cancel")}</Button>}
        </div>
      </Card>
      <Card title={`${t("nav_snippets")} (${items.length})`} actions={<>
        <input type="search" placeholder={t("search")} value={q} onChange={(e) => setQ(e.target.value)} style={{ width: 220 }} />
        <Button onClick={importJson}>{t("import")}</Button>
        <Button onClick={exportJson}>{t("export")}</Button>
      </>}>
        {filtered.length === 0 && <div className="empty">-</div>}
        {filtered.map((s) => (
          <div className="history-item" key={s.id} style={{ opacity: s.enabled ? 1 : 0.5 }}>
            <div><strong>{s.trigger}</strong> <span className="hint">{t("applied_times", { n: s.apply_count })}</span></div>
            <div className="history-raw">{s.expansion}</div>
            <div className="history-actions">
              <Button onClick={() => api.saveSnippet({ ...s, enabled: !s.enabled }).then(load)}>{s.enabled ? "⏸" : "▶"}</Button>
              <Button onClick={() => setForm(s)}>✎</Button>
              <Button kind="danger" onClick={() => api.deleteSnippet(s.id).then(load)}>{t("delete")}</Button>
            </div>
          </div>
        ))}
      </Card>
    </>
  );
}
