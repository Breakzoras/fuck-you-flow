//! The built-in corrections every install starts with.
//!
//! Until 0.9.14 a fresh install had an empty Dictionary, so everything the
//! Dictionary had learned on the machine the app was built on stayed there.
//! The list in `starter/dictionary.json` is compiled into the program, which
//! is how it reaches an install that updates as well as a fresh one.
//!
//! A built-in rule is an ordinary Dictionary row with `source = "starter"`.
//! It is added once: its id goes into `starter_seen` whether it was added or
//! skipped, and an id in there is never looked at again. So a rule the user
//! deleted or switched off stays that way after every update, and a rule of
//! their own for the same word is never joined by ours.

use serde::Deserialize;

use crate::db::{Db, DictionaryRule};

const STARTER_JSON: &str = include_str!("../starter/dictionary.json");

#[derive(Debug, Deserialize)]
struct StarterFile {
    rules: Vec<StarterRule>,
}

#[derive(Debug, Deserialize)]
struct StarterRule {
    wrong: String,
    correct: String,
    /// "all", or the code of the one language whose speakers it is for.
    #[serde(rename = "for")]
    for_language: String,
}

/// FNV-1a, written out because the id must come out the same in every build:
/// the standard hasher makes no such promise.
pub(crate) fn fnv1a(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn rule_id(wrong: &str) -> String {
    format!("starter-{:016x}", fnv1a(&wrong.trim().to_lowercase()))
}

/// Adds the built-in corrections this install has not been offered yet, for
/// everyone and for speakers of `primary` (`LanguageModeSetting::own_code`, so
/// it may be empty). Returns how many rows were added.
pub fn apply(db: &Db, primary: &str) -> anyhow::Result<usize> {
    apply_from(db, primary, STARTER_JSON)
}

fn apply_from(db: &Db, primary: &str, json: &str) -> anyhow::Result<usize> {
    let file: StarterFile = serde_json::from_str(json)?;
    let seen = db.starter_seen()?;
    let mut taken: std::collections::HashSet<String> = db.list_rules()?.into_iter().map(|r| r.wrong.trim().to_lowercase()).collect();
    let mut offered = Vec::new();
    let mut added = 0;
    for rule in file.rules.iter().filter(|r| r.for_language == "all" || r.for_language == primary) {
        let wrong = rule.wrong.trim();
        let correct = rule.correct.trim();
        if wrong.is_empty() || correct.is_empty() {
            continue;
        }
        let id = rule_id(wrong);
        if seen.contains(&id) {
            continue;
        }
        offered.push(id.clone());
        // The user's own rule for this word wins, switched on or off.
        if !taken.insert(wrong.to_lowercase()) {
            continue;
        }
        db.upsert_rule(&DictionaryRule {
            id,
            wrong: wrong.to_string(),
            correct: correct.to_string(),
            match_mode: "whole_word".into(),
            case_sensitive: false,
            language: crate::cleanup::dictionary::script_language(wrong),
            app_scope: None,
            enabled: true,
            // The recognition prompt has room for about fifteen names and they
            // belong to the user. A built-in term never takes one of them.
            use_as_hint: false,
            source: "starter".into(),
            created_at: crate::db::ts_now(),
            updated_at: crate::db::ts_now(),
            apply_count: 0,
            last_applied_at: None,
        })?;
        added += 1;
    }
    db.mark_starter_seen(&offered)?;
    Ok(added)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cleanup::dictionary::DictionaryEngine;

    // Invented words of the same shapes as the real list.
    const SAMPLE: &str = r#"{"version": 1, "rules": [
        {"wrong": "kubernets", "correct": "Kubernetes", "for": "all"},
        {"wrong": "γκίτχαμπ", "correct": "GitHub", "for": "el"},
        {"wrong": "ιστωσελίδα", "correct": "ιστοσελίδα", "for": "el"}
    ]}"#;

    fn rules(db: &Db) -> Vec<DictionaryRule> {
        let mut all = db.list_rules().unwrap();
        all.sort_by(|a, b| a.wrong.cmp(&b.wrong));
        all
    }

    #[test]
    fn the_shipped_list_reads_and_holds_no_duplicate() {
        let file: StarterFile = serde_json::from_str(STARTER_JSON).expect("starter/dictionary.json");
        let mut ids = std::collections::HashSet::new();
        for r in &file.rules {
            assert!(!r.wrong.trim().is_empty() && !r.correct.trim().is_empty(), "an empty side: {r:?}");
            assert!(!r.wrong.trim().contains(char::is_whitespace), "one word on the wrong side: {r:?}");
            assert!(r.for_language == "all" || crate::languages::is_choice(&r.for_language), "unknown language: {r:?}");
            assert!(ids.insert(rule_id(&r.wrong)), "twice in the list: {r:?}");
        }
    }

    #[test]
    fn a_fresh_install_gets_the_list_once() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(apply_from(&db, "el", SAMPLE).unwrap(), 3);
        let got = rules(&db);
        assert_eq!(got.len(), 3);
        assert!(got.iter().all(|r| r.source == "starter" && r.enabled && !r.use_as_hint && r.match_mode == "whole_word"));
        assert_eq!(got.iter().find(|r| r.wrong == "kubernets").unwrap().language.as_deref(), Some("en"));
        assert_eq!(got.iter().find(|r| r.wrong == "γκίτχαμπ").unwrap().language.as_deref(), Some("el"));
        // the second start adds nothing
        assert_eq!(apply_from(&db, "el", SAMPLE).unwrap(), 0);
        assert_eq!(rules(&db).len(), 3);
    }

    #[test]
    fn another_language_gets_only_what_is_for_everyone() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(apply_from(&db, "de", SAMPLE).unwrap(), 1);
        assert_eq!(rules(&db)[0].wrong, "kubernets");
        // and the Greek ones arrive the day the user says they speak Greek
        assert_eq!(apply_from(&db, "el", SAMPLE).unwrap(), 2);
        // a user the app knows no language for gets what is for everyone
        let unknown = Db::open_in_memory().unwrap();
        assert_eq!(apply_from(&unknown, "", SAMPLE).unwrap(), 1);
    }

    #[test]
    fn a_deleted_or_switched_off_rule_does_not_come_back() {
        let db = Db::open_in_memory().unwrap();
        apply_from(&db, "el", SAMPLE).unwrap();
        let all = rules(&db);
        db.delete_rule(&all.iter().find(|r| r.wrong == "kubernets").unwrap().id).unwrap();
        let mut off = all.iter().find(|r| r.wrong == "γκίτχαμπ").unwrap().clone();
        off.enabled = false;
        db.upsert_rule(&off).unwrap();
        // a later version ships the same list with one rule more
        let later = SAMPLE.replace("]}", r#", {"wrong": "docer", "correct": "Docker", "for": "all"}]}"#);
        assert_eq!(apply_from(&db, "el", &later).unwrap(), 1);
        let after = rules(&db);
        assert!(after.iter().all(|r| r.wrong != "kubernets"), "the deleted rule stayed deleted");
        assert!(!after.iter().find(|r| r.wrong == "γκίτχαμπ").unwrap().enabled, "the switched off rule stayed off");
        assert!(after.iter().any(|r| r.wrong == "docer"));
    }

    #[test]
    fn the_users_own_rule_for_the_same_word_is_left_alone() {
        let db = Db::open_in_memory().unwrap();
        let own = DictionaryRule {
            id: "mine".into(),
            wrong: "Kubernets".into(),
            correct: "k8s".into(),
            match_mode: "whole_word".into(),
            case_sensitive: false,
            language: None,
            app_scope: None,
            enabled: false,
            use_as_hint: true,
            source: "user".into(),
            created_at: crate::db::ts_now(),
            updated_at: crate::db::ts_now(),
            apply_count: 7,
            last_applied_at: None,
        };
        db.upsert_rule(&own).unwrap();
        assert_eq!(apply_from(&db, "el", SAMPLE).unwrap(), 2);
        let mine: Vec<DictionaryRule> = rules(&db).into_iter().filter(|r| r.wrong.to_lowercase() == "kubernets").collect();
        assert_eq!(mine.len(), 1);
        assert_eq!((mine[0].id.as_str(), mine[0].correct.as_str(), mine[0].enabled, mine[0].apply_count), ("mine", "k8s", false, 7));
        // deleting their own later does not bring ours in behind their back
        db.delete_rule("mine").unwrap();
        assert_eq!(apply_from(&db, "el", SAMPLE).unwrap(), 0);
    }

    #[test]
    fn built_in_terms_never_reach_the_recognition_prompt() {
        let db = Db::open_in_memory().unwrap();
        apply_from(&db, "el", SAMPLE).unwrap();
        let engine = DictionaryEngine::new(db.list_rules().unwrap(), Vec::new());
        assert_eq!(engine.len(), 3);
        assert!(engine.hint_terms(40).is_empty());
        assert_eq!(engine.apply("το γκίτχαμπ και η ιστωσελίδα", "el").text, "το GitHub και η ιστοσελίδα");
    }
}
