import { useEffect, useState } from "react";
import { api, DayPoint, MilestoneBadge, MilestonesView, Reflect } from "../api";
import { useApp } from "../hooks";
import type { Strings } from "../i18n";
import { Button, Card, Stat } from "../ui";

type Period = "7" | "30" | "all";
type T = ReturnType<typeof useApp>["t"];

// Milestones: a level that grows with every word, badges, and a Reflect
// look-back over 7 days, 30 days or the whole history. Every number comes
// from this PC; the sentences are templates in i18n.ts. User text (app names,
// dictionary words) is rendered as React text only.
export default function Milestones() {
  const { t, toast, settings } = useApp();
  const locale = settings.general.ui_language === "en" ? "en-GB" : "el-GR";
  const [view, setView] = useState<MilestonesView | null>(null);
  const [period, setPeriod] = useState<Period>("30");
  const [shown, setShown] = useState<Period | null>(null);
  const [busy, setBusy] = useState(false);

  const load = (p: Period, reveal: boolean) => {
    setBusy(true);
    api.milestones(p)
      .then((v) => { setView(v); if (reveal) setShown(p); })
      .catch((e) => toast(String(e), "err"))
      .finally(() => setBusy(false));
  };
  useEffect(() => { load("30", false); }, []);

  const pick = (p: Period) => {
    setPeriod(p);
    // After the first look-back, a new period is a new look-back at once.
    if (shown) load(p, true);
  };

  if (!view) return <h1>{t("nav_milestones").toLowerCase()}</h1>;
  const lv = view.level;
  const span = Math.max(1, lv.need_next - lv.need_this);
  const into = Math.min(span, Math.max(0, lv.xp - lv.need_this));
  const nf = (n: number) => Math.round(n).toLocaleString(locale);
  const nextTitleLevel = (lv.title + 1) * 5;
  const done = view.badges.filter((b) => b.done).length;

  return (
    <>
      <h1>{t("nav_milestones").toLowerCase()}</h1>

      <Card>
        <div className="ms-level">
          <div className="ms-level-num">{t("ms_level", { n: lv.level })}</div>
          <div className="ms-level-title">{t(`ms_title_${lv.title}` as keyof Strings)}</div>
        </div>
        <div className="ms-bar" role="progressbar" aria-valuemin={0} aria-valuemax={span} aria-valuenow={into} aria-label={t("ms_level", { n: lv.level + 1 })}>
          <div style={{ width: `${(into / span) * 100}%` }} />
        </div>
        <p className="ms-line">{t("ms_xp_line", { xp: nf(lv.xp), left: nf(lv.need_next - lv.xp), next: lv.level + 1 })}</p>
        <p className="muted ms-small">{t("ms_xp_parts", { w: nf(lv.xp_words), d: nf(lv.xp_days), s: nf(lv.xp_streak) })}</p>
        <p className="ms-line">{t("ms_streak_line", { n: lv.current_streak, m: lv.longest_streak })}</p>
        {lv.title < 10 && (
          <p className="muted ms-small">{t("ms_next_title", { n: nextTitleLevel, title: t(`ms_title_${lv.title + 1}` as keyof Strings) })}</p>
        )}
        <p className="muted ms-small">{t("ms_xp_how")}</p>
      </Card>

      <Card title={`${t("ms_badges")} · ${t("ms_badges_done", { n: done, m: view.badges.length })}`}>
        <div className="ms-badges">
          {view.badges.map((b) => <BadgeTile key={b.id} b={b} t={t} nf={nf} />)}
        </div>
      </Card>

      <Card title={t("ms_reflect")}>
        <p className="ms-line">{t("ms_reflect_lead")}</p>
        <div className="row ms-periods" role="group" aria-label={t("ms_reflect")}>
          {(["7", "30", "all"] as Period[]).map((p) => (
            <button key={p} type="button" className={`ms-chip ${period === p ? "on" : ""}`} aria-pressed={period === p} onClick={() => pick(p)} disabled={busy}>
              {t(`ms_period_${p}` as keyof Strings)}
            </button>
          ))}
          <Button kind="primary" disabled={busy} onClick={() => load(period, true)}>{t("ms_reflect_run")}</Button>
        </div>
      </Card>

      {shown && view.reflect && <ReflectView r={view.reflect} lifetimeStreak={[lv.current_streak, lv.longest_streak]} t={t} nf={nf} locale={locale} />}
    </>
  );
}

