//! Suggested learning from the user's own edits. Transparent: a clear one-word
//! (or short phrase) correction becomes a Dictionary suggestion at once, which
//! the user accepts or dismisses; rewrites are never turned into rules.

use crate::db::{Db, Suggestion};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditKind {
    /// One or two tokens replaced by one or two tokens, rest identical.
    WordCorrection { wrong: String, correct: String },
    /// Several such fixes in different places of the same edit.
    Corrections(Vec<(String, String)>),
    /// Only punctuation or capitalization differs.
    StyleOnly,
    /// Large change: do not learn.
    Rewrite,
    /// The user removed most of the text.
    Deletion,
    NoChange,
}

fn tokens(s: &str) -> Vec<String> {
    s.split_whitespace().map(|t| t.trim_matches(|c: char| !c.is_alphanumeric()).to_string()).filter(|t| !t.is_empty()).collect()
}

pub fn classify(before: &str, after: &str) -> EditKind {
    if before.trim() == after.trim() {
        return EditKind::NoChange;
    }
    let b = tokens(before);
    let a = tokens(after);
    if a.len() < b.len() / 2 {
        return EditKind::Deletion;
    }
    let bl: Vec<String> = b.iter().map(|t| t.to_lowercase()).collect();
    let al: Vec<String> = a.iter().map(|t| t.to_lowercase()).collect();
    if bl == al {
        return EditKind::StyleOnly;
    }
    // Each place the user changed, found separately. Taking everything between
    // the first and the last change as one span turned two small fixes in one
    // edit into a "rewrite" and learned nothing (24 September 2026).
    let hunks = change_hunks(&b, &a, &bl, &al);
    let removed: usize = hunks.iter().map(|(w, _)| w.len()).sum();
    // the changes must be a small part of the text
    if (removed as f32) / (b.len().max(1) as f32) > 0.5 && b.len() > 3 {
        return EditKind::Rewrite;
    }
    let mut fixes: Vec<(String, String)> = hunks
        .into_iter()
        .filter(|(w, c)| !w.is_empty() && !c.is_empty() && w.len() <= 3 && c.len() <= 3)
        .map(|(w, c)| (w.join(" "), c.join(" ")))
        .filter(|(w, c)| worth_a_rule(w, c))
        .collect();
    match fixes.len() {
        0 => EditKind::Rewrite,
        1 => {
            let (wrong, correct) = fixes.remove(0);
            EditKind::WordCorrection { wrong, correct }
        }
        _ => EditKind::Corrections(fixes),
    }
}

/// The runs of words that differ between `b` and `a`, each paired with what
/// replaced it, found by a longest-common-subsequence walk over the
/// lowercased words (`bl`, `al`). Returned in the original spelling.
fn change_hunks(b: &[String], a: &[String], bl: &[String], al: &[String]) -> Vec<(Vec<String>, Vec<String>)> {
    let (n, m) = (bl.len(), al.len());
    let mut lcs = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if bl[i] == al[j] { lcs[i + 1][j + 1] + 1 } else { lcs[i + 1][j].max(lcs[i][j + 1]) };
        }
    }
    let mut hunks = Vec::new();
    let (mut i, mut j) = (0, 0);
    let (mut gone, mut came): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    while i < n || j < m {
        if i < n && j < m && bl[i] == al[j] {
            if !gone.is_empty() || !came.is_empty() {
                hunks.push((std::mem::take(&mut gone), std::mem::take(&mut came)));
            }
            i += 1;
            j += 1;
        } else if j < m && (i == n || lcs[i][j + 1] >= lcs[i + 1][j]) {
            came.push(a[j].clone());
            j += 1;
        } else {
            gone.push(b[i].clone());
            i += 1;
        }
    }
    if !gone.is_empty() || !came.is_empty() {
        hunks.push((gone, came));
    }
    hunks
}

