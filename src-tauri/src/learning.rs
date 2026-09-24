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

/// Records the edit and returns suggestions that reached the evidence threshold.
pub fn learn_from_edit(db: &Db, history_id: &str, before: &str, after: &str) -> anyhow::Result<Vec<Suggestion>> {
    let kind = classify(before, after);
    let mut out = Vec::new();
    let fixes = match kind {
        EditKind::WordCorrection { wrong, correct } => vec![(wrong, correct)],
        EditKind::Corrections(list) => list,
        other => {
            record_other(db, history_id, before, after, other)?;
            return Ok(out);
        }
    };
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
