//! Snippets: a spoken trigger phrase expands to saved text. Whole-phrase match,
//! case-insensitive, punctuation around the trigger tolerated. Separate from the
//! Dictionary on purpose: a snippet inserts new content, a rule fixes a word.

use fancy_regex::{Regex, RegexBuilder};
use serde::Serialize;

use crate::db::Snippet;

#[derive(Debug, Clone, Serialize, Default)]
pub struct SnippetResult {
    pub text: String,
    pub applied: Vec<String>,
    pub snippet_ids: Vec<String>,
}

struct Compiled {
    snippet: Snippet,
    regex: Regex,
}

pub struct SnippetEngine {
    items: Vec<Compiled>,
}

impl SnippetEngine {
    pub fn empty() -> Self {
        Self { items: Vec::new() }
    }

    pub fn new(snippets: Vec<Snippet>) -> Self {
        let mut items = Vec::new();
        for s in snippets.into_iter().filter(|s| s.enabled) {
            let words: Vec<String> = s.trigger.split_whitespace().map(|w| fancy_regex::escape(w).into_owned()).collect();
            if words.is_empty() {
                continue;
            }
            // trigger may be followed by a period/comma that Whisper added
            let pattern = format!(r"(?:^|(?<=[^\p{{L}}\p{{N}}])){}(?:[.,!;:]?)(?:$|(?=[^\p{{L}}\p{{N}}]))", words.join(r"\s+"));
            if let Ok(regex) = RegexBuilder::new(&pattern).case_insensitive(true).build() {
                items.push(Compiled { snippet: s, regex });
            }
        }
        items.sort_by(|a, b| b.snippet.trigger.chars().count().cmp(&a.snippet.trigger.chars().count()));
        Self { items }
    }

    pub fn apply(&self, text: &str) -> SnippetResult {
        let mut out = text.to_string();
        let mut applied = Vec::new();
        let mut ids = Vec::new();
        for c in &self.items {
            if c.regex.is_match(&out).unwrap_or(false) {
                let expansion = c.snippet.expansion.clone();
                let replaced = c.regex.replace_all(&out, expansion.as_str()).to_string();
                if replaced != out {
                    applied.push(c.snippet.trigger.clone());
                    ids.push(c.snippet.id.clone());
                    out = replaced;
                }
            }
        }
        // When the whole transcript was just the trigger, do not leave a stray
        // capitalisation or period problem: return the expansion as is.
        SnippetResult { text: out.trim().to_string(), applied, snippet_ids: ids }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snip(trigger: &str, expansion: &str) -> Snippet {
        Snippet { id: uuid::Uuid::new_v4().to_string(), trigger: trigger.into(), expansion: expansion.into(), enabled: true, created_at: String::new(), updated_at: String::new(), apply_count: 0, last_applied_at: None }
    }

    #[test]
    fn expands_whole_phrase_only() {
        let e = SnippetEngine::new(vec![snip("my email signature", "Best,\nLu\nLuram AI Agency")]);
        let r = e.apply("Thanks for the update. My email signature.");
        assert_eq!(r.text, "Thanks for the update. Best,\nLu\nLuram AI Agency");
        assert_eq!(e.apply("my email signatures are long").text, "my email signatures are long");
    }

    #[test]
    fn greek_trigger() {
        let e = SnippetEngine::new(vec![snip("η υπογραφή μου", "Με εκτίμηση,\nΛου")]);
        assert_eq!(e.apply("Ευχαριστώ. Η υπογραφή μου").text, "Ευχαριστώ. Με εκτίμηση,\nΛου");
    }
}