/// Whether one fix should become a rule that fires in every later dictation.
/// A misheard name or English term, yes. An everyday word swapped for one a
/// letter or two away ("λόγω" / "λόγο", "κόλληση" / "κόλλησε", "App" / "up")
/// is grammar that depends on the sentence: as a rule it rewrites every
/// correct use too, which is what "App -> up" did 24 times.
fn worth_a_rule(wrong: &str, correct: &str) -> bool {
    let has_latin = |s: &str| s.chars().any(|c| c.is_ascii_alphabetic());
    let has_greek = |s: &str| s.chars().any(|c| ('\u{0370}'..='\u{03FF}').contains(&c) || ('\u{1F00}'..='\u{1FFF}').contains(&c));
    // Heard in Greek letters, meant in Latin ones: an English word or a name.
    if has_greek(wrong) && has_latin(correct) && !has_greek(correct) {
        return true;
    }
    let single = !wrong.contains(' ') && !correct.contains(' ');
    let lw = wrong.to_lowercase();
    let lc = correct.to_lowercase();
    let same_script = has_latin(&lw) == has_latin(&lc) && has_greek(&lw) == has_greek(&lc);
    !(single && same_script && edit_distance(&lw, &lc) <= 2)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1; b.len() + 1];
        for (j, cb) in b.iter().enumerate() {
            cur[j + 1] = (prev[j] + usize::from(ca != cb)).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// The one-letter-away fixes `classify` leaves out, with how many times each
/// was made in this edit. They are grammar most of the time ("κόλληση" and
/// "κόλλησε" are both words), but a misheard word the engine makes up
/// ("WebDoc", "καταλαβαίες") looks just the same from here.
pub fn near_fixes(before: &str, after: &str) -> Vec<((String, String), usize)> {
    let b = tokens(before);
    let a = tokens(after);
    let bl: Vec<String> = b.iter().map(|t| t.to_lowercase()).collect();
    let al: Vec<String> = a.iter().map(|t| t.to_lowercase()).collect();
    let mut out: Vec<((String, String), usize)> = Vec::new();
    for (w, c) in change_hunks(&b, &a, &bl, &al) {
        if w.is_empty() || c.is_empty() || w.len() > 3 || c.len() > 3 {
            continue;
        }
        let pair = (w.join(" "), c.join(" "));
        if worth_a_rule(&pair.0, &pair.1) {
            continue;
        }
        match out.iter_mut().find(|(p, _)| *p == pair) {
            Some((_, n)) => *n += 1,
            None => out.push((pair, 1)),
        }
    }
    out
}

/// A word that reads as a name or a technical term: a dot, a digit, an
/// underscore or a capital after the first letter ("WebDoc", "fuckyouflow.up").
fn technical(word: &str) -> bool {
    word.chars().any(|c| c == '.' || c == '_' || c.is_ascii_digit()) || word.chars().skip(1).any(|c| c.is_uppercase())
}

/// How often a word may stand untouched elsewhere before it counts as one of
/// the user's real words, which a rule must never rewrite.
const KEPT_LIMIT: usize = 5;

/// A near fix earns a suggestion on real evidence: made twice in this edit,
/// made once before in another edit, or a name-like word. And never for a word
/// the user leaves alone in other transcripts, which is how "App" is kept safe.
fn near_fix_earns_a_rule(db: &Db, history_id: &str, wrong: &str, correct: &str, times_here: usize) -> anyhow::Result<bool> {
    if technical(wrong) {
        return Ok(true);
    }
    if is_one_of_their_words(db, history_id, wrong)? {
        return Ok(false);
    }
    let before = db.count_learning_pairs("near_correction", wrong, correct)?;
    Ok(times_here >= 2 || before >= 1)
}

/// A word the user leaves standing in other transcripts is one of their real
/// words ("λόγο", "App"), and a rule against it would rewrite every correct
/// use. A name-like word is exempt: the engine may write "WebDoc" every time
/// and the user may simply not have fixed it yet.
fn is_one_of_their_words(db: &Db, history_id: &str, wrong: &str) -> anyhow::Result<bool> {
    if technical(wrong) {
        return Ok(false);
    }
    let words: Vec<&str> = wrong.split_whitespace().collect();
    if words.len() > 1 {
        // A phrase made only of words the user keeps ("Μόνο εκεί") is real
        // speech, and a rule against it rewrites good text wherever those
        // words meet (an accepted suggestion, 2 October 2026). One word they
        // never say is enough to make it a mishearing.
        for w in words {
            if technical(w) || db.kept_occurrences(w, history_id)? < KEPT_LIMIT {
                return Ok(false);
            }
        }
        return Ok(true);
    }
    Ok(db.kept_occurrences(wrong, history_id)? >= KEPT_LIMIT)
}

/// Records the edit and returns suggestions that reached the evidence threshold.
pub fn learn_from_edit(db: &Db, history_id: &str, before: &str, after: &str) -> anyhow::Result<Vec<Suggestion>> {
    let kind = classify(before, after);
    let mut out = Vec::new();
    let mut fixes = match &kind {
        EditKind::WordCorrection { wrong, correct } => vec![(wrong.clone(), correct.clone())],
        EditKind::Corrections(list) => list.clone(),
        _ => Vec::new(),
    };
    // Even a clear fix never becomes a rule against one of the user's own
    // words: "λόγο -> logo" would have broken every "λόγο" (found on Lu's
    // real history, 26 September 2026).
    let mut kept = Vec::with_capacity(fixes.len());
    for (w, c) in fixes {
        if !is_one_of_their_words(db, history_id, &w)? {
            kept.push((w, c));
        }
    }
    fixes = kept;
    // The near fixes, weighed against what the user has done before. Each is
    // recorded first, so the second time it is made anywhere it counts.
    if !matches!(kind, EditKind::NoChange | EditKind::StyleOnly | EditKind::Deletion) {
        for ((wrong, correct), times) in near_fixes(before, after) {
            let earns = near_fix_earns_a_rule(db, history_id, &wrong, &correct, times)?;
            db.add_learning_event(Some(history_id), "near_correction", &wrong, &correct)?;
            if earns {
                fixes.push((wrong, correct));
            }
        }
    }
    if fixes.is_empty() {
        record_other(db, history_id, before, after, kind)?;
        return Ok(out);
    }
    for (wrong, correct) in fixes {
        db.add_learning_event(Some(history_id), "word_correction", &wrong, &correct)?;
        let seen = db.count_learning_pairs("word_correction", &wrong, &correct)?;
        let reason = format!("You changed \"{wrong}\" to \"{correct}\" after dictating ({seen} time(s)).");
        let count = db.upsert_suggestion("dictionary", &wrong, &correct, &reason)?;
        if count >= 1 {
            out.extend(db.list_suggestions()?.into_iter().filter(|s| s.status == "pending" && s.wrong == wrong && s.correct == correct));
        }
    }
    Ok(out)
}

/// Once, after the update that made learning smarter: every edit that taught
/// nothing is read again with today's rules, and whatever it earns shows up in
/// Suggestions for the user to accept or dismiss. Nothing goes into the
/// Dictionary on its own.
pub fn relearn_past_edits(db: &Db) -> anyhow::Result<usize> {
    let mut found = 0;
    for (id, before, after) in db.edits_that_taught_nothing()? {
        found += learn_from_edit(db, &id, &before, &after)?.len();
    }
    Ok(found)
}

/// Once, for rules learned before the kept-word guard existed. A learned rule
/// (never one the user typed) whose wrong word the user had left standing in
/// KEPT_LIMIT or more transcripts before the rule was made is switched off,
/// never deleted: the Dictionary page can turn it back on. On 26 September
/// 2026 one History edit taught "το -> αυτό", and every "το" in 19 dictations
/// after it became "αυτό". Case-only fixes and name-like words are brand
/// spellings and stay. Returns how many rules were switched off.
pub fn switch_off_rules_against_kept_words(db: &Db) -> anyhow::Result<usize> {
    let mut off = 0;
    for mut rule in db.list_rules()? {
        if rule.source == "user" || !rule.enabled {
            continue;
        }
        if rule.wrong.to_lowercase() == rule.correct.to_lowercase() || technical(&rule.wrong) || technical(&rule.correct) {
            continue;
        }
        if db.kept_before(&rule.wrong, rule.case_sensitive, &rule.created_at)? >= KEPT_LIMIT {
            rule.enabled = false;
            db.upsert_rule(&rule)?;
            off += 1;
        }
    }
    Ok(off)
}

fn record_other(db: &Db, history_id: &str, before: &str, after: &str, kind: EditKind) -> anyhow::Result<()> {
    match kind {
        EditKind::WordCorrection { .. } | EditKind::Corrections(_) => {}
        EditKind::StyleOnly => {
            db.add_learning_event(Some(history_id), "style", before, after)?;
        }
        EditKind::Rewrite | EditKind::Deletion => {
            db.add_learning_event(Some(history_id), "rewrite", "", "")?;
        }
        EditKind::NoChange => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_edits() {
        assert_eq!(classify("Το Λούραμ είναι εδώ.", "Το Luram είναι εδώ."), EditKind::WordCorrection { wrong: "Λούραμ".into(), correct: "Luram".into() });
        assert_eq!(classify("γεια σου κόσμε.", "Γεια σου, κόσμε!"), EditKind::StyleOnly);
        assert_eq!(classify("send the report today please", "please write a long essay about reports tomorrow"), EditKind::Rewrite);
        assert_eq!(classify("one two three four five six", "one"), EditKind::Deletion);
        assert_eq!(classify("same", "same "), EditKind::NoChange);
    }

    /// Two fixes in two places of one edit used to become one long "change"
    /// from the first to the last, which counted as a rewrite: nothing learned.
    #[test]
    fn two_separate_fixes_in_one_edit_are_both_learned() {
        assert_eq!(
            classify("Πάμε στο άστρα και μετά στο σλακ τώρα αμέσως", "Πάμε στο astra και μετά στο Slack τώρα αμέσως"),
            EditKind::Corrections(vec![("άστρα".into(), "astra".into()), ("σλακ".into(), "Slack".into())])
        );
    }

    /// Lu's own edit of 24 September 2026, which taught the app nothing. The
    /// English name is worth a rule; "ΑΒΒ, λόγω" into four words is too big a
    /// change to guess from, so it stays a one-off.
    #[test]
    fn the_edit_of_24_september_teaches_astra() {
        let before = "Θέλω να δούμε αν το ΑΒΒ, λόγω του κόστους, και μετά το άστρα, μπαίνει στο σχέδιο ή όχι σήμερα.";
        let after = "Θέλω να δούμε αν το Α η Β λόγο του κόστους, και μετά το astra, μπαίνει στο σχέδιο ή όχι σήμερα.";
        assert_eq!(classify(before, after), EditKind::WordCorrection { wrong: "άστρα".into(), correct: "astra".into() });
    }

    /// A Greek word swapped for one a letter away is grammar that depends on
    /// the sentence ("λόγω" and "λόγο" are both right somewhere). A rule for it
    /// would change every correct use too, the way "App -> up" did.
    #[test]
    fn near_identical_everyday_words_are_not_turned_into_rules() {
        assert_eq!(classify("Και τι κόλληση ακριβώς εδώ τώρα;", "Και τι κόλλησε ακριβώς εδώ τώρα;"), EditKind::Rewrite);
        assert_eq!(classify("Το έκανα λόγω του κόστους μόνο.", "Το έκανα λόγο του κόστους μόνο."), EditKind::Rewrite);
        assert_eq!(classify("Level App ανεβαίνει τώρα εδώ", "Level up ανεβαίνει τώρα εδώ"), EditKind::Rewrite);
        // A misheard name is still learned.
        assert_eq!(classify("Ρώτα το τσάτζι πητεί τώρα", "Ρώτα το chatGPT τώρα"), EditKind::WordCorrection { wrong: "τσάτζι πητεί".into(), correct: "chatGPT".into() });
    }

    #[test]
    fn every_fix_in_an_edit_becomes_a_suggestion() {
        let db = Db::open_in_memory().unwrap();
        let got = learn_from_edit(&db, "h1", "Πάμε στο άστρα και μετά στο σλακ τώρα αμέσως", "Πάμε στο astra και μετά στο Slack τώρα αμέσως").unwrap();
        let mut pairs: Vec<(String, String)> = got.iter().map(|s| (s.wrong.clone(), s.correct.clone())).collect();
        pairs.sort();
        assert_eq!(pairs, vec![("άστρα".to_string(), "astra".to_string()), ("σλακ".to_string(), "Slack".to_string())]);
    }

    /// Lu's edit of 26 September 2026: the same misheard name fixed three
    /// times in one text. One letter away, and still plainly worth a rule.
    #[test]
    fn a_fix_made_twice_in_one_edit_is_learned() {
        let db = Db::open_in_memory().unwrap();
        let got = learn_from_edit(&db, "h1", "Στο WebDoc και μετά πάλι στο WebDoc και ξανά WebDoc εδώ", "Στο WebDock και μετά πάλι στο WebDock και ξανά WebDock εδώ").unwrap();
        assert_eq!(got.iter().map(|s| (s.wrong.as_str(), s.correct.as_str())).collect::<Vec<_>>(), vec![("WebDoc", "WebDock")]);
    }

    /// A near fix made once waits; made again in a later edit, it is learned.
    #[test]
    fn a_near_fix_is_learned_the_second_time() {
        let db = Db::open_in_memory().unwrap();
        assert!(learn_from_edit(&db, "h1", "Αυτό είναι τρίπιο πολύ εδώ", "Αυτό είναι τρύπιο πολύ εδώ").unwrap().is_empty());
        let second = learn_from_edit(&db, "h2", "Το βάζο είναι τρίπιο από κάτω", "Το βάζο είναι τρύπιο από κάτω").unwrap();
        assert_eq!(second.len(), 1);
        assert_eq!((second[0].wrong.as_str(), second[0].correct.as_str()), ("τρίπιο", "τρύπιο"));
    }

    /// "App" stands untouched in many other transcripts, so however often it
    /// is changed to "up" in one of them, no rule is made against it.
    #[test]
    fn a_word_the_user_keeps_elsewhere_is_never_learned() {
        let db = Db::open_in_memory().unwrap();
        for i in 0..6 {
            db.insert_test_history(&format!("k{i}"), "Άνοιξε το App τώρα");
        }
        let got = learn_from_edit(&db, "h1", "Level App και App ξανά εδώ", "Level up και up ξανά εδώ").unwrap();
        assert!(got.is_empty());
    }

    #[test]
    fn a_common_word_is_never_replaced_even_by_an_english_one() {
        let db = Db::open_in_memory().unwrap();
        for i in 0..6 {
            db.insert_test_history(&format!("k{i}"), "Το έκανα για αυτό τον λόγο χθες");
        }
        assert!(learn_from_edit(&db, "h1", "Φτιάξε το λόγο της εταιρείας", "Φτιάξε το logo της εταιρείας").unwrap().is_empty());
    }

    /// 2 October 2026: an accepted suggestion turned the two everyday words
    /// "Μόνο εκεί" into another phrase. Every word on the heard side is one
    /// the user keeps elsewhere, so the phrase is real speech and stays.
    #[test]
    fn a_phrase_of_everyday_words_is_never_learned() {
        let db = Db::open_in_memory().unwrap();
        for i in 0..6 {
            db.insert_test_history(&format!("k{i}"), "Θέλω μόνο αυτό, πήγαινε εκεί και περίμενε");
        }
        let got = learn_from_edit(&db, "h1", "Το είπα μόνο εκεί χθες", "Το είπα με όλα χθες").unwrap();
        assert!(got.is_empty(), "{:?}", got.iter().map(|s| (&s.wrong, &s.correct)).collect::<Vec<_>>());
        // a phrase with a word the user never says still teaches
        let got = learn_from_edit(&db, "h2", "Άνοιξε το κλάβντ κόουντ τώρα", "Άνοιξε το Claude Code τώρα").unwrap();
        assert_eq!(got.len(), 1, "{got:?}");
    }

    /// 26 September 2026, 13:48: one History edit changed "το" to "αυτό" and
    /// the old learning made it a rule for every dictation. Today's learning
    /// sees "το" standing in other transcripts and learns nothing.
    #[test]
    fn a_function_word_edit_is_never_learned() {
        let db = Db::open_in_memory().unwrap();
        for i in 0..6 {
            db.insert_test_history(&format!("k{i}"), "Πες μου το τώρα, να το δούμε");
        }
        assert!(learn_from_edit(&db, "h1", "Φτιάξε το τώρα", "Φτιάξε αυτό τώρα").unwrap().is_empty());
    }

    fn learned(db: &Db, wrong: &str, correct: &str, source: &str) {
        db.upsert_rule(&crate::db::DictionaryRule {
            id: crate::db::new_id(),
            wrong: wrong.into(),
            correct: correct.into(),
            match_mode: "whole_word".into(),
            case_sensitive: true,
            language: None,
            app_scope: None,
            enabled: true,
            use_as_hint: true,
            source: source.into(),
            created_at: crate::db::ts_now(),
            updated_at: crate::db::ts_now(),
            apply_count: 0,
            last_applied_at: None,
        })
        .unwrap();
    }

    /// The rule that came before the guard: switched off at the next start.
    /// A rule the user typed, a brand spelling, a name and a rule with too
    /// little evidence all stay on.
    #[test]
    fn an_old_rule_against_a_kept_word_is_switched_off() {
        let db = Db::open_in_memory().unwrap();
        for i in 0..6 {
            db.insert_test_history(&format!("k{i}"), "Πες μου το τώρα στο oneclickclaw και στο WebDoc");
        }
        db.insert_test_history("once", "Και άσε το αυτό");
        learned(&db, "το", "αυτό", "suggested");
        learned(&db, "το", "αυτό", "user");
        learned(&db, "oneclickclaw", "OneClickClaw", "suggested");
        learned(&db, "WebDoc", "WebDock", "suggested");
        learned(&db, "άσε", "πιάσε", "suggested");
        assert_eq!(switch_off_rules_against_kept_words(&db).unwrap(), 1);
        let on: Vec<(String, String, bool)> = db.list_rules().unwrap().into_iter().map(|r| (r.correct, r.source, r.enabled)).collect();
        assert!(on.contains(&("αυτό".into(), "suggested".into(), false)));
        assert!(on.contains(&("αυτό".into(), "user".into(), true)));
        assert_eq!(on.iter().filter(|r| r.2).count(), 4);
        // it runs once, and a second pass finds nothing more
        assert_eq!(switch_off_rules_against_kept_words(&db).unwrap(), 0);
    }

    /// The engine writes "WebDoc" every time and the user leaves most of them;
    /// that does not make it one of their words.
    #[test]
    fn a_name_the_engine_keeps_mishearing_is_still_learned() {
        let db = Db::open_in_memory().unwrap();
        for i in 0..6 {
            db.insert_test_history(&format!("k{i}"), "Ανέβασέ το στο WebDoc");
        }
        let got = learn_from_edit(&db, "h1", "Μπες στο WebDoc τώρα", "Μπες στο WebDock τώρα").unwrap();
        assert_eq!(got.len(), 1);
    }

    #[test]
    fn suggestion_appears_on_the_first_correction() {
        let db = Db::open_in_memory().unwrap();
        let first = learn_from_edit(&db, "h1", "Το Λούραμ είναι εδώ.", "Το Luram είναι εδώ.").unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].evidence_count, 1);
        assert_eq!((first[0].wrong.as_str(), first[0].correct.as_str()), ("Λούραμ", "Luram"));
        let second = learn_from_edit(&db, "h2", "Πάμε στο Λούραμ αύριο.", "Πάμε στο Luram αύριο.").unwrap();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].evidence_count, 2);
    }
}

