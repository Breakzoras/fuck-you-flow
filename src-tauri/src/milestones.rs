//! Milestones: levels, badges and the Reflect look-back.
//!
//! Everything here is computed on the user's own PC from the tables the app
//! already keeps. Nothing is sent anywhere and no model is asked anything: the
//! sentences the page shows are templates in `src/i18n.ts`, filled with the
//! numbers below.
//!
//! Two sources, on purpose:
//! - `stats_daily` (one row per local day) never loses a day when old history is
//!   pruned, so levels and XP come from it and can only go up.
//! - `history` rows give the detail (hours, apps, languages, cleanup) for the
//!   Reflect window and the badges.
//!
//! The functions take rows that are already in local time, so the tests can
//! build them by hand and the clock change of 25 October cannot move a dictation
//! into the wrong hour.

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};
use serde::Serialize;

/// XP for each day with at least one dictation.
pub const DAY_BONUS: u64 = 250;
/// XP for each day that continues a streak: 50 on the second day in a row,
/// 100 on the third, up to `STREAK_STEP * STREAK_CAP` from the eleventh on.
pub const STREAK_STEP: u64 = 50;
pub const STREAK_CAP: u64 = 10;
/// XP needed to reach levels 1 to 10, picked by hand so a new user climbs a few
/// levels on the first day. Index = level.
const EARLY: [u64; 11] = [0, 0, 50, 150, 300, 600, 1_100, 2_000, 3_500, 6_500, 10_000];
/// After level 10 each step is 15% longer than the one before (about 4,000 to reach 11).
const BASE_STEP: f64 = 3_500.0;
const GROWTH: f64 = 1.15;
/// Typing speed the time-saved figures compare against, the same 40 words a
/// minute the Statistics page uses.
pub const TYPING_WPM: f64 = 40.0;

/// One `stats_daily` row.
#[derive(Debug, Clone)]
pub struct DayRow {
    pub day: NaiveDate,
    pub words: u64,
    pub dictations: u64,
    pub audio_ms: u64,
}

/// One counted dictation (status success or copied), time already local.
#[derive(Debug, Clone, Default)]
pub struct HistRow {
    pub at: NaiveDateTime,
    pub final_text: String,
    pub word_count: u64,
    pub audio_ms: u64,
    pub latency_ms: u64,
    pub inference_ms: u64,
    pub app_name: Option<String>,
    pub app_process: Option<String>,
    pub app_category: String,
    /// The language the engine heard, or the one the user locked. None when
    /// neither is known.
    pub language: Option<String>,
    pub rules: Vec<String>,
    pub engine: Option<String>,
    pub untouched: bool,
}

/// Lifetime numbers that live outside history.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Extras {
    pub rules_taught: u64,
    pub dictionary_fixes: u64,
    pub dictionary_top: Option<(String, String, u64)>,
    pub snippet_uses: u64,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct LevelInfo {
    pub level: u32,
    /// A new title every five levels: 0 for levels 1-4, 1 for 5-9, and so on.
    pub title: u32,
    pub xp: u64,
    pub xp_words: u64,
    pub xp_days: u64,
    pub xp_streak: u64,
    pub need_this: u64,
    pub need_next: u64,
    pub active_days: u64,
    pub current_streak: u64,
    pub longest_streak: u64,
    pub lifetime_words: u64,
    pub lifetime_talk_min: f64,
    pub first_day: Option<String>,
}

pub const TITLES: u32 = 11;

/// XP needed to reach `level`.
pub fn need(level: u32) -> u64 {
    if level <= 10 {
        return EARLY[level as usize];
    }
    let mut n = EARLY[10];
    for k in 11..=level {
        let step = BASE_STEP * GROWTH.powi(k as i32 - 10);
        n += ((step / 50.0).round() as u64) * 50;
    }
    n
}

pub fn level_for(xp: u64) -> u32 {
    let mut level = 1;
    while level < 500 && need(level + 1) <= xp {
        level += 1;
    }
    level
}

