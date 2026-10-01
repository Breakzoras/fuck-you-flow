//! Sharing Dictionary corrections with the maker.
//!
//! On from the start, with a switch in Settings under Privacy (the maker's
//! decision, 1 October 2026; the first design put a question and waited for a
//! yes). The corrections in the Dictionary (the wrong word, the right word,
//! its language and how it matches, names included) are sent to
//! fuckyouflow.app, and new ones follow about once a day. Dictated text,
//! History and audio are not part of it and there is no code path here that
//! reads them. The built-in corrections are not sent back. On the server the
//! batches sit in a closed folder under a random id this install made up; a
//! person reads them, and the corrections that are useful to everyone go into
//! the built-in list of a later version (`starter.rs`).
//!
//! With the switch off nothing is built and nothing is sent: `next_batch` is
//! the only door and it returns `None`. The same holds for a run in which the
//! user's privacy choices could not be read (`privacy_choices_unknown`).

use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;

use crate::db::DictionaryRule;
use crate::pipeline::Shared;
use crate::settings::PrivacySettings;

/// A side longer than this is a sentence, and sentences stay home.
const MAX_TERM_CHARS: usize = 80;
/// The server takes 300 rules in one request.
const BATCH: usize = 200;
/// One run sends at most this many batches; the rest goes next time.
const BATCHES_PER_RUN: usize = 5;

/// One sender at a time, and "delete what I sent" waits behind it.
static RUN: once_cell::sync::Lazy<tokio::sync::Mutex<()>> = once_cell::sync::Lazy::new(|| tokio::sync::Mutex::new(()));

pub async fn run_lock() -> tokio::sync::MutexGuard<'static, ()> {
    RUN.lock().await
}