/// Manual check against a copy of a real database:
/// FYF_DB_COPY=<path> cargo test --lib relearn_on_a_copy -- --ignored --nocapture
#[cfg(test)]
mod real_copy {
    #[test]
    #[ignore]
    fn relearn_on_a_copy() {
        let Ok(p) = std::env::var("FYF_DB_COPY") else { return };
        let db = crate::db::Db::open(std::path::Path::new(&p)).unwrap();
        let langs = db.assign_rule_languages().unwrap();
        let n = super::relearn_past_edits(&db).unwrap();
        println!("RULES_GIVEN_LANGUAGE {langs}");
        println!("NEW_SUGGESTIONS {n}");
        for s in db.list_suggestions().unwrap().into_iter().filter(|s| s.status == "pending") {
            println!("PENDING {} -> {}", s.wrong, s.correct);
        }
        for r in db.list_rules().unwrap() {
            println!("RULE {:?} {}", r.language, r.wrong);
        }
    }

    /// The kept-word check on a copy of a real database: prints the rules it
    /// would switch off. Run with FYF_DB_COPY pointing at a copy, never the
    /// live file.
    #[test]
    #[ignore]
    fn kept_word_check_on_a_copy() {
        let Ok(p) = std::env::var("FYF_DB_COPY") else { return };
        let db = crate::db::Db::open(std::path::Path::new(&p)).unwrap();
        let before: Vec<_> = db.list_rules().unwrap().into_iter().filter(|r| r.enabled).map(|r| r.id).collect();
        let n = super::switch_off_rules_against_kept_words(&db).unwrap();
        println!("SWITCHED_OFF {n}");
        for r in db.list_rules().unwrap().into_iter().filter(|r| !r.enabled && before.contains(&r.id)) {
            println!("OFF {} -> {}", r.wrong, r.correct);
        }
    }
}
