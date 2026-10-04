//! Ready-made dictionaries, downloaded per language.
//!
//! The maker keeps one list of corrections per language in the public GitHub
//! repository (`dictionaries/`): words the engine mishears for most speakers
//! of that language, checked by hand before they go in. A user who says yes
//! (the question on the first screen, or the button on the Dictionary page)
//! gets the list for their own language, and after that the app looks once a
//! day for a newer version and takes it in quietly.
//!
//! Every pack is signed with the key that signs the app's own updates and the
//! signature is checked before a single rule is read: a file changed on the
//! way, or put there by someone else, never reaches the Dictionary.
//!
//! A pack rule is an ordinary Dictionary row with `source = "pack"`. The table
//! `pack_rules` keeps what the app wrote for each one. A newer pack may change
//! or withdraw a rule only while the row still reads exactly as the app wrote
//! it: a rule the user edited, switched off or deleted stays the way they left
//! it, and a rule of their own for the same word is never joined by ours.
//!
//! With the switch off (`PackSettings::enabled`) no pack is downloaded. Only
//! the small list (`index.json`) is read: once per run while the question on
//! the first screen is unanswered, and when the user presses the button on the
//! Dictionary page (PackOffer.tsx).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use base64::Engine as _;
use serde::{Deserialize, Serialize};

use crate::db::{Db, DictionaryRule};
use crate::pipeline::Shared;

/// The pack format this build reads. A pack with a higher number is left
/// alone until the app is updated.
const FORMAT: u64 = 1;
/// A real pack is a few dozen kilobytes. Anything this big is not one.
const MAX_BYTES: usize = 2 * 1024 * 1024;
/// A side longer than this is not a correction.
const MAX_TERM_CHARS: usize = 80;
/// The key that signs the app's updates (`plugins.updater.pubkey` in
/// tauri.conf.json; a test keeps the two the same).
const PUBKEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDQyQUM1NTY3QUNBNTg3QTMKUldTamg2V3NaMVdzUXA2b1FzZHRIZFVCK2psT2kvQStqY1BqaEw0VDl5YmdXYVg3MXBNdVFUbUkK";

/// One check at a time: the daily one, the button and a language change.
static RUN: once_cell::sync::Lazy<tokio::sync::Mutex<()>> = once_cell::sync::Lazy::new(|| tokio::sync::Mutex::new(()));

pub async fn run_lock() -> tokio::sync::MutexGuard<'static, ()> {
    RUN.lock().await
}

/// Where the packs live. FYF_PACKS_URL points a test build at a local folder
/// served over http.
fn base_url() -> String {
    std::env::var("FYF_PACKS_URL").unwrap_or_else(|_| "https://raw.githubusercontent.com/Breakzoras/fuck-you-flow/main/dictionaries".into())
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct IndexEntry {
    pub version: u64,
    pub rules: u64,
}

#[derive(Debug, Deserialize)]
pub struct Index {
    pub format: u64,
    pub packs: HashMap<String, IndexEntry>,
}

#[derive(Debug, Deserialize)]
pub struct Pack {
    pub format: u64,
    pub language: String,
    pub version: u64,
    pub rules: Vec<PackRule>,
}

#[derive(Debug, Deserialize)]
pub struct PackRule {
    pub wrong: String,
    pub correct: String,
}

/// What a check did, for the Dictionary page and the log.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct Applied {
    pub added: usize,
    pub updated: usize,
    pub withdrawn: usize,
}

impl Applied {
    pub fn changed(&self) -> bool {
        self.added + self.updated + self.withdrawn > 0
    }
}

