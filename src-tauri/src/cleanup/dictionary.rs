//! Personal Dictionary: deterministic wrong -> correct replacements applied
//! after speech recognition, plus the list of correct terms used as recognition
//! hints before it.
//!
//! Matching is Unicode aware: a "word boundary" is any position where a letter
//! or digit meets a non letter/digit, so Greek text works exactly like English
//! and "Λούραμ" never matches inside "Λούραμπ".

use fancy_regex::{Regex, RegexBuilder};
use serde::Serialize;

use crate::db::DictionaryRule;

#[derive(Debug, Clone, Serialize, Default)]
pub struct DictResult {
    pub text: String,
    /// "wrong -> correct" for each replacement that fired.
    pub applied: Vec<String>,
    pub rule_ids: Vec<String>,
}

struct Compiled {
    rule: DictionaryRule,
    regex: Regex,
    /// Contexts (surrounding text) in which the user said "do not replace here".
    exceptions: Vec<String>,
}

pub struct DictionaryEngine {
    rules: Vec<Compiled>,
}

fn boundary_regex(wrong: &str, mode: &str, case_sensitive: bool) -> Option<Regex> {
    let escaped = fancy_regex::escape(wrong.trim());
    if escaped.is_empty() {
        return None;
    }
    let pattern = match mode {
        // exact: the whole transcript equals the wrong text
        "exact" => format!(r"^\s*{escaped}\s*$"),
        // phrase and whole_word behave the same for matching; phrase allows
        // flexible whitespace between words
        "phrase" => {
            let flexible = wrong.trim().split_whitespace().map(|w| fancy_regex::escape(w).into_owned()).collect::<Vec<String>>().join(r"\s+");
            format!(r"(?:^|(?<=[^\p{{L}}\p{{N}}])){flexible}(?:$|(?=[^\p{{L}}\p{{N}}]))")
        }
        _ => format!(r"(?:^|(?<=[^\p{{L}}\p{{N}}])){escaped}(?:$|(?=[^\p{{L}}\p{{N}}]))"),
    };
    RegexBuilder::new(&pattern).case_insensitive(!case_sensitive).build().ok()
}

impl DictionaryEngine {
    pub fn empty() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn new(rules: Vec<DictionaryRule>, exceptions: Vec<(String, String)>) -> Self {
        let mut compiled = Vec::new();
        for rule in rules.into_iter().filter(|r| r.enabled) {
            if let Some(regex) = boundary_regex(&rule.wrong, &rule.match_mode, rule.case_sensitive) {
                let ex = exceptions.iter().filter(|(rid, _)| rid == &rule.id).map(|(_, c)| c.clone()).collect();
                compiled.push(Compiled { rule, regex, exceptions: ex });
            }
        }
        // longer "wrong" first so "Λούραμ ΑΙ" wins over "Λούραμ"
        compiled.sort_by(|a, b| b.rule.wrong.chars().count().cmp(&a.rule.wrong.chars().count()));
        Self { rules: compiled }
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Apply every enabled rule whose language scope matches.
    pub fn apply(&self, text: &str, language: &str) -> DictResult {
        let mut out = text.to_string();
        let mut applied = Vec::new();
        let mut rule_ids = Vec::new();
        for c in &self.rules {
            if let Some(lang) = &c.rule.language {
                if language != "auto" && language != "multi" && lang != language {
                    continue;
                }
            }
            if !c.regex.is_match(&out).unwrap_or(false) {
                continue;
            }
            // exception contexts: if the user marked this exact surrounding text as
            // a false positive, skip
            if c.exceptions.iter().any(|ctx| !ctx.is_empty() && out.contains(ctx.as_str())) {
                continue;
            }
            let replaced = c.regex.replace_all(&out, |caps: &fancy_regex::Captures<_>| preserve_case(&caps[0], &c.rule.correct)).to_string();
            if replaced != out {
                applied.push(format!("{} -> {}", c.rule.wrong, c.rule.correct));
                rule_ids.push(c.rule.id.clone());
                out = replaced;
            }
        }
        DictResult { text: out, applied, rule_ids }
    }

    /// Correct terms to bias the recognizer, most recently used first.
    pub fn hint_terms(&self, max: usize) -> Vec<String> {
        let mut rules: Vec<&DictionaryRule> = self.rules.iter().filter(|c| c.rule.use_as_hint).map(|c| &c.rule).collect();
        rules.sort_by(|a, b| b.apply_count.cmp(&a.apply_count).then(b.updated_at.cmp(&a.updated_at)));
        let mut seen = std::collections::HashSet::new();
        rules.into_iter().map(|r| r.correct.trim().to_string()).filter(|t| !t.is_empty() && seen.insert(t.to_lowercase())).take(max).collect()
    }
}

/// If the matched text was written in capitals or capitalized, keep that shape
/// unless the correct form has its own mixed case (brand names like "OpenClaw").
fn preserve_case(matched: &str, correct: &str) -> String {
    let correct_has_mixed = correct.chars().any(|c| c.is_uppercase()) && correct.chars().any(|c| c.is_lowercase());
    if correct_has_mixed {
        return correct.to_string();
    }
    let letters: Vec<char> = matched.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.is_empty() {
        return correct.to_string();
    }
    if letters.iter().all(|c| c.is_uppercase()) && letters.len() > 1 {
        return correct.to_uppercase();
    }
    if letters[0].is_uppercase() && correct.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
        let mut chars = correct.chars();
        return match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        };
    }
    correct.to_string()
}