function BadgeTile({ b, t, nf }: { b: MilestoneBadge; t: T; nf: (n: number) => string }) {
  return (
    <div className={`ms-badge ${b.done ? "done" : ""}`}>
      <div className="ms-badge-name">{t(`b_${b.id}` as keyof Strings)}</div>
      <div className="ms-badge-rule">{t(`b_${b.id}_d` as keyof Strings)}</div>
      <div className="ms-bar thin" role="progressbar" aria-valuemin={0} aria-valuemax={b.target} aria-valuenow={b.value}>
        <div style={{ width: `${(b.value / Math.max(1, b.target)) * 100}%` }} />
      </div>
      <div className="ms-badge-count">{nf(b.value)} / {nf(b.target)}</div>
    </div>
  );
}

/// One finding: the sentence, and when it rests on an assumed constant, the
/// assumption under it.
function Fact({ text, note, t }: { text: string; note?: string; t: T }) {
  return (
    <div className="ms-fact">
      <p>{text}</p>
      {note && <p className="ms-note">{t("ms_assumption")}: {note}</p>}
    </div>
  );
}

function ReflectView({ r, lifetimeStreak, t, nf, locale }: { r: Reflect; lifetimeStreak: [number, number]; t: T; nf: (n: number) => string; locale: string }) {
  const pct = (x: number) => Math.round(x * 100);
  const dec = (x: number) => x.toLocaleString(locale, { maximumFractionDigits: 1 });
  const dayName = (iso: string, weekday = false) => {
    const [y, m, d] = iso.split("-").map(Number);
    return new Date(y, m - 1, d).toLocaleDateString(locale, weekday ? { weekday: "long", day: "numeric", month: "long" } : { day: "numeric", month: "long" });
  };
  const dur = (min: number) => {
    const total = Math.round(min);
    const h = Math.floor(total / 60);
    return h > 0 ? t("dur_h_m", { h, m: total % 60 }) : t("dur_m", { m: total });
  };
  const langName = (code: string) => {
    let name = code;
    try { name = new Intl.DisplayNames([locale], { type: "language" }).of(code) ?? code; } catch { /* old engine */ }
    return locale.startsWith("el") ? name.toLowerCase() : name;
  };

  const head = r.since_first_use
    ? t("ms_since_first", { n: r.days_span - 1 })
    : t("ms_window", { from: dayName(r.start_day), to: dayName(r.end_day), n: r.days_span });

  if (r.dictations === 0) {
    return <Card title={t("ms_reflect")}><p className="ms-line">{head}</p><p className="ms-line">{t("ms_empty")}</p></Card>;
  }

  // Volume
  const volume: React.ReactNode[] = [];
  let line = t("f_words", { words: nf(r.words), n: nf(r.dictations), avg: nf(r.avg_words) });
  if (r.prev_words != null && r.prev_words > 0) {
    const change = Math.round(((r.window_words - r.prev_words) / r.prev_words) * 100);
    line += " " + (change > 0 ? t("f_vs_prev_up", { pct: change, n: r.days_span }) : change < 0 ? t("f_vs_prev_down", { pct: -change, n: r.days_span }) : t("f_vs_prev_same", { n: r.days_span }));
  }
  volume.push(<Fact key="w" text={line} t={t} />);
  if (r.words >= 250) volume.push(<Fact key="p" text={t("f_pages", { pages: nf(r.words / 250) })} note={t("f_pages_note")} t={t} />);
  const perDay = r.words / Math.max(1, r.days_span);
  if (perDay > 0) volume.push(<Fact key="g" text={t("f_gatsby", { days: nf(47000 / perDay) })} note={t("f_gatsby_note")} t={t} />);
  volume.push(<Fact key="k" text={t("f_keys", { n: nf(r.keystrokes) })} t={t} />);
  if (r.accents > 0) volume.push(<Fact key="a" text={t("f_accents", { n: nf(r.accents) })} t={t} />);
  volume.push(<Fact key="d" text={t("f_active", { n: r.active_days, span: r.days_span })} t={t} />);
  if (r.speaking_min > 0) volume.push(<Fact key="tt" text={t("f_talk", { time: dur(r.speaking_min) })} t={t} />);

  // Speed and time saved
  const speed: React.ReactNode[] = [];
  if (r.wpm > 0) speed.push(<Fact key="s" text={t("f_speed", { wpm: nf(r.wpm), x: dec(r.wpm / 40) })} note={t("f_speed_note")} t={t} />);
  if (r.fastest) {
    let f = t("f_fastest", { wpm: r.fastest.wpm, words: r.fastest.words, secs: r.fastest.seconds, day: dayName(r.fastest.day) });
    if (r.fastest.app) f += " " + t("f_fastest_app", { app: r.fastest.app });
    speed.push(<Fact key="f" text={f} t={t} />);
  }
  if (r.p50_ms > 0) speed.push(<Fact key="l" text={t("f_latency", { p50: dec(r.p50_ms / 1000), p95: dec(r.p95_ms / 1000) })} t={t} />);
  if (r.time_saved_min >= 1) {
    const units: [keyof Strings, number][] = [["f_saved_movies", 120], ["f_saved_episodes", 22], ["f_saved_songs", 3.5]];
    const unit = units.find(([, m]) => r.time_saved_min / m >= 2 && r.time_saved_min / m < 100) ?? units[2];
    speed.push(<Fact key="ts" text={`${t("f_saved", { time: dur(r.time_saved_min) })} ${t(unit[0], { n: nf(r.time_saved_min / unit[1]) })}`} note={t("f_saved_note")} t={t} />);
  }
  if (r.inference_min > 0) speed.push(<Fact key="pc" text={`${t("f_pc", { min: dec(r.inference_min) })}${r.all_local ? " " + t("f_pc_local") : ""}`} t={t} />);

  // Habits, personas: only once there is enough to say something true.
  const habits: React.ReactNode[] = [];
  habits.push(<Fact key="st" text={t("f_streak", { n: lifetimeStreak[0], m: lifetimeStreak[1] })} t={t} />);
  if (r.best_day) habits.push(<Fact key="bd" text={t("f_best_day", { day: dayName(r.best_day.day, true), words: nf(r.best_day.words) })} t={t} />);
  if (r.enough_data) {
    if (r.power_hour != null) habits.push(<Fact key="ph" text={t("f_power_hour", { from: String(r.power_hour).padStart(2, "0"), to: String((r.power_hour + 1) % 24).padStart(2, "0"), pct: pct(r.power_hour_share) })} t={t} />);
    const persona = r.night_share >= 0.25 ? t("p_night", { pct: pct(r.night_share) }) : r.morning_share >= 0.25 ? t("p_morning", { pct: pct(r.morning_share) }) : t("p_day");
    habits.push(<Fact key="pn" text={persona} t={t} />);
    if (r.top_weekday != null) habits.push(<Fact key="wd" text={r.weekend_share >= 0.4 ? t("f_weekend", { pct: pct(r.weekend_share) }) : t("f_weekday", { day: t(`wd_${r.top_weekday}` as keyof Strings) })} t={t} />);
  }
  if (r.longest_session) habits.push(<Fact key="ls" text={t("f_session", { min: r.longest_session.minutes, n: r.longest_session.dictations, day: dayName(r.longest_session.day) })} note={t("f_session_note")} t={t} />);

  // Languages and apps
  const lang: React.ReactNode[] = [];
  const total = r.languages.reduce((a, [, w]) => a + w, 0);
  const shown = r.languages.filter(([, w]) => total > 0 && w / total >= 0.01).slice(0, 3);
  if (shown.length > 0) {
    const list = shown.map(([c, w]) => `${pct(w / total)}% ${langName(c)}`).join(", ");
    lang.push(<Fact key="lg" text={`${t("f_langs", { list })}${shown.length >= 2 ? " " + t("f_langs_two") : ""}`} t={t} />);
  }
  if (r.switches >= 3) lang.push(<Fact key="sw" text={t("f_switch", { n: nf(r.switches) })} t={t} />);
  if (r.enough_data) lang.push(<Fact key="pe" text={t("f_persona", { p: t(`p_${r.persona}` as keyof Strings) })} t={t} />);
  if (r.top_app) {
    const a = r.top_app.name ? t("f_top_app", { app: r.top_app.name, pct: pct(r.top_app.share) }) : t("f_top_app_private", { pct: pct(r.top_app.share) });
    lang.push(<Fact key="ta" text={`${a}${r.ai_over_people ? " " + t("f_ai_people") : ""}`} t={t} />);
  }
  if (r.apps > 1) lang.push(<Fact key="ap" text={t("f_apps", { n: r.apps })} t={t} />);

  // What the cleanup did
  const clean: React.ReactNode[] = [];
  if (r.fillers > 0) clean.push(<Fact key="fi" text={t("f_fillers", { n: nf(r.fillers) })} t={t} />);
  if (r.stutters + r.repeats > 0) clean.push(<Fact key="su" text={t("f_stutters", { s: nf(r.stutters), r: nf(r.repeats) })} t={t} />);
  if (r.questions > 0) clean.push(<Fact key="q" text={t("f_questions", { n: nf(r.questions), g: nf(r.greek_questions) })} t={t} />);
  if (r.self_corrections > 0) clean.push(<Fact key="sc" text={t("f_selfcorr", { n: nf(r.self_corrections) })} t={t} />);
  if (r.dictionary_top) clean.push(<Fact key="dt" text={t("f_dict_top", { wrong: r.dictionary_top[0], right: r.dictionary_top[1], n: nf(r.dictionary_top[2]) })} t={t} />);
  if (r.dictionary_fixes > 0) clean.push(<Fact key="df" text={t("f_dict_total", { n: nf(r.dictionary_fixes) })} t={t} />);
  clean.push(<Fact key="un" text={t("f_untouched", { pct: pct(r.untouched_share) })} t={t} />);

  const days = r.daily.slice(-90);
  return (
    <>
      <Card title={t("ms_reflect")}>
        <p className="ms-line">{head}</p>
        {!r.enough_data && <p className="muted ms-small">{t("ms_not_enough")}</p>}
      </Card>
      <Group title={t("ms_group_volume")} items={volume} />
      <Group title={t("ms_group_speed")} items={speed} />
      <Group title={t("ms_group_habits")} items={habits} />
      <Group title={t("ms_group_lang")} items={lang} />
      <Group title={t("ms_group_clean")} items={clean} />
      <Card title={t("ms_usage")}>
        <div className="grid4">
          <Stat label={t("ms_usage_words")} value={nf(r.words)} />
          <Stat label={t("ms_usage_dictations")} value={nf(r.dictations)} />
          <Stat label={t("ms_usage_talk")} value={nf(r.speaking_min)} />
          <Stat label={t("ms_usage_wpm")} value={nf(r.wpm)} />
          <Stat label={t("ms_usage_p50")} value={`${dec(r.p50_ms / 1000)} s`} />
          <Stat label={t("ms_usage_p95")} value={`${dec(r.p95_ms / 1000)} s`} />
          <Stat label={t("ms_usage_pc")} value={dec(r.inference_min)} />
          <Stat label={t("ms_usage_active")} value={`${r.active_days} / ${r.days_span}`} />
        </div>
        <h2 className="ms-chart-title">{t("ms_by_day")}</h2>
        <Bars points={days.map((d: DayPoint) => ({ key: d.day, value: d.words, label: d.day.slice(8), title: `${dayName(d.day)}: ${nf(d.words)}` }))} />
        <h2 className="ms-chart-title">{t("ms_by_hour")}</h2>
        <Bars points={r.by_hour.map((w, h) => ({ key: String(h), value: w, label: h % 3 === 0 ? String(h).padStart(2, "0") : "", title: `${String(h).padStart(2, "0")}:00: ${nf(w)}` }))} />
      </Card>
    </>
  );
}

function Group({ title, items }: { title: string; items: React.ReactNode[] }) {
  if (items.length === 0) return null;
  return (
    <Card title={title}>
      <div className="ms-facts">{items}</div>
    </Card>
  );
}

function Bars({ points }: { points: { key: string; value: number; label: string; title: string }[] }) {
  const max = Math.max(1, ...points.map((p) => p.value));
  return (
    <div className="bars-chart ms-chart">
      {points.map((p) => (
        <div key={p.key} style={{ height: `${Math.max(2, (p.value / max) * 100)}%` }} title={p.title}>
          <span>{p.label}</span>
        </div>
      ))}
    </div>
  );
}
