//! Personal Dictionary: deterministic wrong -> correct replacements applied
//! after speech recognition, plus the list of correct terms used as recognition
//! hints before it.
//!
//! Matching is Unicode aware: a "word boundary" is any position where a letter
//! or digit meets a non letter/digit, so Greek text works exactly like English
//! and "Λούραμ" never matches inside "Λούραμπ".

use std::collections::HashMap;

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
    /// In how many recent dictations each hint name was said, by `name_key`.
    /// Filled by `count_said`; empty until then.
    said: HashMap<String, usize>,
}

fn boundary_regex(wrong: &str, mode: &str, case_sensitive: bool) -> Option<Regex> {
    let escaped = fancy_regex::escape(wrong.trim());
    if escaped.is_empty() {
        return None;
    }
    // A word ends at anything that is not a letter or a digit, except a dot,
    // hyphen, @ or slash with a letter or digit on its far side: those join
    // "fuckyouflow.app" or "my-app" into one word the rule must not cut into.
    const BEFORE: &str = r"(?<![\p{L}\p{N}])(?<![\p{L}\p{N}][.@/\-])";
    const AFTER: &str = r"(?![\p{L}\p{N}])(?![.@/\-][\p{L}\p{N}])";
    let pattern = match mode {
        // exact: the whole transcript equals the wrong text
        "exact" => format!(r"^\s*{escaped}\s*$"),
        // phrase and whole_word behave the same for matching; phrase allows
        // flexible whitespace between words
        "phrase" => {
            let flexible = wrong.trim().split_whitespace().map(|w| fancy_regex::escape(w).into_owned()).collect::<Vec<String>>().join(r"\s+");
            format!("{BEFORE}{flexible}{AFTER}")
        }
        _ => format!("{BEFORE}{escaped}{AFTER}"),
    };
    RegexBuilder::new(&pattern).case_insensitive(!case_sensitive).build().ok()
}

