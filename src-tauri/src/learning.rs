//! Suggested learning from the user's own edits. Transparent: a clear one-word
//! (or short phrase) correction becomes a Dictionary suggestion at once, which
//! the user accepts or dismisses; rewrites are never turned into rules.

use crate::db::{Db, Suggestion};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditKind {
    /// One or two tokens replaced by one or two tokens, rest identical.
    WordCorrection { wrong: String, correct: String },
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
    // common prefix / suffix
    let mut p = 0;
    while p < bl.len() && p < al.len() && bl[p] == al[p] {
        p += 1;
    }
    let mut s = 0;
    while s < bl.len() - p && s < al.len() - p && bl[bl.len() - 1 - s] == al[al.len() - 1 - s] {
        s += 1;
    }
    let wrong: Vec<String> = b[p..b.len() - s].to_vec();
    let correct: Vec<String> = a[p..a.len() - s].to_vec();
    if wrong.is_empty() || correct.is_empty() || wrong.len() > 3 || correct.len() > 3 {
        return EditKind::Rewrite;
    }
    // the change must be a small part of the text
    if (wrong.len() as f32) / (b.len().max(1) as f32) > 0.5 && b.len() > 3 {
        return EditKind::Rewrite;
    }
    EditKind::WordCorrection { wrong: wrong.join(" "), correct: correct.join(" ") }
}

/// Records the edit and returns suggestions that reached the evidence threshold.
pub fn learn_from_edit(db: &Db, history_id: &str, before: &str, after: &str) -> anyhow::Result<Vec<Suggestion>> {
    let kind = classify(before, after);
    let mut out = Vec::new();
    match kind {
        EditKind::WordCorrection { wrong, correct } => {
            db.add_learning_event(Some(history_id), "word_correction", &wrong, &correct)?;
            let seen = db.count_learning_pairs("word_correction", &wrong, &correct)?;
            let reason = format!("You changed \"{wrong}\" to \"{correct}\" after dictating ({seen} time(s)).");
            let count = db.upsert_suggestion("dictionary", &wrong, &correct, &reason)?;
            if count >= 1 {
                out = db.list_suggestions()?.into_iter().filter(|s| s.status == "pending" && s.wrong == wrong && s.correct == correct).collect();
            }
        }
        EditKind::StyleOnly => {
            db.add_learning_event(Some(history_id), "style", before, after)?;
        }
        EditKind::Rewrite | EditKind::Deletion => {
            db.add_learning_event(Some(history_id), "rewrite", "", "")?;
        }
        EditKind::NoChange => {}
    }
    Ok(out)
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