/// Build the recognition prompt: dictionary terms joined as a natural phrase list.
/// Whisper treats the prompt as preceding text, so a comma list of names works
/// well and stays under the 224-token context.
pub fn build_hint_prompt(terms: &[String], language: &str) -> Option<String> {
    // The prompt is "preceding text": Whisper copies its punctuation habits, so a
    // short exemplar with question marks makes it punctuate questions more often.
    let exemplar = match language {
        "el" => "Τι λες; Πώς σου φαίνεται; Ωραία, πάμε.",
        "en" => "What do you think? How does it look? Fine, let's go.",
        _ => "Τι λες; Πώς σου φαίνεται; What do you think? Fine, let's go.",
    };
    if terms.is_empty() {
        return Some(exemplar.to_string());
    }
    let joined = terms.join(", ");
    let joined: String = joined.chars().take(600).collect();
    Some(match language {
        "el" => format!("Λεξιλόγιο: {joined}. {exemplar}"),
        "en" => format!("Vocabulary: {joined}. {exemplar}"),
        _ => format!("Λεξιλόγιο, vocabulary: {joined}. {exemplar}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(wrong: &str, correct: &str, mode: &str, cs: bool) -> DictionaryRule {
        DictionaryRule {
            id: uuid::Uuid::new_v4().to_string(),
            wrong: wrong.into(),
            correct: correct.into(),
            match_mode: mode.into(),
            case_sensitive: cs,
            language: None,
            app_scope: None,
            enabled: true,
            use_as_hint: true,
            source: "user".into(),
            created_at: String::new(),
            updated_at: String::new(),
            apply_count: 0,
            last_applied_at: None,
        }
    }

    #[test]
    fn greek_whole_word_replacement() {
        let e = DictionaryEngine::new(vec![rule("Λούραμ", "Luram", "whole_word", false)], vec![]);
        let r = e.apply("Το Λούραμ είναι εταιρεία. Λούραμπ όχι.", "el");
        assert_eq!(r.text, "Το Luram είναι εταιρεία. Λούραμπ όχι.");
        assert_eq!(r.applied, vec!["Λούραμ -> Luram"]);
    }

    #[test]
    fn case_insensitive_and_case_preserving() {
        let e = DictionaryEngine::new(vec![rule("open claw", "OpenClaw", "phrase", false)], vec![]);
        let r = e.apply("I use Open Claw and open  claw daily", "en");
        assert_eq!(r.text, "I use OpenClaw and OpenClaw daily");
        let e = DictionaryEngine::new(vec![rule("kavouras", "καβούρας", "whole_word", false)], vec![]);
        assert_eq!(e.apply("Kavouras is here", "en").text, "Καβούρας is here");
    }

    #[test]
    fn does_not_touch_inside_longer_words() {
        let e = DictionaryEngine::new(vec![rule("cat", "dog", "whole_word", false)], vec![]);
        assert_eq!(e.apply("concatenate the cat", "en").text, "concatenate the dog");
    }

    #[test]
    fn language_scope_respected() {
        let mut r = rule("Λούραμ", "Luram", "whole_word", false);
        r.language = Some("el".into());
        let e = DictionaryEngine::new(vec![r], vec![]);
        assert_eq!(e.apply("Λούραμ", "en").text, "Λούραμ");
        assert_eq!(e.apply("Λούραμ", "el").text, "Luram");
        assert_eq!(e.apply("Λούραμ", "auto").text, "Luram");
    }

    #[test]
    fn exceptions_block_false_positives() {
        let r = rule("μάρκα", "Marka", "whole_word", false);
        let id = r.id.clone();
        let e = DictionaryEngine::new(vec![r], vec![(id, "η μάρκα του αυτοκινήτου".into())]);
        assert_eq!(e.apply("η μάρκα του αυτοκινήτου", "el").text, "η μάρκα του αυτοκινήτου");
        assert_eq!(e.apply("πάμε στη μάρκα", "el").text, "πάμε στη Marka");
    }

    #[test]
    fn longer_rules_win() {
        let e = DictionaryEngine::new(vec![rule("Λούραμ", "Luram", "whole_word", false), rule("Λούραμ ΑΙ", "Luram AI Agency", "phrase", false)], vec![]);
        assert_eq!(e.apply("η Λούραμ ΑΙ", "el").text, "η Luram AI Agency");
    }

    #[test]
    fn hints() {
        let e = DictionaryEngine::new(vec![rule("Λούραμ", "Luram", "whole_word", false), rule("γλυκόζ μέντορ", "Glucose Mentor", "phrase", false)], vec![]);
        let terms = e.hint_terms(10);
        assert_eq!(terms.len(), 2);
        let p = build_hint_prompt(&terms, "el").unwrap();
        assert!(p.starts_with("Λεξιλόγιο: "));
    }
}