/// Streak runs over active days: (current, longest). Current counts only when
/// the last active day is today or yesterday, the rule Statistics uses.
fn streaks(days: &[NaiveDate], today: NaiveDate) -> (u64, u64) {
    let mut longest = 0;
    let mut run = 0;
    let mut prev: Option<NaiveDate> = None;
    for d in days {
        run = match prev {
            Some(p) if (*d - p).num_days() == 1 => run + 1,
            _ => 1,
        };
        longest = longest.max(run);
        prev = Some(*d);
    }
    let current = match days.last() {
        Some(last) if (today - *last).num_days() <= 1 => run,
        _ => 0,
    };
    (current, longest)
}

fn active_days(days: &[DayRow]) -> Vec<NaiveDate> {
    let mut v: Vec<NaiveDate> = days.iter().filter(|d| d.dictations > 0).map(|d| d.day).collect();
    v.sort();
    v.dedup();
    v
}

pub fn level_info(days: &[DayRow], today: NaiveDate) -> LevelInfo {
    let active = active_days(days);
    let words: u64 = days.iter().map(|d| d.words).sum();
    let talk_ms: u64 = days.iter().map(|d| d.audio_ms).sum();
    let mut streak_xp = 0;
    let mut run = 0u64;
    let mut prev: Option<NaiveDate> = None;
    for d in &active {
        run = match prev {
            Some(p) if (*d - p).num_days() == 1 => run + 1,
            _ => 1,
        };
        streak_xp += STREAK_STEP * (run - 1).min(STREAK_CAP);
        prev = Some(*d);
    }
    let day_xp = DAY_BONUS * active.len() as u64;
    let xp = words + day_xp + streak_xp;
    let level = level_for(xp);
    let (current, longest) = streaks(&active, today);
    LevelInfo {
        level,
        title: (level / 5).min(TITLES - 1),
        xp,
        xp_words: words,
        xp_days: day_xp,
        xp_streak: streak_xp,
        need_this: need(level),
        need_next: need(level + 1),
        active_days: active.len() as u64,
        current_streak: current,
        longest_streak: longest,
        lifetime_words: words,
        lifetime_talk_min: talk_ms as f64 / 60_000.0,
        first_day: active.first().map(|d| d.to_string()),
    }
}

// ----- badges -----

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Badge {
    pub id: &'static str,
    pub value: u64,
    pub target: u64,
    pub done: bool,
}

fn is_night(h: u32) -> bool {
    h >= 22 || h <= 4
}

fn is_morning(h: u32) -> bool {
    (5..=8).contains(&h)
}

fn has_rule(r: &HistRow, label: &str) -> bool {
    r.rules.iter().any(|x| x == label)
}

fn wpm_of(r: &HistRow) -> Option<f64> {
    (r.audio_ms >= 5_000 && r.word_count >= 10).then(|| r.word_count as f64 / (r.audio_ms as f64 / 60_000.0))
}

/// Language switches between dictations, grouped by local day.
fn switches_by_day(rows: &[HistRow]) -> BTreeMap<NaiveDate, u64> {
    let mut out = BTreeMap::new();
    let mut prev: Option<&str> = None;
    for r in rows {
        let Some(lang) = r.language.as_deref() else { continue };
        if let Some(p) = prev {
            if p != lang {
                *out.entry(r.at.date()).or_insert(0) += 1;
            }
        }
        prev = Some(lang);
    }
    out
}