/// FNV-1a, written out because the result must come out the same in every
/// build: the standard hasher makes no such promise.
pub(crate) fn fnv1a(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn rule_id(wrong: &str) -> String {
    format!("pack-{:016x}", fnv1a(&wrong.trim().to_lowercase()))
}

/// Everything about a row the user can change from the Dictionary page. Use
/// counts and times are left out: they move every time the rule fires.
fn print(rule: &DictionaryRule) -> String {
    let joined = format!(
        "{}\u{1}{}\u{1}{}\u{1}{}\u{1}{}\u{1}{}\u{1}{}",
        rule.wrong,
        rule.correct,
        rule.match_mode,
        rule.case_sensitive,
        rule.enabled,
        rule.use_as_hint,
        rule.app_scope.as_deref().unwrap_or("")
    );
    format!("{:016x}", fnv1a(&joined))
}

/// Which pack a user of `own` (`Settings::starter_language`, maybe empty)
/// gets: their language's when there is one, else the one for everyone.
pub fn pick(index: &Index, own: &str) -> Option<String> {
    if !own.is_empty() && index.packs.contains_key(own) {
        return Some(own.to_string());
    }
    index.packs.contains_key("all").then(|| "all".to_string())
}

/// Checks `data` against the minisign signature in `sig` (the base64 text a
/// `tauri signer sign` writes next to a file), with the update key.
pub fn verify(data: &[u8], sig: &str) -> anyhow::Result<()> {
    verify_with(data, sig, PUBKEY)
}

fn verify_with(data: &[u8], sig: &str, pubkey_b64: &str) -> anyhow::Result<()> {
    let b64 = base64::engine::general_purpose::STANDARD;
    let key_text = String::from_utf8(b64.decode(pubkey_b64.trim())?)?;
    let sig_text = String::from_utf8(b64.decode(sig.trim())?)?;
    let key = minisign_verify::PublicKey::decode(&key_text)?;
    let signature = minisign_verify::Signature::decode(&sig_text)?;
    key.verify(data, &signature, true)?;
    Ok(())
}

/// Reads a pack whose signature already checked out, and refuses one that is
/// not what was asked for.
pub fn parse(data: &[u8], language: &str) -> anyhow::Result<Pack> {
    let pack: Pack = serde_json::from_slice(data)?;
    anyhow::ensure!(pack.format <= FORMAT, "pack format {} is newer than this app", pack.format);
    anyhow::ensure!(pack.language == language, "asked for {language}, got {}", pack.language);
    Ok(pack)
}

fn row(wrong: &str, correct: &str, id: &str) -> DictionaryRule {
    let now = crate::db::ts_now();
    DictionaryRule {
        id: id.to_string(),
        wrong: wrong.to_string(),
        correct: correct.to_string(),
        match_mode: "whole_word".into(),
        case_sensitive: false,
        language: crate::cleanup::dictionary::script_language(wrong),
        app_scope: None,
        enabled: true,
        // The recognition prompt has room for about fifteen names and they
        // belong to the user. A pack term never takes one of them.
        use_as_hint: false,
        source: "pack".into(),
        created_at: now.clone(),
        updated_at: now,
        apply_count: 0,
        last_applied_at: None,
    }
}

/// Brings the Dictionary in line with `pack`. Adds what is new, follows a
/// changed correction and removes a withdrawn one, each only while the user
/// has not touched that row. Safe to run again with the same pack.
pub fn apply(db: &Db, pack: &Pack) -> anyhow::Result<Applied> {
    let records = db.pack_records()?;
    let rules = db.list_rules()?;
    let by_id: HashMap<&str, &DictionaryRule> = rules.iter().map(|r| (r.id.as_str(), r)).collect();
    // Words that already have a rule of the user's own, switched on or off.
    let mut taken: HashSet<String> = rules.iter().filter(|r| r.source != "pack").map(|r| r.wrong.trim().to_lowercase()).collect();
    let mut done = Applied::default();
    let mut in_pack = HashSet::new();
    for item in &pack.rules {
        let (wrong, correct) = (item.wrong.trim(), item.correct.trim());
        if wrong.is_empty() || correct.is_empty() || wrong.chars().count() > MAX_TERM_CHARS || correct.chars().count() > MAX_TERM_CHARS {
            continue;
        }
        let id = rule_id(wrong);
        if !in_pack.insert(id.clone()) {
            continue;
        }
        match records.get(&id) {
            Some((written, state)) if state == "installed" => {
                // Still exactly what the app wrote: follow the pack.
                if let Some(current) = by_id.get(id.as_str()).filter(|r| print(r) == *written) {
                    if current.wrong != wrong || current.correct != correct {
                        let mut next = (*current).clone();
                        next.wrong = wrong.to_string();
                        next.correct = correct.to_string();
                        db.upsert_rule(&next)?;
                        db.set_pack_record(&id, &pack.language, &print(&next), "installed")?;
                        done.updated += 1;
                    }
                }
                // Edited, switched off or deleted by the user: theirs now.
            }
            Some(_) => {}
            None => {
                if by_id.contains_key(id.as_str()) || !taken.insert(wrong.to_lowercase()) {
                    // The user's own rule for this word wins, and keeps winning
                    // after they delete it: ours never moves in behind them.
                    db.set_pack_record(&id, &pack.language, "", "skipped")?;
                    continue;
                }
                let new = row(wrong, correct, &id);
                db.upsert_rule(&new)?;
                db.set_pack_record(&id, &pack.language, &print(&new), "installed")?;
                done.added += 1;
            }
        }
    }
    for (id, (written, state)) in &records {
        if in_pack.contains(id) {
            continue;
        }
        if state == "installed" {
            match by_id.get(id.as_str()) {
                Some(r) if print(r) == *written => {
                    db.delete_rule(id)?;
                    done.withdrawn += 1;
                }
                Some(_) => db.set_rule_source(id, "user")?,
                None => {}
            }
        }
        db.delete_pack_record(id)?;
    }
    Ok(done)
}

/// Takes out every pack rule the user has not touched and forgets the rest,
/// so the ones they changed stay as their own.
pub fn remove(db: &Db) -> anyhow::Result<usize> {
    let rules = db.list_rules()?;
    let by_id: HashMap<&str, &DictionaryRule> = rules.iter().map(|r| (r.id.as_str(), r)).collect();
    let mut removed = 0;
    for (id, (written, state)) in db.pack_records()? {
        if state == "installed" {
            match by_id.get(id.as_str()) {
                Some(r) if print(r) == written => {
                    db.delete_rule(&id)?;
                    removed += 1;
                }
                Some(_) => db.set_rule_source(&id, "user")?,
                None => {}
            }
        }
        db.delete_pack_record(&id)?;
    }
    Ok(removed)
}

fn client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent(concat!("FuckYouFlow/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(25))
        .build()?)
}

async fn fetch(client: &reqwest::Client, name: &str) -> anyhow::Result<Vec<u8>> {
    let resp = client.get(format!("{}/{name}", base_url())).send().await?.error_for_status()?;
    if resp.content_length().is_some_and(|n| n as usize > MAX_BYTES) {
        anyhow::bail!("{name} is too big");
    }
    let bytes = resp.bytes().await?;
    anyhow::ensure!(bytes.len() <= MAX_BYTES, "{name} is too big");
    Ok(bytes.to_vec())
}

/// The list of packs on the server. `Ok(None)` when the server answered but
/// has no list this app can read (none published yet, or a newer format);
/// `Err` when it was not reached at all.
pub async fn index() -> Result<Option<Index>, ()> {
    let client = client().map_err(|_| ())?;
    let data = match fetch(&client, "index.json").await {
        Ok(data) => data,
        Err(e) => {
            tracing::info!("packs: the list was not read: {e}");
            let answered = e.downcast_ref::<reqwest::Error>().is_some_and(|r| r.status().is_some());
            return if answered { Ok(None) } else { Err(()) };
        }
    };
    Ok(serde_json::from_slice::<Index>(&data).ok().filter(|i| i.format <= FORMAT))
}

/// What the Dictionary page and the first screen show.
#[derive(Debug, Clone, Serialize)]
pub struct Status {
    /// The pack this user would get ("el", "all"), empty when none is offered
    /// or the list could not be read.
    pub language: String,
    pub rules: u64,
    pub version: u64,
    pub online: bool,
}

pub async fn status(own: &str) -> Status {
    let none = |online| Status { language: String::new(), rules: 0, version: 0, online };
    match index().await {
        Ok(Some(index)) => match pick(&index, own) {
            Some(lang) => {
                let entry = &index.packs[&lang];
                Status { language: lang, rules: entry.rules, version: entry.version, online: true }
            }
            None => none(true),
        },
        Ok(None) => none(true),
        Err(()) => none(false),
    }
}

/// Looks for a newer pack for this user's language and takes it in. Does
/// nothing while the switch is off. `force` fetches even when the version
/// on the server is the one already installed (the button).
pub async fn check(shared: &Arc<Shared>, force: bool) -> anyhow::Result<Applied> {
    let _run = run_lock().await;
    let (packs, own) = {
        let s = shared.settings.read();
        (s.packs.clone(), s.starter_language().to_string())
    };
    if !packs.enabled {
        return Ok(Applied::default());
    }
    let Some(index) = index().await.map_err(|_| anyhow::anyhow!("offline"))? else {
        return Ok(Applied::default());
    };
    let Some(lang) = pick(&index, &own) else {
        return Ok(Applied::default());
    };
    let entry = &index.packs[&lang];
    if !force && lang == packs.language && entry.version <= packs.version {
        return Ok(Applied::default());
    }
    let client = client()?;
    let data = fetch(&client, &format!("{lang}.json")).await?;
    let sig = fetch(&client, &format!("{lang}.json.sig")).await?;
    verify(&data, &String::from_utf8_lossy(&sig)).map_err(|e| {
        tracing::warn!("packs: the {lang} pack failed its signature check and was not used: {e}");
        e
    })?;
    let pack = parse(&data, &lang)?;
    let done = apply(&shared.db, &pack)?;
    {
        let mut current = shared.settings.write();
        let mut next = current.clone();
        next.packs.language = lang.clone();
        next.packs.version = pack.version;
        next.save(&crate::paths::settings_file())?;
        *current = next;
    }
    if done.changed() {
        crate::app::reload_dictionary(shared);
        tracing::info!("packs: {lang} v{} added {}, changed {}, withdrew {}", pack.version, done.added, done.updated, done.withdrawn);
        crate::journal::info("packs.applied", serde_json::json!({ "language": lang, "version": pack.version, "added": done.added, "updated": done.updated, "withdrawn": done.withdrawn }));
    }
    Ok(done)
}

/// Two minutes after start, then once a day, for as long as the app runs.
pub fn spawn(shared: Arc<Shared>) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(120)).await;
        loop {
            if let Err(e) = check(&shared, false).await {
                tracing::info!("packs: no check today: {e}");
            }
            tokio::time::sleep(std::time::Duration::from_secs(24 * 3600)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cleanup::dictionary::DictionaryEngine;

    // Invented words of the same shapes as the real list.
    fn pack(version: u64, rules: &[(&str, &str)]) -> Pack {
        Pack { format: 1, language: "el".into(), version, rules: rules.iter().map(|(w, c)| PackRule { wrong: w.to_string(), correct: c.to_string() }).collect() }
    }

    fn v1() -> Pack {
        pack(1, &[("kubernets", "Kubernetes"), ("γκίτχαμπ", "GitHub"), ("ιστωσελίδα", "ιστοσελίδα")])
    }

    fn rules(db: &Db) -> Vec<DictionaryRule> {
        let mut all = db.list_rules().unwrap();
        all.sort_by(|a, b| a.wrong.cmp(&b.wrong));
        all
    }

    fn find<'a>(all: &'a [DictionaryRule], wrong: &str) -> Option<&'a DictionaryRule> {
        all.iter().find(|r| r.wrong == wrong)
    }

    #[test]
    fn the_key_is_the_one_that_signs_the_updates() {
        let conf: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(conf["plugins"]["updater"]["pubkey"].as_str().unwrap(), PUBKEY);
    }

    #[test]
    fn a_first_pack_goes_in_once_and_never_as_a_name_hint() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(apply(&db, &v1()).unwrap(), Applied { added: 3, updated: 0, withdrawn: 0 });
        let got = rules(&db);
        assert!(got.iter().all(|r| r.source == "pack" && r.enabled && !r.use_as_hint && r.match_mode == "whole_word"));
        assert_eq!(find(&got, "kubernets").unwrap().language.as_deref(), Some("en"));
        assert_eq!(find(&got, "γκίτχαμπ").unwrap().language.as_deref(), Some("el"));
        assert_eq!(apply(&db, &v1()).unwrap(), Applied::default(), "the same pack again changes nothing");
        let engine = DictionaryEngine::new(db.list_rules().unwrap(), Vec::new());
        assert!(engine.hint_terms(40).is_empty());
        assert_eq!(engine.apply("το γκίτχαμπ και η ιστωσελίδα", "el").text, "το GitHub και η ιστοσελίδα");
    }

    #[test]
    fn a_newer_pack_adds_changes_and_withdraws_what_nobody_touched() {
        let db = Db::open_in_memory().unwrap();
        apply(&db, &v1()).unwrap();
        let v2 = pack(2, &[("kubernets", "Kubernetes"), ("γκίτχαμπ", "GitHub Desktop"), ("docer", "Docker")]);
        assert_eq!(apply(&db, &v2).unwrap(), Applied { added: 1, updated: 1, withdrawn: 1 });
        let got = rules(&db);
        assert_eq!(find(&got, "γκίτχαμπ").unwrap().correct, "GitHub Desktop");
        assert!(find(&got, "ιστωσελίδα").is_none(), "withdrawn");
        assert!(find(&got, "docer").is_some());
        assert!(db.pack_records().unwrap().get(&rule_id("ιστωσελίδα")).is_none());
    }

    #[test]
    fn what_the_user_changed_switched_off_or_deleted_stays_their_way() {
        let db = Db::open_in_memory().unwrap();
        apply(&db, &v1()).unwrap();
        let all = rules(&db);
        // edited
        let mut edited = find(&all, "kubernets").unwrap().clone();
        edited.correct = "k8s".into();
        db.upsert_rule(&edited).unwrap();
        // switched off
        let mut off = find(&all, "γκίτχαμπ").unwrap().clone();
        off.enabled = false;
        db.upsert_rule(&off).unwrap();
        // deleted
        db.delete_rule(&find(&all, "ιστωσελίδα").unwrap().id).unwrap();

        // a newer pack changes the first two and keeps the third
        let v2 = pack(2, &[("kubernets", "Kubernetes!"), ("γκίτχαμπ", "GitHub Desktop"), ("ιστωσελίδα", "ιστοσελίδα")]);
        assert_eq!(apply(&db, &v2).unwrap(), Applied::default());
        let got = rules(&db);
        assert_eq!(find(&got, "kubernets").unwrap().correct, "k8s");
        let g = find(&got, "γκίτχαμπ").unwrap();
        assert!(!g.enabled && g.correct == "GitHub");
        assert!(find(&got, "ιστωσελίδα").is_none(), "a deleted rule does not come back");

        // and a pack that withdraws all three leaves the user's two in place,
        // as their own, so a pack that brings the words back never overwrites them
        assert_eq!(apply(&db, &pack(3, &[])).unwrap(), Applied::default());
        assert_eq!(rules(&db).len(), 2);
        assert!(rules(&db).iter().all(|r| r.source == "user"));
        assert!(db.pack_records().unwrap().is_empty());
        assert_eq!(apply(&db, &v1()).unwrap().added, 1);
        assert_eq!(find(&rules(&db), "kubernets").unwrap().correct, "k8s");
    }

    #[test]
    fn the_users_own_rule_for_the_same_word_is_left_alone() {
        let db = Db::open_in_memory().unwrap();
        let mut own = row("Kubernets", "k8s", "mine");
        own.source = "user".into();
        own.enabled = false;
        db.upsert_rule(&own).unwrap();
        assert_eq!(apply(&db, &v1()).unwrap().added, 2);
        let mine: Vec<DictionaryRule> = rules(&db).into_iter().filter(|r| r.wrong.to_lowercase() == "kubernets").collect();
        assert_eq!(mine.len(), 1);
        assert_eq!((mine[0].id.as_str(), mine[0].correct.as_str(), mine[0].enabled), ("mine", "k8s", false));
        // deleting their own later does not bring ours in behind their back
        db.delete_rule("mine").unwrap();
        assert_eq!(apply(&db, &v1()).unwrap(), Applied::default());
    }

    #[test]
    fn removing_the_pack_keeps_what_the_user_made_their_own() {
        let db = Db::open_in_memory().unwrap();
        apply(&db, &v1()).unwrap();
        let mut edited = find(&rules(&db), "kubernets").unwrap().clone();
        edited.correct = "k8s".into();
        db.upsert_rule(&edited).unwrap();
        assert_eq!(remove(&db).unwrap(), 2);
        let left = rules(&db);
        assert_eq!(left.len(), 1);
        assert_eq!((left[0].correct.as_str(), left[0].source.as_str()), ("k8s", "user"), "theirs now");
        assert!(db.pack_records().unwrap().is_empty());
        // and a later yes brings the other two back, never over theirs
        assert_eq!(apply(&db, &v1()).unwrap().added, 2);
        assert_eq!(find(&rules(&db), "kubernets").unwrap().correct, "k8s");
    }

    #[test]
    fn a_sentence_or_an_empty_side_never_becomes_a_rule() {
        let db = Db::open_in_memory().unwrap();
        let long = "λέξη ".repeat(30);
        let p = pack(1, &[("", "x"), ("y", " "), (long.as_str(), "z"), ("ok", "OK")]);
        assert_eq!(apply(&db, &p).unwrap().added, 1);
    }

    #[test]
    fn the_pack_follows_the_language_of_the_user() {
        let index: Index = serde_json::from_str(r#"{"format":1,"packs":{"el":{"version":3,"rules":248},"all":{"version":1,"rules":14}}}"#).unwrap();
        assert_eq!(pick(&index, "el").as_deref(), Some("el"));
        assert_eq!(pick(&index, "de").as_deref(), Some("all"));
        assert_eq!(pick(&index, "").as_deref(), Some("all"));
        let only_el: Index = serde_json::from_str(r#"{"format":1,"packs":{"el":{"version":1,"rules":2}}}"#).unwrap();
        assert_eq!(pick(&only_el, "de"), None);
    }

    #[test]
    fn a_pack_for_another_language_or_a_newer_format_is_refused() {
        let ok = br#"{"format":1,"language":"el","version":1,"rules":[]}"#;
        assert!(parse(ok, "el").is_ok());
        assert!(parse(ok, "de").is_err());
        assert!(parse(br#"{"format":2,"language":"el","version":1,"rules":[]}"#, "el").is_err());
        assert!(parse(b"not json", "el").is_err());
    }

    #[test]
    fn a_changed_or_unsigned_file_is_refused() {
        // Signed with a throwaway key made for this test (`tauri signer
        // generate`, no password), never the real one.
        let data = br#"{"format":1,"language":"el","version":1,"rules":[]}"#;
        assert!(verify_with(data, TEST_SIG, TEST_PUBKEY).is_ok());
        let mut changed = data.to_vec();
        changed[30] = b'2';
        assert!(verify_with(&changed, TEST_SIG, TEST_PUBKEY).is_err(), "one byte changed");
        assert!(verify_with(data, "", TEST_PUBKEY).is_err(), "no signature");
        assert!(verify_with(data, "bm90IGEgc2lnbmF0dXJl", TEST_PUBKEY).is_err(), "garbage");
        assert!(verify(data, TEST_SIG).is_err(), "signed by another key than the update key");
    }

    #[test]
    fn the_packs_in_the_repository_are_signed_with_the_update_key() {
        for (lang, data, sig) in [
            ("el", include_bytes!("../../dictionaries/el.json").as_slice(), include_str!("../../dictionaries/el.json.sig")),
            ("all", include_bytes!("../../dictionaries/all.json").as_slice(), include_str!("../../dictionaries/all.json.sig")),
        ] {
            verify(data, sig).unwrap_or_else(|e| panic!("{lang}: {e}"));
            let pack = parse(data, lang).unwrap();
            assert!(!pack.rules.is_empty());
        }
        let index: Index = serde_json::from_str(include_str!("../../dictionaries/index.json")).unwrap();
        assert_eq!(index.packs["el"].rules, parse(include_bytes!("../../dictionaries/el.json"), "el").unwrap().rules.len() as u64);
    }

    const TEST_PUBKEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDM2NDA1QzczQTcxQUVBRjIKUldUeTZocW5jMXhBTnU1VlhRMFUrbU53RlNielBScGFhVnR1Z1BSek9QR2RyYVRwRnJNWXNMNDcK";
    const TEST_SIG: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVUeTZocW5jMXhBTmxGZVZ1QmJCZ0k0dUJhbGVhQ2FSRUttRnBXczAyekNRUDBqM0ZvZ2pJMFh1TWR3YWV1dHBIcllDQW1GLy9XQjJ3QkxlZjFOWTRTYllac3c0TEhpY2drPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzkxMDQwODAwCWZpbGU6ZGF0YS5qc29uCkp3eURsakZVRnBtK1BtWkxlM2hKSVgyUi8zVVlRNjYweFYzQW43Q2ZZLzdvUjg4bWdXS21xeWlwRFBwaUtxQURLUzJ2L1RrTCtUdXFMaVI0cTM1L0NRPT0K";

    /// Against the real files on GitHub (or FYF_PACKS_URL):
    ///   cargo test --lib packs::tests::live -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn live_index_and_signed_pack() {
        let index = index().await.expect("reached").expect("the list");
        for (lang, entry) in &index.packs {
            let client = client().unwrap();
            let data = fetch(&client, &format!("{lang}.json")).await.unwrap();
            let sig = fetch(&client, &format!("{lang}.json.sig")).await.unwrap();
            verify(&data, &String::from_utf8_lossy(&sig)).expect("signed with the update key");
            let pack = parse(&data, lang).unwrap();
            assert_eq!((pack.version, pack.rules.len() as u64), (entry.version, entry.rules));
            println!("{lang} v{} {} rules OK", pack.version, pack.rules.len());
        }
    }
}
