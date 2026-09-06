//! Transcript cleanup pipeline:
//! 1. deterministic normalization (fillers, self-corrections, punctuation, capitalization)
//! 2. dictionary replacements (wrong -> correct)
//! 3. snippet expansion (spoken trigger -> saved text)
//! 4. optional AI cleanup (local llama.cpp or cloud), validated by
//! 5. divergence check (reject rewrites that changed the meaning)

pub mod deterministic;
pub mod dictionary;
pub mod questions;
pub mod divergence;
pub mod snippets;

use serde::{Deserialize, Serialize};

use crate::settings::CleanupIntensity;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CleanupOutcome {
    pub cleaned: String,
    /// Human readable list of what was applied ("dictionary: Λούραμ -> Luram").
    pub applied: Vec<String>,
    pub rule_ids: Vec<String>,
    pub snippet_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CleanupOptions {
    pub intensity: CleanupIntensity,
    pub remove_fillers: bool,
    pub resolve_self_corrections: bool,
    pub auto_punctuate: bool,
    pub auto_capitalize: bool,
    /// Style of the target application.
    pub trailing_punctuation: bool,
    pub capitalize_first: bool,
    /// "el", "en" or "auto"
    pub language: String,
}

/// Runs the deterministic part of the pipeline (steps 1 to 3).
pub fn run_deterministic(
    raw: &str,
    opts: &CleanupOptions,
    dict: &dictionary::DictionaryEngine,
    snips: &snippets::SnippetEngine,
) -> CleanupOutcome {
    let mut applied = Vec::new();
    let mut text = raw.trim().to_string();

    if opts.intensity != CleanupIntensity::Off {
        let det = deterministic::clean(&text, &deterministic::DetOptions {
            intensity: opts.intensity.clone(),
            remove_fillers: opts.remove_fillers,
            resolve_self_corrections: opts.resolve_self_corrections,
            auto_punctuate: opts.auto_punctuate && opts.trailing_punctuation,
            auto_capitalize: opts.auto_capitalize && opts.capitalize_first,
            language: opts.language.clone(),
        });
        if det.text != text {
            applied.extend(det.applied);
            text = det.text;
        }
    }

    let d = dict.apply(&text, opts.language.as_str());
    if d.text != text {
        applied.extend(d.applied.iter().map(|a| format!("dictionary: {a}")));
        text = d.text;
    }
    let s = snips.apply(&text);
    if s.text != text {
        applied.extend(s.applied.iter().map(|a| format!("snippet: {a}")));
        text = s.text;
    }

    CleanupOutcome { cleaned: text, applied, rule_ids: d.rule_ids, snippet_ids: s.snippet_ids }
}