pub fn badges(history: &[HistRow], days: &[DayRow], extras: &Extras, today: NaiveDate) -> Vec<Badge> {
    let info = level_info(days, today);
    let night = history.iter().filter(|r| is_night(r.at.hour())).count() as u64;
    let morning = history.iter().filter(|r| is_morning(r.at.hour())).count() as u64;
    let lang_words = |code: &str| history.iter().filter(|r| r.language.as_deref() == Some(code)).map(|r| r.word_count).sum::<u64>();
    let bilingual = lang_words("el").min(lang_words("en"));
    let switch_day = switches_by_day(history).values().copied().max().unwrap_or(0);
    let fillers = history.iter().filter(|r| has_rule(r, "removed fillers")).count() as u64;
    let questions = history.iter().filter(|r| has_rule(r, "question mark") || has_rule(r, "greek question mark")).count() as u64;
    let fastest = history.iter().filter_map(wpm_of).fold(0.0f64, f64::max).round() as u64;
    let best_day = days.iter().map(|d| d.words).max().unwrap_or(0);
    let apps = history.iter().filter_map(|r| r.app_process.as_deref()).map(str::to_lowercase).collect::<HashSet<_>>().len() as u64;
    let b = |id, value: u64, target: u64| Badge { id, value: value.min(target), target, done: value >= target };
    vec![
        b("streak_7", info.longest_streak, 7),
        b("streak_30", info.longest_streak, 30),
        b("thousand_day", best_day, 1_000),
        b("night_owl", night, 25),
        b("early_bird", morning, 25),
        b("bilingual", bilingual, 1_000),
        b("code_switcher", switch_day, 10),
        b("um_buster", fillers, 100),
        b("question_master", questions, 100),
        b("speed_demon", fastest, 160),
        b("marathon", info.lifetime_talk_min.round() as u64, 120),
        b("app_hopper", apps, 10),
        b("teacher", extras.rules_taught, 10),
        b("dictionary_hero", extras.dictionary_fixes, 100),
        b("snippet_wizard", extras.snippet_uses, 50),
    ]
}

// ----- Reflect -----

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Period {
    Week,
    Month,
    All,
}