impl DictionaryEngine {
    pub fn empty() -> Self {
        Self { rules: Vec::new(), said: HashMap::new() }
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
        Self { rules: compiled, said: HashMap::new() }
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Apply every enabled rule whose language scope matches.
    /// (See `script_language` for how a rule gets its scope.)
    pub fn apply(&self, text: &str, language: &str) -> DictResult {
        let mut out = text.to_string();
        let mut applied = Vec::new();
        let mut rule_ids = Vec::new();
        for c in &self.rules {
            if let Some(lang) = &c.rule.language {
                // An English rule can only ever match Latin letters, and Latin
                // letters inside a Greek dictation are English words or names,
                // so it runs wherever such letters are.
                let english_here = lang == "en" && out.chars().any(|ch| ch.is_ascii_alphabetic());
                if language != "auto" && language != "multi" && lang != language && !english_here {
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
            let replaced = super::replace_all_or_keep(&c.regex, &out, |caps: &fancy_regex::Captures<_>| preserve_case(&caps[0], &c.rule.correct)).to_string();
            if replaced != out {
                applied.push(format!("{} -> {}", c.rule.wrong, c.rule.correct));
                rule_ids.push(c.rule.id.clone());
                out = replaced;
            }
        }
        DictResult { text: out, applied, rule_ids }
    }

    /// The rules whose correct form may go into the recognition prompt.
    ///
    /// Rules the user typed in count as they are. A rule learned from one edit
    /// in History counts only when its correct form looks like a name (at most
    /// two words, a capital or a digit in each): the prompt is there for names,
    /// and an ordinary word or a corrected sentence only pulls the recognizer off
    /// course. Two words with a capital on the first alone open a sentence.
    fn hint_rules(&self) -> impl Iterator<Item = &DictionaryRule> {
        fn looks_like_a_name(t: &str) -> bool {
            let words: Vec<&str> = t.split_whitespace().collect();
            !words.is_empty() && words.len() <= 2 && words.iter().all(|w| w.chars().any(|c| c.is_uppercase() || c.is_ascii_digit()))
        }
        self.rules
            .iter()
            .map(|c| &c.rule)
            .filter(|r| r.use_as_hint)
            .filter(|r| r.source == "user" || looks_like_a_name(&r.correct))
    }

    /// Counts in how many of `texts` (the user's recent dictations) each hint
    /// name was said. The prompt has room for about a dozen names, and the
    /// order decides which ones get in. It used to follow how often a rule had
    /// corrected something, so a name the engine already heard right never
    /// fired its rule and went last: on 3 October 2026 the second most said
    /// name of the month was cut while a name said in none went in.
    pub fn count_said(&mut self, texts: &[String]) {
        // The word regex took 0.7 s over a month of dictations (debug build,
        // 3 October 2026). A plain search for the name's longest word first
        // leaves the regex only the dictations that may hold the name.
        fn fold(t: &str) -> String {
            t.to_lowercase().replace('ς', "σ")
        }
        let folded: Vec<String> = texts.iter().map(|t| fold(t)).collect();
        let mut said = HashMap::new();
        for rule in self.hint_rules() {
            let key = name_key(&rule.correct);
            if key.is_empty() || said.contains_key(&key) {
                continue;
            }
            let Some(regex) = boundary_regex(&rule.correct, "phrase", false) else { continue };
            let probe = key.split_whitespace().max_by_key(|w| w.chars().count()).map(fold).unwrap_or_default();
            let n = texts.iter().zip(&folded).filter(|(t, f)| f.contains(probe.as_str()) && regex.is_match(t).unwrap_or(false)).count();
            said.insert(key, n);
        }
        self.said = said;
    }

    /// Correct terms to bias the recognizer, the names said most first.
    /// Before anything was counted (a new install has no dictations yet) the
    /// order is the old one: the rule that corrected most, then the newest.
    pub fn hint_terms(&self, max: usize) -> Vec<String> {
        let said = |r: &DictionaryRule| self.said.get(&name_key(&r.correct)).copied().unwrap_or(0);
        let mut rules: Vec<&DictionaryRule> = self.hint_rules().collect();
        rules.sort_by(|a, b| said(b).cmp(&said(a)).then(b.apply_count.cmp(&a.apply_count)).then(b.updated_at.cmp(&a.updated_at)));
        let mut seen = std::collections::HashSet::new();
        rules.into_iter().map(|r| r.correct.trim().to_string()).filter(|t| !t.is_empty() && seen.insert(t.to_lowercase())).take(max).collect()
    }
}

/// One name, whatever its spelling in capitals: "WebDock" and "Webdock" are
/// the same name in the prompt and in the counts.
fn name_key(correct: &str) -> String {
    correct.trim().to_lowercase()
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

/// What the names in one prompt may cost, in engine tokens. The engine keeps
/// the last 223 tokens of a prompt and drops the front without a word, and the
/// front is where the most used names stand. Of those 223 the label and the
/// exemplar take about 40, and the 200 characters of earlier speech that
/// `join_prompt` adds took up to 119 in Greek.
const HINT_TOKEN_BUDGET: usize = 60;

/// A cautious guess at what a term costs. Against the engine's own vocabulary a
/// Latin letter came to half a token, a Greek letter to 0.8 and a Greek capital
/// to a whole one; the comma in front of the term is one more.
fn hint_cost(term: &str) -> usize {
    let greek = |c: char| ('\u{0370}'..='\u{03FF}').contains(&c) || ('\u{1F00}'..='\u{1FFF}').contains(&c);
    let tenths: usize = term
        .chars()
        .map(|c| match c {
            c if greek(c) && c.is_uppercase() => 10,
            c if greek(c) => 8,
            _ => 5,
        })
        .sum();
    1 + tenths.div_ceil(10)
}

/// Build the recognition prompt: dictionary terms joined as a natural phrase list.
/// Whisper treats the prompt as preceding text, so a comma list of names works
/// well. The terms arrive most used first and are taken for as long as
/// `HINT_TOKEN_BUDGET` lasts.
/// `language` is what the engine is told ("el", "en" or "auto"); `primary` is
/// the user's own language, which decides the exemplar when the engine is
/// left to guess. A Greek exemplar in front of a Turkish speaker would pull the
/// guess towards Greek, so only Greek gets Greek words.
pub fn build_hint_prompt(terms: &[String], language: &str, primary: &str) -> Option<String> {
    // The prompt is "preceding text": Whisper copies its punctuation habits, so a
    // short exemplar with question marks makes it punctuate questions more often.
    let greek_mixed = language == "auto" && primary == "el";
    let exemplar = match language {
        "el" => "Τι λες; Πώς σου φαίνεται; Ωραία, πάμε.",
        "auto" if greek_mixed => "Τι λες; Πώς σου φαίνεται; What do you think? Fine, let's go.",
        _ => "What do you think? How does it look? Fine, let's go.",
    };
    let mut spent = 0;
    let kept: Vec<&str> = terms
        .iter()
        .map(String::as_str)
        .take_while(|t| {
            spent += hint_cost(t);
            spent <= HINT_TOKEN_BUDGET
        })
        .collect();
    if kept.is_empty() {
        return Some(exemplar.to_string());
    }
    let joined = kept.join(", ");
    Some(match language {
        "el" => format!("Λεξιλόγιο: {joined}. {exemplar}"),
        "auto" if greek_mixed => format!("Λεξιλόγιο, vocabulary: {joined}. {exemplar}"),
        _ => format!("Vocabulary: {joined}. {exemplar}"),
    })
}

/// The language a rule belongs to, read from the letters of the words it
/// fixes: Greek letters make it a Greek rule, Latin letters an English one.
/// Anything else (digits only, mixed scripts) stays for every language.
pub fn script_language(wrong: &str) -> Option<String> {
    let greek = wrong.chars().any(|c| ('\u{0370}'..='\u{03FF}').contains(&c) || ('\u{1F00}'..='\u{1FFF}').contains(&c));
    let latin = wrong.chars().any(|c| c.is_ascii_alphabetic());
    match (greek, latin) {
        (true, false) => Some("el".into()),
        (false, true) => Some("en".into()),
        _ => None,
    }
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
    fn rules_know_their_language_from_their_letters() {
        assert_eq!(script_language("Λούραμ").as_deref(), Some("el"));
        assert_eq!(script_language("WebDoc").as_deref(), Some("en"));
        assert_eq!(script_language("τσάτζι GPT"), None);
    }

    /// Lu speaks Greek with English in it. An English rule must still fix an
    /// English word inside a Greek sentence, where the transcript counts as Greek.
    #[test]
    fn an_english_rule_reaches_english_words_in_a_greek_sentence() {
        let mut r = rule("WebDoc", "WebDock", "whole_word", false);
        r.language = Some("en".into());
        let e = DictionaryEngine::new(vec![r], vec![]);
        assert_eq!(e.apply("Ανέβασέ το στο WebDoc τώρα", "el").text, "Ανέβασέ το στο WebDock τώρα");
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

    /// A rule for a short word must not reach inside a web address or a
    /// hyphenated name. Measured on 23 September 2026: a learned rule
    /// "App -> up" turned "fuckyouflow.app" into "fuckyouflow.up" eight times,
    /// because the dot counted as the end of a word.
    #[test]
    fn a_word_inside_a_web_address_is_left_alone() {
        let e = DictionaryEngine::new(vec![rule("app", "up", "whole_word", false)], vec![]);
        assert_eq!(e.apply("Δες το fuckyouflow.app και το my-app.", "el").text, "Δες το fuckyouflow.app και το my-app.");
        assert_eq!(e.apply("Άνοιξε το app. Μετά το app, τέλος.", "el").text, "Άνοιξε το up. Μετά το up, τέλος.");
        let e = DictionaryEngine::new(vec![rule("open claw", "OpenClaw", "phrase", false)], vec![]);
        assert_eq!(e.apply("see open claw.io and open claw.", "en").text, "see open claw.io and OpenClaw.");
    }

    /// Hints tell the recognizer which names to expect. A learned correction of
    /// an ordinary word ("up") or a whole phrase fixed once ("Αυτό είναι λάθος")
    /// is no name, and as the most used rule "up" went first in every prompt.
    #[test]
    fn learned_corrections_hint_only_names() {
        let mut learned_word = rule("App", "up", "whole_word", false);
        learned_word.source = "suggested".into();
        learned_word.apply_count = 23;
        let mut learned_phrase = rule("Tienen lazos", "Αυτό είναι λάθος", "whole_word", false);
        learned_phrase.source = "suggested".into();
        let mut learned_name = rule("τσάτζι πητεί", "chatGPT", "whole_word", false);
        learned_name.source = "suggested".into();
        let typed_by_user = rule("λογο", "logo", "whole_word", false);
        let e = DictionaryEngine::new(vec![learned_word, learned_phrase, learned_name, typed_by_user], vec![]);
        let mut terms = e.hint_terms(10);
        terms.sort();
        assert_eq!(terms, vec!["chatGPT".to_string(), "logo".to_string()]);
    }

    /// "Then send" was learned from one edit and went into every prompt as a
    /// name, because its first letter is a capital.
    #[test]
    fn learned_phrase_that_opens_a_sentence_is_no_name() {
        let mut opener = rule("Στίλε το", "Στείλε το", "whole_word", false);
        opener.source = "suggested".into();
        let mut name = rule("blue harbor", "Blue Harbor", "whole_word", false);
        name.source = "suggested".into();
        let e = DictionaryEngine::new(vec![opener, name], vec![]);
        assert_eq!(e.hint_terms(10), vec!["Blue Harbor".to_string()]);
    }

    /// Forty names cost more tokens than the engine keeps, and what it drops is
    /// the front of the list: the names said most often.
    #[test]
    fn hint_prompt_keeps_the_most_used_names_within_budget() {
        let terms: Vec<String> = (0..40).map(|i| format!("Harborline{i}")).collect();
        let p = build_hint_prompt(&terms, "auto", "el").unwrap();
        assert!(p.starts_with("Λεξιλόγιο, vocabulary: Harborline0, "));
        assert!(!p.contains("Harborline39"));
        let names = p.split(": ").nth(1).unwrap().split(". ").next().unwrap();
        assert!(names.split(", ").map(hint_cost).sum::<usize>() <= HINT_TOKEN_BUDGET);
        assert!(p.ends_with("Fine, let's go."));

        let greek = vec!["Λευκός Πύργος".to_string()];
        assert_eq!(hint_cost(&greek[0]), 12);
        assert!(build_hint_prompt(&greek, "el", "el").unwrap().contains("Λευκός Πύργος"));
    }

    /// The engine hears "Velmora" right, so its rule never fires, while the
    /// rule for "Harborline" fired twenty times. Velmora is said in more
    /// dictations and goes first once the dictations are counted. Before that,
    /// as on a new install, the old order stands.
    #[test]
    fn hint_order_follows_the_names_said_most() {
        let mut fired = rule("harbor line", "Harborline", "phrase", false);
        fired.apply_count = 20;
        let quiet = rule("vel mora", "Velmora", "phrase", false);
        let two_words = rule("blue harbour", "Blue Harbor", "phrase", false);
        let mut e = DictionaryEngine::new(vec![fired, quiet, two_words], vec![]);
        assert_eq!(e.hint_terms(10)[0], "Harborline");

        let texts: Vec<String> = [
            "Ask VELMORA first.",
            "Velmora and Harborline today.",
            "velmora again, then the blue  harbor team",
            // a longer word and a joined name are other words
            "Velmoras is someone else",
            "harborline-velmora",
        ]
        .map(String::from)
        .to_vec();
        e.count_said(&texts);
        assert_eq!(e.said["velmora"], 3);
        assert_eq!(e.said["harborline"], 1);
        assert_eq!(e.said["blue harbor"], 1);
        // equal counts: the rule that corrected more goes first
        assert_eq!(e.hint_terms(10), vec!["Velmora".to_string(), "Harborline".to_string(), "Blue Harbor".to_string()]);
    }

    /// A Greek name ends in a small "ς" and in a capital "Σ"; both are said.
    #[test]
    fn said_counts_a_greek_name_in_capitals() {
        let mut e = DictionaryEngine::new(vec![rule("βελ μορας", "Βελμορας", "phrase", false)], vec![]);
        e.count_said(&["ο ΒΕΛΜΟΡΑΣ ήρθε".to_string(), "είπε ο Βελμορας.".to_string(), "Βελμοραστ".to_string()]);
        assert_eq!(e.said["βελμορας"], 2);
    }

    /// Prints the prompt the app would build from a copy of a real database
    /// (FYF_HINT_DB, kept outside the repository) in the old order and in the
    /// new one, and how long the counting took.
    #[test]
    #[ignore]
    fn live_hint_order() {
        let Ok(path) = std::env::var("FYF_HINT_DB") else { return };
        let db = crate::db::Db::open(std::path::Path::new(&path)).unwrap();
        let mut e = DictionaryEngine::new(db.list_rules().unwrap(), db.list_rule_exceptions().unwrap());
        let old = build_hint_prompt(&e.hint_terms(40), "auto", "el").unwrap();
        let texts = db.recent_final_texts(30, 3000).unwrap();
        let started = std::time::Instant::now();
        e.count_said(&texts);
        let took = started.elapsed();
        let new = build_hint_prompt(&e.hint_terms(40), "auto", "el").unwrap();
        let mut said: Vec<(&String, &usize)> = e.said.iter().collect();
        said.sort_by(|a, b| b.1.cmp(a.1));
        println!("dictations {}, counted in {took:?}\nsaid {said:?}\nold: {old}\nnew: {new}", texts.len());
    }

    #[test]
    fn hints() {
        let e = DictionaryEngine::new(vec![rule("Λούραμ", "Luram", "whole_word", false), rule("γλυκόζ μέντορ", "Glucose Mentor", "phrase", false)], vec![]);
        let terms = e.hint_terms(10);
        assert_eq!(terms.len(), 2);
        let p = build_hint_prompt(&terms, "el", "el").unwrap();
        assert!(p.starts_with("Λεξιλόγιο: "));
    }
}
