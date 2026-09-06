//! Safety net for AI cleanup: compare the candidate against the raw transcript
//! and reject rewrites that drift too far (invented content, dropped numbers,
//! translated text, changed names).

use std::collections::HashSet;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DivergenceReport {
    pub accepted: bool,
    pub reason: Option<String>,
    /// 0..1, share of raw content words that survived.
    pub content_overlap: f32,
    pub length_ratio: f32,
}

fn tokens(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '@' || c == '.' || c == '-' || c == '_' || c == '/'))
        .map(|t| t.trim_matches(|c: char| c == '.' || c == '-' || c == '\''))
        .filter(|t| t.chars().count() >= 2)
        .map(|t| t.to_string())
        .collect()
}

fn numbers(text: &str) -> HashSet<String> {
    text.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == ',' || c == ':'))
        .map(|t| t.trim_matches(|c: char| c == '.' || c == ','))
        .filter(|t| !t.is_empty() && t.chars().any(|c| c.is_ascii_digit()))
        .map(|t| t.to_string())
        .collect()
}

fn script_share(text: &str) -> (f32, f32) {
    let letters: Vec<char> = text.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.is_empty() {
        return (0.0, 0.0);
    }
    let greek = letters
        .iter()
        .filter(|c| {
            let c = **c;
            ('\u{0370}'..='\u{03FF}').contains(&c) || ('\u{1F00}'..='\u{1FFF}').contains(&c)
        })
        .count() as f32;
    let latin = letters.iter().filter(|c| c.is_ascii_alphabetic()).count() as f32;
    (greek / letters.len() as f32, latin / letters.len() as f32)
}

/// `max_drop` is the share of raw content words allowed to disappear
/// (fillers, repeats) before the rewrite is considered suspicious.
pub fn check(raw: &str, candidate: &str, max_drop: f32) -> DivergenceReport {
    let raw_t = tokens(raw);
    let cand_t = tokens(candidate);
    if raw_t.is_empty() {
        return DivergenceReport { accepted: !candidate.trim().is_empty() || raw.trim().is_empty(), reason: None, content_overlap: 1.0, length_ratio: 1.0 };
    }
    let cand_set: HashSet<&str> = cand_t.iter().map(|s| s.as_str()).collect();
    let survived = raw_t.iter().filter(|t| cand_set.contains(t.as_str())).count();
    let overlap = survived as f32 / raw_t.len() as f32;
    let length_ratio = cand_t.len() as f32 / raw_t.len() as f32;

    // numbers, urls, emails must survive exactly
    let raw_nums = numbers(raw);
    let cand_nums = numbers(candidate);
    let missing_numbers: Vec<&String> = raw_nums.iter().filter(|n| !cand_nums.contains(*n)).collect();
    if !missing_numbers.is_empty() {
        return DivergenceReport { accepted: false, reason: Some(format!("numbers changed: {}", missing_numbers.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "))), content_overlap: overlap, length_ratio };
    }
    // translation guard: the dominant script must not flip
    let (rg, rl) = script_share(raw);
    let (cg, cl) = script_share(candidate);
    if (rg > 0.6 && cl > 0.6) || (rl > 0.6 && cg > 0.6) {
        return DivergenceReport { accepted: false, reason: Some("language changed".into()), content_overlap: overlap, length_ratio };
    }
    if overlap < 1.0 - max_drop {
        return DivergenceReport { accepted: false, reason: Some(format!("too much content changed ({:.0}% kept)", overlap * 100.0)), content_overlap: overlap, length_ratio };
    }
    // invented content: candidate has many words the raw never had
    let raw_set: HashSet<&str> = raw_t.iter().map(|s| s.as_str()).collect();
    let novel = cand_t.iter().filter(|t| !raw_set.contains(t.as_str())).count() as f32 / cand_t.len().max(1) as f32;
    if novel > 0.35 && cand_t.len() > 6 {
        return DivergenceReport { accepted: false, reason: Some(format!("too many new words ({:.0}%)", novel * 100.0)), content_overlap: overlap, length_ratio };
    }
    if length_ratio > 1.6 && cand_t.len() > 8 {
        return DivergenceReport { accepted: false, reason: Some("rewrite is much longer than what was said".into()), content_overlap: overlap, length_ratio };
    }
    DivergenceReport { accepted: true, reason: None, content_overlap: overlap, length_ratio }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_light_cleanup() {
        let r = check("εε θέλω να πάω εμ στο σπίτι στις 5", "Θέλω να πάω στο σπίτι στις 5.", 0.4);
        assert!(r.accepted, "{:?}", r);
    }

    #[test]
    fn rejects_changed_numbers() {
        let r = check("meet at 5 with 3 people", "Meet at 6 with 3 people.", 0.4);
        assert!(!r.accepted);
        assert!(r.reason.unwrap().contains("numbers"));
    }

    #[test]
    fn rejects_translation() {
        let r = check("θέλω να πάω στο σπίτι μου τώρα", "I want to go to my house now.", 0.4);
        assert!(!r.accepted);
    }

    #[test]
    fn rejects_invented_content() {
        let r = check("send the report", "Send the quarterly financial report to the board of directors before Friday please.", 0.4);
        assert!(!r.accepted);
    }
}