impl Period {
    pub fn parse(s: &str) -> Period {
        match s {
            "7" | "week" => Period::Week,
            "30" | "month" => Period::Month,
            _ => Period::All,
        }
    }
    fn days(self) -> Option<i64> {
        match self {
            Period::Week => Some(7),
            Period::Month => Some(30),
            Period::All => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Fastest {
    pub wpm: u64,
    pub words: u64,
    pub seconds: u64,
    pub day: String,
    /// None when the app is in the sensitive category (banking, passwords).
    pub app: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Session {
    pub minutes: u64,
    pub dictations: u64,
    pub day: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TopApp {
    /// None when the app is in the sensitive category.
    pub name: Option<String>,
    pub share: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DayPoint {
    pub day: String,
    pub words: u64,
    pub dictations: u64,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct Reflect {
    pub period: String,
    pub start_day: String,
    pub end_day: String,
    pub days_span: u64,
    /// The window was cut short because the app is younger than the period.
    pub since_first_use: bool,
    pub active_days: u64,
    /// Personas and awards wait for 7 active days or 1,000 words in all.
    pub enough_data: bool,
    pub dictations: u64,
    pub words: u64,
    pub avg_words: f64,
    pub speaking_min: f64,
    pub wpm: f64,
    pub time_saved_min: f64,
    pub keystrokes: u64,
    pub accents: u64,
    pub fastest: Option<Fastest>,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub inference_min: f64,
    pub all_local: bool,
    pub power_hour: Option<u32>,
    pub power_hour_share: f64,
    pub night_share: f64,
    pub morning_share: f64,
    pub by_hour: Vec<u64>,
    pub best_day: Option<DayPoint>,
    /// 0 = Monday.
    pub top_weekday: Option<u32>,
    pub weekend_share: f64,
    pub longest_session: Option<Session>,
    pub languages: Vec<(String, u64)>,
    pub switches: u64,
    pub top_app: Option<TopApp>,
    pub ai_over_people: bool,
    pub persona: String,
    pub apps: u64,
    pub fillers: u64,
    pub stutters: u64,
    pub repeats: u64,
    pub questions: u64,
    pub greek_questions: u64,
    pub self_corrections: u64,
    pub dictionary_fixes: u64,
    pub dictionary_top: Option<(String, String, u64)>,
    pub untouched_share: f64,
    /// Words in the same number of days just before this window, only when the
    /// app already ran on the first of those days. Otherwise a rise or a drop
    /// would be made up.
    pub prev_words: Option<u64>,
    pub window_words: u64,
    pub daily: Vec<DayPoint>,
}

const ACCENTED: &str = "άέήίόύώΐΰΆΈΉΊΌΎΏ";
const SESSION_GAP_MIN: i64 = 5;

fn share(part: u64, total: u64) -> f64 {
    if total == 0 { 0.0 } else { part as f64 / total as f64 }
}

fn percentile(sorted: &[u64], p: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let i = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

fn persona(by_cat: &HashMap<&str, u64>) -> String {
    let mut v: Vec<(&str, u64)> = by_cat
        .iter()
        .filter(|(c, _)| !matches!(**c, "unknown" | "sensitive" | ""))
        .map(|(c, w)| (*c, *w))
        .collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let total: u64 = v.iter().map(|x| x.1).sum();
    let Some(first) = v.first() else { return "all_rounder".into() };
    let top = share(first.1, total);
    let close = v.get(1).is_some_and(|s| top - share(s.1, total) < 0.05);
    if top < 0.30 || close {
        return "all_rounder".into();
    }
    match first.0 {
        "ai_chat" | "chat" | "email" | "document" | "code" | "browser" | "terminal" => first.0.to_string(),
        _ => "all_rounder".into(),
    }
}

pub fn reflect(history: &[HistRow], days: &[DayRow], extras: &Extras, period: Period, now: NaiveDateTime) -> Reflect {
    let today = now.date();
    let info = level_info(days, today);
    let first = active_days(days).first().copied().or_else(|| history.first().map(|r| r.at.date())).unwrap_or(today);
    let (start, since_first_use) = match period.days() {
        Some(n) => {
            let s = today - Duration::days(n - 1);
            if first > s { (first, true) } else { (s, false) }
        }
        None => (first.min(today), false),
    };
    let w: Vec<&HistRow> = history.iter().filter(|r| r.at.date() >= start && r.at.date() <= today).collect();
    let dw: Vec<&DayRow> = days.iter().filter(|d| d.day >= start && d.day <= today).collect();

    let words: u64 = w.iter().map(|r| r.word_count).sum();
    let dictations = w.len() as u64;
    let audio_ms: u64 = w.iter().map(|r| r.audio_ms).sum();
    let speaking_min = audio_ms as f64 / 60_000.0;
    let wpm = if speaking_min > 0.0 { words as f64 / speaking_min } else { 0.0 };
    let time_saved_min = (words as f64 / TYPING_WPM - speaking_min).max(0.0);

    let fastest = w
        .iter()
        .filter_map(|r| wpm_of(r).map(|v| (v, *r)))
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(v, r)| Fastest {
            wpm: v.round() as u64,
            words: r.word_count,
            seconds: (r.audio_ms as f64 / 1000.0).round() as u64,
            day: r.at.date().to_string(),
            app: (r.app_category != "sensitive").then(|| r.app_name.clone()).flatten(),
        });

    let mut lat: Vec<u64> = w.iter().map(|r| r.latency_ms).filter(|v| *v > 0).collect();
    lat.sort_unstable();

    let mut by_hour = vec![0u64; 24];
    let mut by_weekday = [0u64; 7];
    for r in &w {
        by_hour[r.at.hour() as usize] += r.word_count;
        by_weekday[r.at.weekday().num_days_from_monday() as usize] += r.word_count;
    }
    let power_hour = (words > 0).then(|| (0..24).max_by_key(|h| (by_hour[*h as usize], 24 - *h)).unwrap_or(0));
    let night: u64 = (0..24u32).filter(|h| is_night(*h)).map(|h| by_hour[h as usize]).sum();
    let morning: u64 = (0..24u32).filter(|h| is_morning(*h)).map(|h| by_hour[h as usize]).sum();
    let top_weekday = (words > 0).then(|| (0..7u32).max_by_key(|d| (by_weekday[*d as usize], 7 - *d)).unwrap_or(0));

    // Sessions: a pause longer than five minutes starts a new one.
    let mut longest_session: Option<Session> = None;
    let mut i = 0;
    while i < w.len() {
        let mut j = i;
        while j + 1 < w.len() && (w[j + 1].at - w[j].at).num_minutes() <= SESSION_GAP_MIN {
            j += 1;
        }
        let span = (w[j].at - w[i].at).num_seconds().max(0) as u64 + w[j].audio_ms / 1000;
        let minutes = (span as f64 / 60.0).round() as u64;
        let n = (j - i + 1) as u64;
        if n >= 2 && longest_session.as_ref().is_none_or(|s| minutes > s.minutes) {
            longest_session = Some(Session { minutes, dictations: n, day: w[i].at.date().to_string() });
        }
        i = j + 1;
    }

    let mut langs: HashMap<String, u64> = HashMap::new();
    for r in &w {
        if let Some(l) = &r.language {
            *langs.entry(l.clone()).or_insert(0) += r.word_count;
        }
    }
    let mut languages: Vec<(String, u64)> = langs.into_iter().collect();
    languages.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let owned: Vec<HistRow> = w.iter().map(|r| (*r).clone()).collect();
    let switches: u64 = switches_by_day(&owned).values().sum();

    let mut by_app: HashMap<String, (u64, bool, String)> = HashMap::new();
    let mut by_cat: HashMap<&str, u64> = HashMap::new();
    for r in &w {
        *by_cat.entry(r.app_category.as_str()).or_insert(0) += r.word_count;
        let key = r.app_process.clone().unwrap_or_default().to_lowercase();
        let e = by_app.entry(key).or_insert((0, false, r.app_name.clone().unwrap_or_default()));
        e.0 += r.word_count;
        e.1 |= r.app_category == "sensitive";
    }
    let top_app = by_app
        .iter()
        .filter(|(k, _)| !k.is_empty())
        .max_by(|a, b| a.1 .0.cmp(&b.1 .0).then(b.0.cmp(a.0)))
        .map(|(_, (n, sensitive, name))| TopApp { name: (!sensitive && !name.is_empty()).then(|| name.clone()), share: share(*n, words) });
    let cat = |c: &str| by_cat.get(c).copied().unwrap_or(0);
    let ai_over_people = cat("ai_chat") > cat("chat") + cat("email");
    let apps = by_app.keys().filter(|k| !k.is_empty()).count() as u64;

    let count = |label: &str| w.iter().filter(|r| has_rule(r, label)).count() as u64;
    let mut dict: HashMap<&str, u64> = HashMap::new();
    for r in &w {
        for a in &r.rules {
            if let Some(rest) = a.strip_prefix("dictionary: ") {
                *dict.entry(rest).or_insert(0) += 1;
            }
        }
    }
    let dictionary_fixes: u64 = dict.values().sum();
    let dictionary_top = dict
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .and_then(|(k, n)| k.split_once(" -> ").map(|(x, y)| (x.to_string(), y.to_string(), *n)));

    let daily: Vec<DayPoint> = {
        let mut out = Vec::new();
        let mut d = start;
        while d <= today {
            let row = dw.iter().find(|x| x.day == d);
            out.push(DayPoint { day: d.to_string(), words: row.map_or(0, |x| x.words), dictations: row.map_or(0, |x| x.dictations) });
            d += Duration::days(1);
        }
        out
    };
    let best_day = dw.iter().filter(|d| d.words > 0).max_by(|a, b| a.words.cmp(&b.words).then(b.day.cmp(&a.day))).map(|d| DayPoint {
        day: d.day.to_string(),
        words: d.words,
        dictations: d.dictations,
    });
    let window_words: u64 = dw.iter().map(|d| d.words).sum();
    let prev_words = match period.days() {
        Some(n) if !since_first_use => {
            let ps = start - Duration::days(n);
            let pe = start - Duration::days(1);
            (first <= ps).then(|| days.iter().filter(|d| d.day >= ps && d.day <= pe).map(|d| d.words).sum())
        }
        _ => None,
    };

    let weekend = by_weekday[5] + by_weekday[6];
    Reflect {
        period: match period {
            Period::Week => "7",
            Period::Month => "30",
            Period::All => "all",
        }
        .into(),
        start_day: start.to_string(),
        end_day: today.to_string(),
        days_span: ((today - start).num_days() + 1).max(1) as u64,
        since_first_use,
        active_days: dw.iter().filter(|d| d.dictations > 0).count() as u64,
        enough_data: info.active_days >= 7 || info.lifetime_words >= 1_000,
        dictations,
        words,
        avg_words: if dictations > 0 { words as f64 / dictations as f64 } else { 0.0 },
        speaking_min,
        wpm,
        time_saved_min,
        keystrokes: w.iter().map(|r| r.final_text.chars().count() as u64).sum(),
        accents: w.iter().map(|r| r.final_text.chars().filter(|c| ACCENTED.contains(*c)).count() as u64).sum(),
        fastest,
        p50_ms: percentile(&lat, 0.5),
        p95_ms: percentile(&lat, 0.95),
        inference_min: w.iter().map(|r| r.inference_ms).sum::<u64>() as f64 / 60_000.0,
        // Some older rows carry an empty engine name; only a named
        // remote engine makes the line false.
        all_local: !w.is_empty() && !w.iter().any(|r| r.engine.as_deref().is_some_and(|e| !e.is_empty() && e != "whisper_local")),
        power_hour,
        power_hour_share: power_hour.map_or(0.0, |h| share(by_hour[h as usize], words)),
        night_share: share(night, words),
        morning_share: share(morning, words),
        by_hour,
        best_day,
        top_weekday,
        weekend_share: share(weekend, words),
        longest_session,
        languages,
        switches,
        top_app,
        ai_over_people,
        persona: persona(&by_cat),
        apps,
        fillers: count("removed fillers"),
        stutters: count("removed a stutter"),
        repeats: count("removed repeated word"),
        questions: w.iter().filter(|r| has_rule(r, "question mark") || has_rule(r, "greek question mark")).count() as u64,
        greek_questions: count("greek question mark"),
        self_corrections: count("resolved self-correction"),
        dictionary_fixes,
        dictionary_top: dictionary_top.or_else(|| if period == Period::All { extras.dictionary_top.clone() } else { None }),
        untouched_share: share(w.iter().filter(|r| r.untouched).count() as u64, dictations),
        prev_words,
        window_words,
        daily,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Prints the Milestones page for a COPY of a real database, to check the
    /// numbers before a build. Run with FYF_MS_DB=<copy> FYF_MS_OUT=<json>
    /// `cargo test --lib milestones_from_a_copy -- --ignored`. Never point it
    /// at the live file: opening runs the schema upgrade.
    #[test]
    #[ignore]
    fn milestones_from_a_copy() {
        let (Ok(db), Ok(out)) = (std::env::var("FYF_MS_DB"), std::env::var("FYF_MS_OUT")) else { return };
        let db = crate::db::Db::open(std::path::Path::new(&db)).unwrap();
        let days = db.milestone_days().unwrap();
        let history = db.milestone_history().unwrap();
        let extras = db.milestone_extras().unwrap();
        let now = chrono::Local::now().naive_local();
        let mut all = serde_json::Map::new();
        all.insert("level".into(), serde_json::to_value(level_info(&days, now.date())).unwrap());
        all.insert("badges".into(), serde_json::to_value(badges(&history, &days, &extras, now.date())).unwrap());
        for p in ["7", "30", "all"] {
            all.insert(format!("reflect_{p}"), serde_json::to_value(reflect(&history, &days, &extras, Period::parse(p), now)).unwrap());
        }
        std::fs::write(out, serde_json::to_string_pretty(&all).unwrap()).unwrap();
    }

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn at(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").unwrap()
    }

    fn day(s: &str, words: u64) -> DayRow {
        DayRow { day: d(s), words, dictations: if words > 0 { 1 } else { 0 }, audio_ms: words * 400 }
    }

    fn row(when: &str, words: u64, text: &str) -> HistRow {
        HistRow {
            at: at(when),
            final_text: text.into(),
            word_count: words,
            audio_ms: words * 400,
            latency_ms: 900,
            inference_ms: 300,
            app_name: Some("Notepad".into()),
            app_process: Some("notepad.exe".into()),
            app_category: "document".into(),
            language: Some("el".into()),
            rules: vec![],
            engine: Some("whisper_local".into()),
            untouched: true,
        }
    }

    #[test]
    fn the_curve_is_quick_at_first_and_grows_after_ten() {
        assert_eq!(need(1), 0);
        assert_eq!(need(2), 50);
        assert_eq!(need(10), 10_000);
        // Each step after ten is longer than the one before it.
        let mut last = need(10) - need(9);
        for l in 11..40 {
            let step = need(l) - need(l - 1);
            assert!(step > last, "step to {l} is {step}, before it {last}");
            last = step;
        }
        assert_eq!(level_for(0), 1);
        assert_eq!(level_for(49), 1);
        assert_eq!(level_for(50), 2);
        assert_eq!(level_for(need(17)), 17);
        assert_eq!(level_for(need(17) - 1), 16);
    }

    #[test]
    fn a_first_day_of_a_few_hundred_words_already_climbs() {
        let info = level_info(&[day("2026-03-02", 400)], d("2026-03-02"));
        assert_eq!(info.xp, 400 + DAY_BONUS);
        assert_eq!(info.level, 5);
        assert_eq!(info.title, 1);
        assert_eq!(info.current_streak, 1);
    }

    #[test]
    fn days_in_a_row_earn_a_growing_bonus_and_a_gap_resets_it() {
        let days = [day("2026-03-01", 10), day("2026-03-02", 10), day("2026-03-03", 10), day("2026-03-05", 10)];
        let info = level_info(&days, d("2026-03-05"));
        assert_eq!(info.xp_days, 4 * DAY_BONUS);
        assert_eq!(info.xp_streak, STREAK_STEP + 2 * STREAK_STEP);
        assert_eq!((info.current_streak, info.longest_streak), (1, 3));
        // An empty day does not count as active.
        let info = level_info(&[day("2026-03-01", 0)], d("2026-03-01"));
        assert_eq!((info.xp, info.active_days, info.level), (0, 0, 1));
    }

    #[test]
    fn the_streak_bonus_stops_growing_at_the_cap() {
        let start = d("2026-01-01");
        let days: Vec<DayRow> = (0..20).map(|i| DayRow { day: start + Duration::days(i), words: 0, dictations: 1, audio_ms: 0 }).collect();
        let info = level_info(&days, start + Duration::days(19));
        let expected: u64 = (1..20u64).map(|k| STREAK_STEP * k.min(STREAK_CAP)).sum();
        assert_eq!(info.xp_streak, expected);
        assert_eq!(info.current_streak, 20);
    }

    #[test]
    fn a_young_install_reflects_from_its_first_day() {
        let days = [day("2026-03-10", 100), day("2026-03-12", 200)];
        let hist = [row("2026-03-10 09:00", 100, "Καλημέρα σε όλους."), row("2026-03-12 23:30", 200, "Good night.")];
        let r = reflect(&hist, &days, &Extras::default(), Period::Month, at("2026-03-12 23:59"));
        assert!(r.since_first_use);
        assert_eq!((r.start_day.as_str(), r.days_span), ("2026-03-10", 3));
        assert_eq!((r.words, r.dictations, r.active_days), (300, 2, 2));
        assert_eq!(r.daily.len(), 3);
        assert_eq!(r.prev_words, None);
        assert_eq!(r.power_hour, Some(23));
        assert!((r.night_share - 200.0 / 300.0).abs() < 1e-9);
    }

    #[test]
    fn the_previous_period_is_shown_only_when_it_is_fully_covered() {
        let start = d("2026-02-01");
        let days: Vec<DayRow> = (0..14).map(|i| day(&(start + Duration::days(i)).to_string(), 10)).collect();
        let r = reflect(&[], &days, &Extras::default(), Period::Week, at("2026-02-14 12:00"));
        assert_eq!(r.prev_words, Some(70));
        assert_eq!(r.window_words, 70);
        let r = reflect(&[], &days[1..], &Extras::default(), Period::Week, at("2026-02-14 12:00"));
        assert_eq!(r.prev_words, None);
    }

    #[test]
    fn cleanup_counts_are_per_dictation_and_dictionary_fixes_are_per_rule() {
        let mut a = row("2026-03-01 10:00", 12, "Θα τα πούμε αύριο;");
        a.rules = vec!["removed fillers".into(), "greek question mark".into(), "dictionary: Κλοντ -> Claude".into(), "dictionary: Κλοντ -> Claude".into()];
        let mut b = row("2026-03-01 10:02", 8, "Ready when you are?");
        b.rules = vec!["removed fillers".into(), "question mark".into(), "resolved self-correction".into()];
        b.language = Some("en".into());
        let r = reflect(&[a, b], &[day("2026-03-01", 20)], &Extras::default(), Period::All, at("2026-03-01 18:00"));
        assert_eq!((r.fillers, r.questions, r.greek_questions, r.self_corrections), (2, 2, 1, 1));
        assert_eq!(r.dictionary_fixes, 2);
        assert_eq!(r.dictionary_top, Some(("Κλοντ".into(), "Claude".into(), 2)));
        assert_eq!(r.switches, 1);
        assert_eq!(r.accents, 2);
        assert_eq!(r.longest_session.as_ref().map(|s| s.dictations), Some(2));
    }

    #[test]
    fn a_sensitive_app_is_never_named() {
        let mut a = row("2026-03-01 10:00", 50, "Ένα κείμενο");
        a.app_name = Some("My Bank".into());
        a.app_process = Some("bank.exe".into());
        a.app_category = "sensitive".into();
        a.audio_ms = 10_000;
        let r = reflect(&[a], &[day("2026-03-01", 50)], &Extras::default(), Period::All, at("2026-03-01 18:00"));
        assert_eq!(r.top_app.as_ref().map(|t| t.name.clone()), Some(None));
        assert_eq!(r.fastest.as_ref().map(|f| f.app.clone()), Some(None));
        assert_eq!(r.fastest.as_ref().map(|f| f.wpm), Some(300));
    }

    #[test]
    fn the_persona_follows_the_category_with_most_words_unless_it_is_close() {
        let mut m = HashMap::new();
        m.insert("ai_chat", 700u64);
        m.insert("chat", 300);
        assert_eq!(persona(&m), "ai_chat");
        m.insert("chat", 680);
        assert_eq!(persona(&m), "all_rounder");
        let mut only_hidden = HashMap::new();
        only_hidden.insert("sensitive", 900u64);
        assert_eq!(persona(&only_hidden), "all_rounder");
    }

    #[test]
    fn badges_fill_up_to_their_target() {
        let mut rows = vec![];
        for i in 0..30 {
            rows.push(row(&format!("2026-03-01 23:{:02}", i), 40, "Αργά το βράδυ."));
        }
        let days = [day("2026-03-01", 1_200)];
        let extras = Extras { rules_taught: 3, ..Default::default() };
        let b = badges(&rows, &days, &extras, d("2026-03-01"));
        let get = |id: &str| b.iter().find(|x| x.id == id).unwrap().clone();
        assert!(get("night_owl").done);
        assert!(!get("early_bird").done);
        assert!(get("thousand_day").done);
        assert_eq!((get("teacher").value, get("teacher").done), (3, false));
        assert_eq!(get("streak_30").value, 1);
        assert_eq!(b.len(), 15);
    }
}