/// Where the corrections go. FYF_SHARE_URL points a test build at a local receiver.
fn share_url() -> String {
    std::env::var("FYF_SHARE_URL").unwrap_or_else(|_| "https://fuckyouflow.app/api/dictionary".into())
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SharedRule {
    pub wrong: String,
    pub correct: String,
    pub language: Option<String>,
    pub match_mode: String,
}

fn fingerprint(rule: &SharedRule) -> String {
    let joined = format!("{}\u{1}{}\u{1}{}\u{1}{}", rule.wrong, rule.correct, rule.language.as_deref().unwrap_or(""), rule.match_mode);
    format!("{:016x}", crate::starter::fnv1a(&joined))
}

/// Every rule that may leave, as (rule id, fingerprint, what is sent): the
/// user's own rules and the learned ones they accepted, switched on, with both
/// sides short enough to be a correction.
pub fn shareable(rules: &[DictionaryRule]) -> Vec<(String, String, SharedRule)> {
    rules
        .iter()
        .filter(|r| r.enabled && r.source != "starter")
        .filter_map(|r| {
            let (wrong, correct) = (r.wrong.trim(), r.correct.trim());
            if wrong.is_empty() || correct.is_empty() || wrong.chars().count() > MAX_TERM_CHARS || correct.chars().count() > MAX_TERM_CHARS {
                return None;
            }
            let shared = SharedRule { wrong: wrong.to_string(), correct: correct.to_string(), language: r.language.clone(), match_mode: r.match_mode.clone() };
            Some((r.id.clone(), fingerprint(&shared), shared))
        })
        .collect()
}

fn valid_install_id(id: &str) -> bool {
    id.len() == 32 && id.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

pub fn new_install_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

/// The next request body and the rows to mark as sent once the server has
/// taken it. `None` when sharing is switched off or nothing new is waiting.
pub fn next_batch(privacy: &PrivacySettings, primary: &str, rules: &[DictionaryRule], sent: &HashMap<String, String>) -> Option<(serde_json::Value, Vec<(String, String)>)> {
    if !privacy.share_dictionary || !valid_install_id(&privacy.share_install_id) {
        return None;
    }
    let waiting: Vec<(String, String, SharedRule)> =
        shareable(rules).into_iter().filter(|(id, print, _)| sent.get(id) != Some(print)).take(BATCH).collect();
    if waiting.is_empty() {
        return None;
    }
    let body = serde_json::json!({
        "install_id": privacy.share_install_id,
        "app_version": env!("CARGO_PKG_VERSION"),
        "primary": primary,
        "rules": waiting.iter().map(|(_, _, r)| r).collect::<Vec<_>>(),
    });
    Some((body, waiting.into_iter().map(|(id, print, _)| (id, print)).collect()))
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(concat!("FuckYouFlow/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(25))
        .build()
        .map_err(|e| e.to_string())
}

/// One request to the server. "offline" when it was not reached, "too_many"
/// or "refused" when it answered with a no.
async fn post(path: &str, body: &serde_json::Value) -> Result<(), String> {
    let resp = client()?.post(format!("{}{path}", share_url())).json(body).send().await.map_err(|e| {
        tracing::info!("sharing: the server was not reached: {e}");
        "offline".to_string()
    })?;
    let status = resp.status();
    // The receiver says `"ok": true`. A 200 from anything else on the way (a
    // captive portal, a filtering proxy) is not the server having taken it.
    let reply: serde_json::Value = resp.json().await.unwrap_or_default();
    if status.is_success() && reply["ok"] == true {
        return Ok(());
    }
    tracing::warn!("sharing: the server said {status}");
    Err(if status.as_u16() == 429 { "too_many".into() } else { "refused".into() })
}

/// Sends what is waiting. Returns how many corrections the server took.
pub async fn sync(shared: &Arc<Shared>) -> Result<usize, String> {
    let _run = run_lock().await;
    let mut total = 0;
    for _ in 0..BATCHES_PER_RUN {
        let (privacy, primary) = {
            let s = shared.settings.read();
            (s.privacy.clone(), s.language.own_code().to_string())
        };
        let rules = shared.db.list_rules().map_err(|e| e.to_string())?;
        let sent = shared.db.shared_fingerprints().map_err(|e| e.to_string())?;
        let Some((body, rows)) = next_batch(&privacy, &primary, &rules, &sent) else {
            break;
        };
        post("", &body).await?;
        shared.db.mark_shared(&rows).map_err(|e| e.to_string())?;
        total += rows.len();
    }
    if total > 0 {
        tracing::info!("sharing: {total} corrections sent");
        crate::journal::info("sharing.sent", serde_json::json!({ "corrections": total }));
    }
    Ok(total)
}

/// Asks the server to delete everything this install sent.
pub async fn forget(install_id: &str) -> Result<(), String> {
    if !valid_install_id(install_id) {
        return Ok(());
    }
    post("/forget", &serde_json::json!({ "install_id": install_id })).await
}

/// A minute and a half after start, then once a day, for as long as the app runs.
pub fn spawn(shared: Arc<Shared>) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(90)).await;
        loop {
            let _ = sync(&shared).await;
            tokio::time::sleep(std::time::Duration::from_secs(24 * 3600)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(id: &str, wrong: &str, correct: &str, source: &str, enabled: bool) -> DictionaryRule {
        DictionaryRule {
            id: id.into(),
            wrong: wrong.into(),
            correct: correct.into(),
            match_mode: "whole_word".into(),
            case_sensitive: false,
            language: Some("en".into()),
            app_scope: None,
            enabled,
            use_as_hint: false,
            source: source.into(),
            created_at: String::new(),
            updated_at: String::new(),
            apply_count: 0,
            last_applied_at: None,
        }
    }

    fn agreed() -> PrivacySettings {
        PrivacySettings { share_dictionary: true, share_dictionary_asked: true, share_install_id: new_install_id(), ..Default::default() }
    }

    fn sample() -> Vec<DictionaryRule> {
        vec![
            rule("a", "kubernets", "Kubernetes", "user", true),
            rule("b", "docer", "Docker", "suggested", true),
            rule("c", "γκίτχαμπ", "GitHub", "starter", true),
            rule("d", "teh", "the", "user", false),
            rule("e", "x", &"a whole sentence somebody saved as a correction ".repeat(3), "user", true),
        ]
    }

    #[test]
    fn nothing_is_built_with_the_switch_off_or_without_an_id() {
        let none = HashMap::new();
        // switched off by the user
        let off = PrivacySettings { share_dictionary: false, share_dictionary_asked: true, share_install_id: new_install_id(), ..Default::default() };
        assert!(next_batch(&off, "el", &sample(), &none).is_none());
        // on, before startup has given this install an id of its own
        assert!(next_batch(&PrivacySettings::default(), "el", &sample(), &none).is_none());
    }

    #[test]
    fn sharing_is_on_from_the_start_and_a_switch_set_by_hand_stays() {
        let none = HashMap::new();
        // a new install, and an older one whose file never said anything
        let mut fresh = PrivacySettings::default();
        assert!(fresh.settle_sharing(), "startup has something to save");
        assert!(fresh.share_dictionary && valid_install_id(&fresh.share_install_id));
        assert!(next_batch(&fresh, "el", &sample(), &none).is_some());
        assert!(!fresh.settle_sharing(), "the second start changes nothing");

        // the file of the first design: off, because nobody had answered yet
        let mut unanswered = PrivacySettings { share_dictionary: false, share_dictionary_asked: false, ..Default::default() };
        unanswered.settle_sharing();
        assert!(unanswered.share_dictionary);

        // switched off by the user: left alone, and no id is made for it
        let mut off = PrivacySettings { share_dictionary: false, share_dictionary_asked: true, ..Default::default() };
        assert!(!off.settle_sharing());
        assert!(!off.share_dictionary && off.share_install_id.is_empty());
        assert!(next_batch(&off, "el", &sample(), &none).is_none());
    }

    #[test]
    fn only_the_users_own_live_corrections_leave() {
        let privacy = agreed();
        let (body, rows) = next_batch(&privacy, "el", &sample(), &HashMap::new()).expect("a batch");
        let sent: Vec<&str> = body["rules"].as_array().unwrap().iter().map(|r| r["wrong"].as_str().unwrap()).collect();
        assert_eq!(sent, vec!["kubernets", "docer"], "no built-in rule, no switched off rule, no sentence");
        assert_eq!(body["install_id"], privacy.share_install_id.as_str());
        assert_eq!(body["primary"], "el");
        let keys: Vec<&String> = body.as_object().unwrap().keys().collect();
        assert_eq!(keys.len(), 4, "install id, version, language and the rules, nothing else: {keys:?}");
        let rule_keys: Vec<&String> = body["rules"][0].as_object().unwrap().keys().collect();
        assert_eq!(rule_keys.len(), 4, "{rule_keys:?}");
        assert_eq!(rows.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(), vec!["a", "b"]);
    }

    #[test]
    fn a_sent_correction_goes_again_only_when_it_changed() {
        let privacy = agreed();
        let mut rules = sample();
        let (_, rows) = next_batch(&privacy, "el", &rules, &HashMap::new()).unwrap();
        let sent: HashMap<String, String> = rows.into_iter().collect();
        assert!(next_batch(&privacy, "el", &rules, &sent).is_none(), "nothing new, nothing sent");
        rules[0].correct = "k8s".into();
        let (body, _) = next_batch(&privacy, "el", &rules, &sent).unwrap();
        assert_eq!(body["rules"].as_array().unwrap().len(), 1);
        assert_eq!(body["rules"][0]["correct"], "k8s");
    }

    /// Against a receiver started by hand, the way a real install talks to it:
    ///   DICT_DIR=<folder> PORT=8799 python server/report-receiver/receiver.py
    ///   FYF_SHARE_URL=http://127.0.0.1:8799/api/dictionary cargo test --lib sharing::tests::live -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn live_round_trip_with_the_receiver() {
        let privacy = agreed();
        let (body, rows) = next_batch(&privacy, "el", &sample(), &HashMap::new()).expect("a batch");
        post("", &body).await.expect("the receiver takes the batch");
        println!("sent {} corrections as {}", rows.len(), privacy.share_install_id);
        if std::env::var("FYF_LIVE_KEEP").is_err() {
            forget(&privacy.share_install_id).await.expect("the receiver forgets them");
        }
    }

    #[test]
    fn a_large_dictionary_goes_in_batches_the_server_accepts() {
        let rules: Vec<DictionaryRule> = (0..450).map(|i| rule(&format!("r{i}"), &format!("wrong{i}"), "right", "user", true)).collect();
        let (body, rows) = next_batch(&agreed(), "el", &rules, &HashMap::new()).unwrap();
        assert_eq!(body["rules"].as_array().unwrap().len(), BATCH);
        assert_eq!(rows.len(), BATCH);
    }
}
