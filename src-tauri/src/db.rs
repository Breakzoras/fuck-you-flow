//! Local SQLite store: history, dictionary, snippets, learning, app styles, stats.
//! Migrations are numbered and applied in order; a backup copy is taken before
//! every migration that changes an existing table.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

pub struct Db {
    conn: Mutex<Connection>,
}

const MIGRATIONS: &[&str] = &[
    // 1
    r#"
    CREATE TABLE IF NOT EXISTS history (
        id TEXT PRIMARY KEY,
        created_at TEXT NOT NULL,
        raw_text TEXT NOT NULL,
        cleaned_text TEXT NOT NULL,
        final_text TEXT NOT NULL,
        language TEXT NOT NULL,
        detected_language TEXT,
        app_name TEXT,
        app_process TEXT,
        app_category TEXT,
        cleanup_mode TEXT NOT NULL,
        rules_applied TEXT NOT NULL DEFAULT '[]',
        audio_ms INTEGER NOT NULL DEFAULT 0,
        latency_ms INTEGER NOT NULL DEFAULT 0,
        inference_ms INTEGER NOT NULL DEFAULT 0,
        word_count INTEGER NOT NULL DEFAULT 0,
        engine TEXT,
        model TEXT,
        insertion_method TEXT,
        status TEXT NOT NULL,
        audio_path TEXT,
        context_used INTEGER NOT NULL DEFAULT 0,
        retried INTEGER NOT NULL DEFAULT 0,
        undone INTEGER NOT NULL DEFAULT 0,
        edited_text TEXT
    );
    CREATE INDEX IF NOT EXISTS history_created ON history(created_at DESC);
    CREATE TABLE IF NOT EXISTS dictionary (
        id TEXT PRIMARY KEY,
        wrong TEXT NOT NULL,
        correct TEXT NOT NULL,
        match_mode TEXT NOT NULL DEFAULT 'whole_word',
        case_sensitive INTEGER NOT NULL DEFAULT 0,
        language TEXT,
        app_scope TEXT,
        enabled INTEGER NOT NULL DEFAULT 1,
        use_as_hint INTEGER NOT NULL DEFAULT 1,
        source TEXT NOT NULL DEFAULT 'user',
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        apply_count INTEGER NOT NULL DEFAULT 0,
        last_applied_at TEXT
    );
    CREATE TABLE IF NOT EXISTS dictionary_exceptions (
        id TEXT PRIMARY KEY,
        rule_id TEXT NOT NULL,
        context TEXT NOT NULL,
        created_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS snippets (
        id TEXT PRIMARY KEY,
        trigger TEXT NOT NULL,
        expansion TEXT NOT NULL,
        enabled INTEGER NOT NULL DEFAULT 1,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        apply_count INTEGER NOT NULL DEFAULT 0,
        last_applied_at TEXT
    );
    CREATE TABLE IF NOT EXISTS learning_events (
        id TEXT PRIMARY KEY,
        history_id TEXT,
        kind TEXT NOT NULL,
        before_text TEXT NOT NULL,
        after_text TEXT NOT NULL,
        created_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS suggestions (
        id TEXT PRIMARY KEY,
        kind TEXT NOT NULL,
        wrong TEXT NOT NULL,
        correct TEXT NOT NULL,
        evidence_count INTEGER NOT NULL DEFAULT 1,
        reason TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'pending',
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS app_styles (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        process_match TEXT NOT NULL,
        category TEXT NOT NULL,
        trailing_punctuation INTEGER NOT NULL DEFAULT 1,
        capitalize_first INTEGER NOT NULL DEFAULT 1,
        tone TEXT NOT NULL DEFAULT 'neutral',
        enabled INTEGER NOT NULL DEFAULT 1
    );
    CREATE TABLE IF NOT EXISTS stats_daily (
        day TEXT PRIMARY KEY,
        dictations INTEGER NOT NULL DEFAULT 0,
        words INTEGER NOT NULL DEFAULT 0,
        audio_ms INTEGER NOT NULL DEFAULT 0,
        corrections INTEGER NOT NULL DEFAULT 0,
        retries INTEGER NOT NULL DEFAULT 0,
        edits INTEGER NOT NULL DEFAULT 0
    );
    "#,
];

fn now() -> String {
    Utc::now().to_rfc3339()
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub created_at: String,
    pub raw_text: String,
    pub cleaned_text: String,
    pub final_text: String,
    pub language: String,
    pub detected_language: Option<String>,
    pub app_name: Option<String>,
    pub app_process: Option<String>,
    pub app_category: Option<String>,
    pub cleanup_mode: String,
    pub rules_applied: Vec<String>,
    pub audio_ms: u64,
    pub latency_ms: u64,
    pub inference_ms: u64,
    pub word_count: u64,
    pub engine: Option<String>,
    pub model: Option<String>,
    pub insertion_method: Option<String>,
    pub status: String,
    pub audio_path: Option<String>,
    pub context_used: bool,
    pub retried: bool,
    pub undone: bool,
    pub edited_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryRule {
    pub id: String,
    pub wrong: String,
    pub correct: String,
    /// whole_word | phrase | exact
    pub match_mode: String,
    pub case_sensitive: bool,
    pub language: Option<String>,
    pub app_scope: Option<String>,
    pub enabled: bool,
    pub use_as_hint: bool,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
    pub apply_count: u64,
    pub last_applied_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: String,
    pub trigger: String,
    pub expansion: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
    pub apply_count: u64,
    pub last_applied_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub id: String,
    pub kind: String,
    pub wrong: String,
    pub correct: String,
    pub evidence_count: u64,
    pub reason: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStyle {
    pub id: String,
    pub name: String,
    pub process_match: String,
    pub category: String,
    pub trailing_punctuation: bool,
    pub capitalize_first: bool,
    pub tone: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyStat {
    pub day: String,
    pub dictations: u64,
    pub words: u64,
    pub audio_ms: u64,
    pub corrections: u64,
    pub retries: u64,
    pub edits: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatsSummary {
    pub total_words: u64,
    pub total_minutes: f64,
    pub avg_wpm: f64,
    pub dictations: u64,
    pub corrections_applied: u64,
    pub retries: u64,
    pub edits: u64,
    pub current_streak: u64,
    pub longest_streak: u64,
    pub time_saved_minutes: f64,
    pub time_saved_formula: String,
    pub p50_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub by_category: Vec<(String, u64)>,
    pub daily: Vec<DailyStat>,
}

/// Copies `lalia.db` to `backup/lalia-YYYY-MM-DD.db` once per day and prunes
/// copies older than 14 days. Only the main file is copied; with a rollback
/// journal that file is complete whenever no transaction is open, which is the
/// case before the connection is created.
fn daily_backup(path: &Path) -> anyhow::Result<()> {
    let dir = path.parent().map(|p| p.join("backup")).unwrap_or_else(|| PathBuf::from("backup"));
    std::fs::create_dir_all(&dir)?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let target = dir.join(format!("lalia-{today}.db"));
    if !target.exists() {
        std::fs::copy(path, &target)?;
        tracing::info!("database backup written: {}", target.display());
    }
    let cutoff = chrono::Local::now() - chrono::Duration::days(14);
    for entry in std::fs::read_dir(&dir)?.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(date) = name.strip_prefix("lalia-").and_then(|n| n.strip_suffix(".db")) {
            if let Ok(d) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                if d < cutoff.date_naive() {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }
    Ok(())
}

impl Db {
    pub fn open(path: &Path) -> anyhow::Result<Db> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Daily safety copy of the single database file, taken before anything
        // else touches it (kept for 14 days). Cheap: the file is a few hundred KB.
        if path.exists() {
            if let Err(e) = daily_backup(path) {
                tracing::warn!("database backup skipped: {e}");
            }
        }
        let conn = Connection::open(path)?;
        // One file, no write-ahead log. On 2026-09-06 the app came up after a
        // reboot seeing an empty database while every row was still on disk in
        // the WAL; a rollback journal keeps the main file the only source of truth.
        // journal_mode=DELETE and synchronous=FULL are deliberate and stay:
        // a write-ahead log held a whole day of history and lost it on
        // 6 September 2026. busy_timeout is the missing piece; without it a
        // second connection meeting a locked file fails at once instead of
        // waiting the moment out.
        conn.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL; PRAGMA busy_timeout=5000;")?;
        let db = Db { conn: Mutex::new(conn) };
        db.migrate(path)?;
        {
            let c = db.conn.lock();
            let history: i64 = c.query_row("SELECT count(*) FROM history", [], |r| r.get(0)).unwrap_or(-1);
            let rules: i64 = c.query_row("SELECT count(*) FROM dictionary", [], |r| r.get(0)).unwrap_or(-1);
            let mode: String = c.query_row("PRAGMA journal_mode", [], |r| r.get(0)).unwrap_or_default();
            tracing::info!("database opened: {} ({history} history rows, {rules} dictionary rules, journal {mode})", path.display());
        }
        Ok(db)
    }

    pub fn open_in_memory() -> anyhow::Result<Db> {
        let conn = Connection::open_in_memory()?;
        let db = Db { conn: Mutex::new(conn) };
        db.migrate(Path::new(":memory:"))?;
        Ok(db)
    }

    fn migrate(&self, path: &Path) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute_batch("CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);")?;
        let current: i64 = conn.query_row("SELECT COALESCE(MAX(version), 0) FROM schema_version", [], |r| r.get(0))?;
        for (i, sql) in MIGRATIONS.iter().enumerate() {
            let v = (i + 1) as i64;
            if v <= current {
                continue;
            }
            if current > 0 && path.exists() {
                let backup = path.with_extension(format!("pre-v{v}.bak"));
                let _ = std::fs::copy(path, backup);
            }
            conn.execute_batch(sql)?;
            conn.execute("INSERT INTO schema_version(version) VALUES (?1)", params![v])?;
            tracing::info!("database migrated to v{v}");
        }
        Ok(())
    }

    // ----- history -----

    pub fn insert_history(&self, e: &HistoryEntry) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO history (id, created_at, raw_text, cleaned_text, final_text, language, detected_language, app_name, app_process, app_category, cleanup_mode, rules_applied, audio_ms, latency_ms, inference_ms, word_count, engine, model, insertion_method, status, audio_path, context_used, retried, undone, edited_text)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25)",
            params![
                e.id, e.created_at, e.raw_text, e.cleaned_text, e.final_text, e.language, e.detected_language, e.app_name, e.app_process, e.app_category,
                e.cleanup_mode, serde_json::to_string(&e.rules_applied)?, e.audio_ms as i64, e.latency_ms as i64, e.inference_ms as i64, e.word_count as i64,
                e.engine, e.model, e.insertion_method, e.status, e.audio_path, e.context_used as i64, e.retried as i64, e.undone as i64, e.edited_text
            ],
        )?;
        Ok(())
    }

    fn row_to_history(r: &rusqlite::Row) -> rusqlite::Result<HistoryEntry> {
        let rules: String = r.get("rules_applied")?;
        Ok(HistoryEntry {
            id: r.get("id")?,
            created_at: r.get("created_at")?,
            raw_text: r.get("raw_text")?,
            cleaned_text: r.get("cleaned_text")?,
            final_text: r.get("final_text")?,
            language: r.get("language")?,
            detected_language: r.get("detected_language")?,
            app_name: r.get("app_name")?,
            app_process: r.get("app_process")?,
            app_category: r.get("app_category")?,
            cleanup_mode: r.get("cleanup_mode")?,
            rules_applied: serde_json::from_str(&rules).unwrap_or_default(),
            audio_ms: r.get::<_, i64>("audio_ms")? as u64,
            latency_ms: r.get::<_, i64>("latency_ms")? as u64,
            inference_ms: r.get::<_, i64>("inference_ms")? as u64,
            word_count: r.get::<_, i64>("word_count")? as u64,
            engine: r.get("engine")?,
            model: r.get("model")?,
            insertion_method: r.get("insertion_method")?,
            status: r.get("status")?,
            audio_path: r.get("audio_path")?,
            context_used: r.get::<_, i64>("context_used")? != 0,
            retried: r.get::<_, i64>("retried")? != 0,
            undone: r.get::<_, i64>("undone")? != 0,
            edited_text: r.get("edited_text")?,
        })
    }

    pub fn list_history(&self, search: Option<&str>, limit: u32, offset: u32) -> anyhow::Result<Vec<HistoryEntry>> {
        let conn = self.conn.lock();
        let mut out = Vec::new();
        match search.map(|s| s.trim()).filter(|s| !s.is_empty()) {
            Some(q) => {
                let like = format!("%{q}%");
                let mut st = conn.prepare("SELECT * FROM history WHERE raw_text LIKE ?1 OR final_text LIKE ?1 OR edited_text LIKE ?1 OR app_name LIKE ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3")?;
                let rows = st.query_map(params![like, limit, offset], Self::row_to_history)?;
                for r in rows {
                    out.push(r?);
                }
            }
            None => {
                let mut st = conn.prepare("SELECT * FROM history ORDER BY created_at DESC LIMIT ?1 OFFSET ?2")?;
                let rows = st.query_map(params![limit, offset], Self::row_to_history)?;
                for r in rows {
                    out.push(r?);
                }
            }
        }
        Ok(out)
    }

    pub fn get_history(&self, id: &str) -> anyhow::Result<Option<HistoryEntry>> {
        let conn = self.conn.lock();
        Ok(conn.query_row("SELECT * FROM history WHERE id = ?1", params![id], Self::row_to_history).optional()?)
    }

    pub fn last_successful_history(&self) -> anyhow::Result<Option<HistoryEntry>> {
        let conn = self.conn.lock();
        Ok(conn
            .query_row("SELECT * FROM history WHERE status IN ('success','copied') ORDER BY created_at DESC LIMIT 1", [], Self::row_to_history)
            .optional()?)
    }

    pub fn delete_history(&self, id: &str) -> anyhow::Result<Option<String>> {
        let conn = self.conn.lock();
        let audio: Option<String> = conn.query_row("SELECT audio_path FROM history WHERE id = ?1", params![id], |r| r.get(0)).optional()?.flatten();
        conn.execute("DELETE FROM history WHERE id = ?1", params![id])?;
        Ok(audio)
    }

    pub fn delete_all_history(&self) -> anyhow::Result<Vec<String>> {
        let conn = self.conn.lock();
        let mut st = conn.prepare("SELECT audio_path FROM history WHERE audio_path IS NOT NULL")?;
        let paths: Vec<String> = st.query_map([], |r| r.get::<_, String>(0))?.filter_map(|r| r.ok()).collect();
        conn.execute("DELETE FROM history", [])?;
        Ok(paths)
    }

    pub fn delete_history_older_than(&self, days: u32) -> anyhow::Result<Vec<String>> {
        let cutoff = (Utc::now() - chrono::Duration::days(days as i64)).to_rfc3339();
        let conn = self.conn.lock();
        let mut st = conn.prepare("SELECT audio_path FROM history WHERE created_at < ?1 AND audio_path IS NOT NULL")?;
        let paths: Vec<String> = st.query_map(params![cutoff], |r| r.get::<_, String>(0))?.filter_map(|r| r.ok()).collect();
        conn.execute("DELETE FROM history WHERE created_at < ?1", params![cutoff])?;
        Ok(paths)
    }

    pub fn mark_history(&self, id: &str, field: &str, value: bool) -> anyhow::Result<()> {
        let col = match field {
            "retried" => "retried",
            "undone" => "undone",
            _ => anyhow::bail!("unknown field"),
        };
        let conn = self.conn.lock();
        conn.execute(&format!("UPDATE history SET {col} = ?1 WHERE id = ?2"), params![value as i64, id])?;
        Ok(())
    }

    pub fn set_history_edit(&self, id: &str, edited: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute("UPDATE history SET edited_text = ?1 WHERE id = ?2", params![edited, id])?;
        Ok(())
    }

    pub fn update_history_texts(&self, id: &str, cleaned: &str, final_text: &str, rules: &[String]) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE history SET cleaned_text = ?1, final_text = ?2, rules_applied = ?3 WHERE id = ?4",
            params![cleaned, final_text, serde_json::to_string(rules)?, id],
        )?;
        Ok(())
    }

    // ----- dictionary -----

    fn row_to_rule(r: &rusqlite::Row) -> rusqlite::Result<DictionaryRule> {
        Ok(DictionaryRule {
            id: r.get("id")?,
            wrong: r.get("wrong")?,
            correct: r.get("correct")?,
            match_mode: r.get("match_mode")?,
            case_sensitive: r.get::<_, i64>("case_sensitive")? != 0,
            language: r.get("language")?,
            app_scope: r.get("app_scope")?,
            enabled: r.get::<_, i64>("enabled")? != 0,
            use_as_hint: r.get::<_, i64>("use_as_hint")? != 0,
            source: r.get("source")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            apply_count: r.get::<_, i64>("apply_count")? as u64,
            last_applied_at: r.get("last_applied_at")?,
        })
    }

    pub fn list_rules(&self) -> anyhow::Result<Vec<DictionaryRule>> {
        let conn = self.conn.lock();
        let mut st = conn.prepare("SELECT * FROM dictionary ORDER BY updated_at DESC")?;
        let rows = st.query_map([], Self::row_to_rule)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn upsert_rule(&self, rule: &DictionaryRule) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO dictionary (id, wrong, correct, match_mode, case_sensitive, language, app_scope, enabled, use_as_hint, source, created_at, updated_at, apply_count, last_applied_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
             ON CONFLICT(id) DO UPDATE SET wrong=excluded.wrong, correct=excluded.correct, match_mode=excluded.match_mode, case_sensitive=excluded.case_sensitive,
             language=excluded.language, app_scope=excluded.app_scope, enabled=excluded.enabled, use_as_hint=excluded.use_as_hint, updated_at=excluded.updated_at",
            params![
                rule.id, rule.wrong, rule.correct, rule.match_mode, rule.case_sensitive as i64, rule.language, rule.app_scope, rule.enabled as i64,
                rule.use_as_hint as i64, rule.source, rule.created_at, now(), rule.apply_count as i64, rule.last_applied_at
            ],
        )?;
        Ok(())
    }

    pub fn delete_rule(&self, id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM dictionary WHERE id = ?1", params![id])?;
        conn.execute("DELETE FROM dictionary_exceptions WHERE rule_id = ?1", params![id])?;
        Ok(())
    }

    pub fn bump_rule_usage(&self, ids: &[String]) -> anyhow::Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let conn = self.conn.lock();
        let ts = now();
        for id in ids {
            conn.execute("UPDATE dictionary SET apply_count = apply_count + 1, last_applied_at = ?1 WHERE id = ?2", params![ts, id])?;
        }
        Ok(())
    }

    pub fn add_rule_exception(&self, rule_id: &str, context: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute("INSERT INTO dictionary_exceptions (id, rule_id, context, created_at) VALUES (?1,?2,?3,?4)", params![new_id(), rule_id, context, now()])?;
        Ok(())
    }

    pub fn list_rule_exceptions(&self) -> anyhow::Result<Vec<(String, String)>> {
        let conn = self.conn.lock();
        let mut st = conn.prepare("SELECT rule_id, context FROM dictionary_exceptions")?;
        let rows = st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    // ----- snippets -----

    fn row_to_snippet(r: &rusqlite::Row) -> rusqlite::Result<Snippet> {
        Ok(Snippet {
            id: r.get("id")?,
            trigger: r.get("trigger")?,
            expansion: r.get("expansion")?,
            enabled: r.get::<_, i64>("enabled")? != 0,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
            apply_count: r.get::<_, i64>("apply_count")? as u64,
            last_applied_at: r.get("last_applied_at")?,
        })
    }

    pub fn list_snippets(&self) -> anyhow::Result<Vec<Snippet>> {
        let conn = self.conn.lock();
        let mut st = conn.prepare("SELECT * FROM snippets ORDER BY updated_at DESC")?;
        let rows = st.query_map([], Self::row_to_snippet)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn upsert_snippet(&self, s: &Snippet) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO snippets (id, trigger, expansion, enabled, created_at, updated_at, apply_count, last_applied_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(id) DO UPDATE SET trigger=excluded.trigger, expansion=excluded.expansion, enabled=excluded.enabled, updated_at=excluded.updated_at",
            params![s.id, s.trigger, s.expansion, s.enabled as i64, s.created_at, now(), s.apply_count as i64, s.last_applied_at],
        )?;
        Ok(())
    }

    pub fn delete_snippet(&self, id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM snippets WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn bump_snippet_usage(&self, ids: &[String]) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        let ts = now();
        for id in ids {
            conn.execute("UPDATE snippets SET apply_count = apply_count + 1, last_applied_at = ?1 WHERE id = ?2", params![ts, id])?;
        }
        Ok(())
    }

    // ----- learning -----

    pub fn add_learning_event(&self, history_id: Option<&str>, kind: &str, before: &str, after: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO learning_events (id, history_id, kind, before_text, after_text, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![new_id(), history_id, kind, before, after, now()],
        )?;
        Ok(())
    }

    pub fn count_learning_pairs(&self, kind: &str, before: &str, after: &str) -> anyhow::Result<u64> {
        let conn = self.conn.lock();
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM learning_events WHERE kind = ?1 AND before_text = ?2 AND after_text = ?3",
            params![kind, before, after],
            |r| r.get(0),
        )?;
        Ok(n as u64)
    }

    pub fn delete_learning_data(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM learning_events", [])?;
        conn.execute("DELETE FROM suggestions", [])?;
        Ok(())
    }

    fn row_to_suggestion(r: &rusqlite::Row) -> rusqlite::Result<Suggestion> {
        Ok(Suggestion {
            id: r.get("id")?,
            kind: r.get("kind")?,
            wrong: r.get("wrong")?,
            correct: r.get("correct")?,
            evidence_count: r.get::<_, i64>("evidence_count")? as u64,
            reason: r.get("reason")?,
            status: r.get("status")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
        })
    }

    pub fn list_suggestions(&self) -> anyhow::Result<Vec<Suggestion>> {
        let conn = self.conn.lock();
        let mut st = conn.prepare("SELECT * FROM suggestions ORDER BY updated_at DESC")?;
        let rows = st.query_map([], Self::row_to_suggestion)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Insert or bump a suggestion. Returns the current evidence count.
    pub fn upsert_suggestion(&self, kind: &str, wrong: &str, correct: &str, reason: &str) -> anyhow::Result<u64> {
        let conn = self.conn.lock();
        let existing: Option<(String, i64, String)> = conn
            .query_row(
                "SELECT id, evidence_count, status FROM suggestions WHERE kind = ?1 AND wrong = ?2 AND correct = ?3",
                params![kind, wrong, correct],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        match existing {
            Some((id, count, status)) => {
                if status == "ignored" {
                    return Ok(0);
                }
                conn.execute("UPDATE suggestions SET evidence_count = ?1, reason = ?2, updated_at = ?3 WHERE id = ?4", params![count + 1, reason, now(), id])?;
                Ok((count + 1) as u64)
            }
            None => {
                conn.execute(
                    "INSERT INTO suggestions (id, kind, wrong, correct, evidence_count, reason, status, created_at, updated_at) VALUES (?1,?2,?3,?4,1,?5,'pending',?6,?6)",
                    params![new_id(), kind, wrong, correct, reason, now()],
                )?;
                Ok(1)
            }
        }
    }

    pub fn set_suggestion_status(&self, id: &str, status: &str) -> anyhow::Result<Option<Suggestion>> {
        let conn = self.conn.lock();
        conn.execute("UPDATE suggestions SET status = ?1, updated_at = ?2 WHERE id = ?3", params![status, now(), id])?;
        Ok(conn.query_row("SELECT * FROM suggestions WHERE id = ?1", params![id], Self::row_to_suggestion).optional()?)
    }

    // ----- app styles -----

    fn row_to_style(r: &rusqlite::Row) -> rusqlite::Result<AppStyle> {
        Ok(AppStyle {
            id: r.get("id")?,
            name: r.get("name")?,
            process_match: r.get("process_match")?,
            category: r.get("category")?,
            trailing_punctuation: r.get::<_, i64>("trailing_punctuation")? != 0,
            capitalize_first: r.get::<_, i64>("capitalize_first")? != 0,
            tone: r.get("tone")?,
            enabled: r.get::<_, i64>("enabled")? != 0,
        })
    }

    pub fn list_app_styles(&self) -> anyhow::Result<Vec<AppStyle>> {
        let conn = self.conn.lock();
        let mut st = conn.prepare("SELECT * FROM app_styles ORDER BY name")?;
        let rows = st.query_map([], Self::row_to_style)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn upsert_app_style(&self, s: &AppStyle) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO app_styles (id, name, process_match, category, trailing_punctuation, capitalize_first, tone, enabled) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(id) DO UPDATE SET name=excluded.name, process_match=excluded.process_match, category=excluded.category, trailing_punctuation=excluded.trailing_punctuation,
             capitalize_first=excluded.capitalize_first, tone=excluded.tone, enabled=excluded.enabled",
            params![s.id, s.name, s.process_match, s.category, s.trailing_punctuation as i64, s.capitalize_first as i64, s.tone, s.enabled as i64],
        )?;
        Ok(())
    }

    pub fn delete_app_style(&self, id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM app_styles WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ----- stats -----

    pub fn bump_daily(&self, words: u64, audio_ms: u64, corrections: u64) -> anyhow::Result<()> {
        let day = chrono::Local::now().format("%Y-%m-%d").to_string();
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO stats_daily (day, dictations, words, audio_ms, corrections) VALUES (?1, 1, ?2, ?3, ?4)
             ON CONFLICT(day) DO UPDATE SET dictations = dictations + 1, words = words + ?2, audio_ms = audio_ms + ?3, corrections = corrections + ?4",
            params![day, words as i64, audio_ms as i64, corrections as i64],
        )?;
        Ok(())
    }

    pub fn bump_daily_counter(&self, column: &str) -> anyhow::Result<()> {
        let col = match column {
            "retries" => "retries",
            "edits" => "edits",
            _ => anyhow::bail!("bad column"),
        };
        let day = chrono::Local::now().format("%Y-%m-%d").to_string();
        let conn = self.conn.lock();
        conn.execute(
            &format!("INSERT INTO stats_daily (day, {col}) VALUES (?1, 1) ON CONFLICT(day) DO UPDATE SET {col} = {col} + 1"),
            params![day],
        )?;
        Ok(())
    }

    pub fn stats_summary(&self, typing_wpm: f64) -> anyhow::Result<StatsSummary> {
        let conn = self.conn.lock();
        let mut st = conn.prepare("SELECT * FROM stats_daily ORDER BY day ASC")?;
        let daily: Vec<DailyStat> = st
            .query_map([], |r| {
                Ok(DailyStat {
                    day: r.get("day")?,
                    dictations: r.get::<_, i64>("dictations")? as u64,
                    words: r.get::<_, i64>("words")? as u64,
                    audio_ms: r.get::<_, i64>("audio_ms")? as u64,
                    corrections: r.get::<_, i64>("corrections")? as u64,
                    retries: r.get::<_, i64>("retries")? as u64,
                    edits: r.get::<_, i64>("edits")? as u64,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        let total_words: u64 = daily.iter().map(|d| d.words).sum();
        let total_ms: u64 = daily.iter().map(|d| d.audio_ms).sum();
        let dictations: u64 = daily.iter().map(|d| d.dictations).sum();
        let corrections: u64 = daily.iter().map(|d| d.corrections).sum();
        let retries: u64 = daily.iter().map(|d| d.retries).sum();
        let edits: u64 = daily.iter().map(|d| d.edits).sum();
        let total_minutes = total_ms as f64 / 60_000.0;
        let avg_wpm = if total_minutes > 0.0 { total_words as f64 / total_minutes } else { 0.0 };
        // Time saved = time it would take to type the words at `typing_wpm` minus time spent speaking.
        let typing_minutes = if typing_wpm > 0.0 { total_words as f64 / typing_wpm } else { 0.0 };
        let time_saved_minutes = (typing_minutes - total_minutes).max(0.0);

        // streaks over days with at least one dictation
        let today = chrono::Local::now().date_naive();
        let mut days: Vec<chrono::NaiveDate> = daily
            .iter()
            .filter(|d| d.dictations > 0)
            .filter_map(|d| chrono::NaiveDate::parse_from_str(&d.day, "%Y-%m-%d").ok())
            .collect();
        days.sort();
        let mut longest = 0u64;
        let mut run = 0u64;
        let mut prev: Option<chrono::NaiveDate> = None;
        for d in &days {
            run = match prev {
                Some(p) if (*d - p).num_days() == 1 => run + 1,
                _ => 1,
            };
            longest = longest.max(run);
            prev = Some(*d);
        }
        let mut current = 0u64;
        if let Some(last) = days.last() {
            if (today - *last).num_days() <= 1 {
                current = run;
            }
        }

        let mut lst = conn.prepare("SELECT latency_ms FROM history WHERE status = 'success' AND latency_ms > 0 ORDER BY latency_ms ASC")?;
        let lat: Vec<u64> = lst.query_map([], |r| r.get::<_, i64>(0))?.filter_map(|r| r.ok()).map(|v| v as u64).collect();
        let pct = |p: f64| -> u64 {
            if lat.is_empty() {
                0
            } else {
                let idx = ((lat.len() as f64 - 1.0) * p).round() as usize;
                lat[idx.min(lat.len() - 1)]
            }
        };
        let mut cst = conn.prepare("SELECT COALESCE(app_category,'unknown'), COUNT(*) FROM history WHERE status='success' GROUP BY 1 ORDER BY 2 DESC")?;
        let by_category: Vec<(String, u64)> = cst.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u64)))?.filter_map(|r| r.ok()).collect();

        Ok(StatsSummary {
            total_words,
            total_minutes,
            avg_wpm,
            dictations,
            corrections_applied: corrections,
            retries,
            edits,
            current_streak: current,
            longest_streak: longest,
            time_saved_minutes,
            time_saved_formula: format!("(λέξεις / {typing_wpm} λέξεις ανά λεπτό πληκτρολόγησης) - λεπτά ομιλίας"),
            p50_latency_ms: pct(0.5),
            p95_latency_ms: pct(0.95),
            by_category,
            daily,
        })
    }

    /// Everything the user owns, as one JSON document (export).
    pub fn export_all(&self) -> anyhow::Result<serde_json::Value> {
        let history = self.list_history(None, 1_000_000, 0)?;
        let rules = self.list_rules()?;
        let snippets = self.list_snippets()?;
        let suggestions = self.list_suggestions()?;
        let styles = self.list_app_styles()?;
        let stats = self.stats_summary(40.0)?;
        Ok(serde_json::json!({
            "exported_at": now(),
            "history": history,
            "dictionary": rules,
            "snippets": snippets,
            "suggestions": suggestions,
            "app_styles": styles,
            "stats_daily": stats.daily,
        }))
    }

    pub fn wipe_everything(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        conn.execute_batch(
            "DELETE FROM history; DELETE FROM dictionary; DELETE FROM dictionary_exceptions; DELETE FROM snippets; DELETE FROM learning_events; DELETE FROM suggestions; DELETE FROM app_styles; DELETE FROM stats_daily;",
        )?;
        Ok(())
    }
}

pub fn ts_now() -> String {
    now()
}

pub fn parse_ts(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s).ok().map(|d| d.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(text: &str) -> HistoryEntry {
        HistoryEntry {
            id: new_id(),
            created_at: now(),
            raw_text: text.into(),
            cleaned_text: text.into(),
            final_text: text.into(),
            language: "el".into(),
            detected_language: Some("el".into()),
            app_name: Some("Notepad".into()),
            app_process: Some("notepad.exe".into()),
            app_category: Some("editor".into()),
            cleanup_mode: "normal".into(),
            rules_applied: vec![],
            audio_ms: 3000,
            latency_ms: 900,
            inference_ms: 500,
            word_count: 3,
            engine: Some("whisper_local".into()),
            model: Some("large-v3-q5_0".into()),
            insertion_method: Some("paste".into()),
            status: "success".into(),
            audio_path: None,
            context_used: false,
            retried: false,
            undone: false,
            edited_text: None,
        }
    }

    #[test]
    fn history_and_stats_round_trip() {
        let db = Db::open_in_memory().unwrap();
        db.insert_history(&entry("γεια σου κόσμε")).unwrap();
        db.bump_daily(3, 3000, 0).unwrap();
        let list = db.list_history(Some("κόσμε"), 10, 0).unwrap();
        assert_eq!(list.len(), 1);
        let s = db.stats_summary(40.0).unwrap();
        assert_eq!(s.total_words, 3);
        assert_eq!(s.dictations, 1);
        assert_eq!(s.current_streak, 1);
        assert_eq!(s.p50_latency_ms, 900);
        let last = db.last_successful_history().unwrap().unwrap();
        assert_eq!(last.final_text, "γεια σου κόσμε");
    }

    #[test]
    fn dictionary_crud_and_suggestions() {
        let db = Db::open_in_memory().unwrap();
        let rule = DictionaryRule {
            id: new_id(),
            wrong: "Λούραμ".into(),
            correct: "Luram".into(),
            match_mode: "whole_word".into(),
            case_sensitive: false,
            language: None,
            app_scope: None,
            enabled: true,
            use_as_hint: true,
            source: "user".into(),
            created_at: now(),
            updated_at: now(),
            apply_count: 0,
            last_applied_at: None,
        };
        db.upsert_rule(&rule).unwrap();
        db.bump_rule_usage(&[rule.id.clone()]).unwrap();
        let rules = db.list_rules().unwrap();
        assert_eq!(rules[0].apply_count, 1);
        assert_eq!(db.upsert_suggestion("dictionary", "Λούραμ", "Luram", "seen twice").unwrap(), 1);
        assert_eq!(db.upsert_suggestion("dictionary", "Λούραμ", "Luram", "seen twice").unwrap(), 2);
        db.delete_rule(&rule.id).unwrap();
        assert!(db.list_rules().unwrap().is_empty());
    }
}
