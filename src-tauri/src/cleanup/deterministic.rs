//! Rule-based cleanup that is always available and never invents content.
//! Greek and English. Everything here is reversible: the raw transcript is kept.

use once_cell::sync::Lazy;
use fancy_regex::Regex;

use crate::settings::CleanupIntensity;

#[derive(Debug, Clone)]
pub struct DetOptions {
    pub intensity: CleanupIntensity,
    pub remove_fillers: bool,
    pub resolve_self_corrections: bool,
    pub auto_punctuate: bool,
    pub auto_capitalize: bool,
    pub language: String,
}

#[derive(Debug, Clone, Default)]
pub struct DetResult {
    pub text: String,
    pub applied: Vec<String>,
}

/// Filler words that carry no meaning in dictation. Whole-word, case-insensitive.
/// Kept conservative: "like", "so", "well" are NOT here because they are often meaningful.
const FILLERS_EN: &[&str] = &["um", "umm", "uh", "uhh", "uhm", "erm", "er", "ehm", "em", "ah", "hmm", "mmm", "mm"];
const FILLERS_EL: &[&str] = &["ε", "εε", "εεε", "εμ", "εμμ", "μμ", "μμμ", "αα", "ααα", "χμ", "χμμ"];
/// Phrases removed only at Normal/Strong intensity.
const FILLER_PHRASES_EN: &[&str] = &["you know", "i mean", "sort of", "kind of"];
const FILLER_PHRASES_EL: &[&str] = &["ας πούμε", "να πούμε", "ξέρω γω", "πώς το λένε", "πώς να το πω"];
/// Only at Strong intensity (these are sometimes meaningful).
const FILLER_STRONG_EN: &[&str] = &["like", "basically", "actually", "literally"];
const FILLER_STRONG_EL: &[&str] = &["βασικά", "δηλαδή", "λοιπόν", "ρε"];

static WS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
static SPACE_BEFORE_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+([,.;:!?…])").unwrap());
static DOUBLE_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"([,.;:!?])\1+").unwrap());
static REPEAT_WORD: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(\p{L}{2,})\s+\1\b").unwrap());

/// "Tuesday, no, Friday" / "Τρίτη, όχι, Παρασκευή" -> keep the corrected word.
/// Pattern: <word(s)> <sep> <corrector> <sep> <word(s)>. We only resolve when the
/// replacement is 1..=3 words to avoid eating whole sentences.
// One word before and one word after the corrector. Articles that get doubled
// ("την την Παρασκευή") are collapsed by the repeated-word rule right after.
static SELF_CORR_EN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\p{L}+),\s*(?:no no|no|correction),\s*(\p{L}+)\b").unwrap()
});
static SELF_CORR_EL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\p{L}+),\s*(?:όχι όχι|διόρθωση),\s*(\p{L}+)\b").unwrap()
});

pub fn clean(input: &str, o: &DetOptions) -> DetResult {
    let mut applied = Vec::new();
    let mut text = input.trim().to_string();
    if text.is_empty() || o.intensity == CleanupIntensity::Off {
        return DetResult { text, applied };
    }

    // Whisper sometimes emits bracketed noise tags or leading dashes.
    let before = text.clone();
    text = strip_noise_tags(&text);
    if text != before {
        applied.push("removed noise tags".into());
    }

    if o.remove_fillers {
        let before = text.clone();
        text = remove_fillers(&text, &o.intensity);
        if text != before {
            applied.push("removed fillers".into());
        }
    }

    if o.resolve_self_corrections && o.intensity != CleanupIntensity::Light {
        let before = text.clone();
        text = resolve_self_corrections(&text);
        if text != before {
            applied.push("resolved self-correction".into());
        }
    }

    if o.intensity == CleanupIntensity::Strong {
        let before = text.clone();
        text = super::replace_all_or_keep(&REPEAT_WORD, &text, "$1").to_string();
        if text != before {
            applied.push("removed repeated word".into());
        }
    }

    // spacing and punctuation normalization
    let before = text.clone();
    text = normalize_spacing(&text);
    if text != before {
        applied.push("normalized spacing".into());
    }

    if looks_greek(&text) {
        // Whisper often writes a Latin "?" in Greek sentences; Greek uses ";"
        let before = text.clone();
        text = text.replace('?', ";");
        if text != before {
            applied.push("greek question mark".into());
        }
    }

    if o.auto_punctuate {
        let before = text.clone();
        text = super::questions::mark_questions(&super::questions::spoken_marks(&text));
        if text != before {
            applied.push("question mark".into());
        }
    }

    if o.auto_capitalize {
        let before = text.clone();
        text = capitalize_sentences(&text);
        if text != before {
            applied.push("capitalized".into());
        }
    }

    if o.auto_punctuate {
        let before = text.clone();
        text = ensure_terminal_punctuation(&text, &o.language);
        if text != before {
            applied.push("added final punctuation".into());
        }
    } else {
        // chat style: drop a single trailing period that Whisper added
        let before = text.clone();
        text = strip_single_trailing_period(&text);
        if text != before {
            applied.push("dropped trailing period".into());
        }
    }

    DetResult { text, applied }
}

fn strip_noise_tags(text: &str) -> String {
    static TAGS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\s*[\[\(](?:music|applause|laughter|noise|silence|blank_audio|inaudible|μουσική|γέλια|χειροκρότημα|ήχος|σιωπή)[^\]\)]*[\]\)]\s*").unwrap());
    let t = super::replace_all_or_keep(&TAGS, text, " ").to_string();
    let t = t.trim_start_matches(|c: char| c == '-' || c == '–' || c == ' ');
    t.to_string()
}

fn word_list_regex(words: &[&str]) -> Regex {
    let alts: Vec<String> = words.iter().map(|w| fancy_regex::escape(w).into_owned()).collect();
    // Word boundary that understands Greek: not preceded/followed by a letter.
    Regex::new(&format!(r"(?i)(?:^|(?<=[^\p{{L}}]))(?:{})(?:$|(?=[^\p{{L}}]))", alts.join("|"))).unwrap()
}

static FILLERS_LIGHT: Lazy<Regex> = Lazy::new(|| {
    let mut all: Vec<&str> = Vec::new();
    all.extend(FILLERS_EN);
    all.extend(FILLERS_EL);
    word_list_regex(&all)
});
static FILLERS_NORMAL: Lazy<Regex> = Lazy::new(|| {
    let mut all: Vec<&str> = Vec::new();
    all.extend(FILLER_PHRASES_EN);
    all.extend(FILLER_PHRASES_EL);
    word_list_regex(&all)
});
static FILLERS_STRONG: Lazy<Regex> = Lazy::new(|| {
    let mut all: Vec<&str> = Vec::new();
    all.extend(FILLER_STRONG_EN);
    all.extend(FILLER_STRONG_EL);
    word_list_regex(&all)
});

fn remove_fillers(text: &str, intensity: &CleanupIntensity) -> String {
    let mut t = super::replace_all_or_keep(&FILLERS_LIGHT, text, "").to_string();
    if matches!(intensity, CleanupIntensity::Strong) {
        t = super::replace_all_or_keep(&FILLERS_NORMAL, &t, "").to_string();
    }
    if matches!(intensity, CleanupIntensity::Strong) {
        t = super::replace_all_or_keep(&FILLERS_STRONG, &t, "").to_string();
    }
    // a filler often leaves ", ," or " , " behind
    static ORPHAN_COMMA: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*,\s*,+").unwrap());
    static LEADING_COMMA: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?:\s*[,;:.!?])+\s*").unwrap());
    static COMMA_BEFORE_END: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*,\s*([.!?;])").unwrap());
    t = super::replace_all_or_keep(&ORPHAN_COMMA, &t, ",").to_string();
    t = super::replace_all_or_keep(&LEADING_COMMA, &t, "").to_string();
    t = super::replace_all_or_keep(&COMMA_BEFORE_END, &t, "$1").to_string();
    t
}

fn resolve_self_corrections(text: &str) -> String {
    let t = super::replace_all_or_keep(&SELF_CORR_EN, text, "$2").to_string();
    super::replace_all_or_keep(&SELF_CORR_EL, &t, "$2").to_string()
}

fn normalize_spacing(text: &str) -> String {
    let t = super::replace_all_or_keep(&WS, text.trim(), " ").to_string();
    let t = super::replace_all_or_keep(&SPACE_BEFORE_PUNCT, &t, "$1").to_string();
    let t = super::replace_all_or_keep(&DOUBLE_PUNCT, &t, "$1").to_string();
    // ensure a space after sentence punctuation when followed by a letter; for the
    // period only when an uppercase letter follows, so "example.com" and
    // "file.bin" stay intact
    static AFTER_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"([!?;,])(\p{L})|(\.)(\p{Lu})").unwrap());
    let t = super::replace_all_or_keep(&AFTER_PUNCT, &t, "$1$3 $2$4").to_string();
    // repair URLs and domains damaged by the previous rule ("example. com")
    static DOMAIN_FIX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b([a-z0-9-]+)\. (com|gr|io|net|org|eu|ai|dev|app|co|uk|de|fr|info)\b").unwrap());
    super::replace_all_or_keep(&DOMAIN_FIX, &t, "$1.$2").to_string()
}

fn capitalize_sentences(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut capitalize_next = true;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if capitalize_next && c.is_alphabetic() {
            for u in c.to_uppercase() {
                out.push(u);
            }
            capitalize_next = false;
            continue;
        }
        if c == '.' || c == '!' || c == '?' || c == ';' {
            // Greek question mark is ';' but also used as semicolon in English text.
            // Only treat as sentence end when followed by whitespace.
            if matches!(chars.peek(), Some(' ') | None) && c != ';' {
                capitalize_next = true;
            } else if c == ';' && matches!(chars.peek(), Some(' ')) && looks_greek(text) {
                capitalize_next = true;
            }
        } else if c == '\n' {
            capitalize_next = true;
        } else if !c.is_whitespace() && c != '"' && c != '«' && c != '(' && c != '\'' && c != '“' {
            capitalize_next = false;
        }
        out.push(c);
    }
    out
}

pub fn looks_greek(text: &str) -> bool {
    let greek = text.chars().filter(|c| ('\u{0370}'..='\u{03FF}').contains(c) || ('\u{1F00}'..='\u{1FFF}').contains(c)).count();
    let latin = text.chars().filter(|c| c.is_ascii_alphabetic()).count();
    greek > latin
}

fn ensure_terminal_punctuation(text: &str, _language: &str) -> String {
    let t = text.trim_end();
    if t.is_empty() {
        return t.to_string();
    }
    let last = t.chars().last().unwrap();
    if matches!(last, '.' | '!' | '?' | ';' | ':' | '…' | '"' | '»' | ')' | ']' | '}') {
        return t.to_string();
    }
    // Do not add punctuation to a single token (a URL, a filename, a number, one word)
    if !t.contains(' ') {
        return t.to_string();
    }
    format!("{t}.")
}

fn strip_single_trailing_period(text: &str) -> String {
    let t = text.trim_end();
    if t.ends_with('.') && !t.ends_with("..") {
        let body = &t[..t.len() - 1];
        // keep the period if there is another sentence break inside (multi-sentence text)
        if body.contains(". ") || body.contains("! ") || body.contains("? ") {
            return t.to_string();
        }
        return body.to_string();
    }
    t.to_string()
}

pub fn word_count(text: &str) -> u64 {
    text.split_whitespace().filter(|w| w.chars().any(|c| c.is_alphanumeric())).count() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(intensity: CleanupIntensity) -> DetOptions {
        DetOptions { intensity, remove_fillers: true, resolve_self_corrections: true, auto_punctuate: true, auto_capitalize: true, language: "auto".into() }
    }

    #[test]
    fn removes_fillers_greek_and_english() {
        let r = clean("εε θέλω να πάω, εμ, στο σπίτι", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "Θέλω να πάω, στο σπίτι.");
        let r = clean("um I think uh we should go", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "I think we should go.");
    }

    #[test]
    fn resolves_self_corrections() {
        let r = clean("we meet on Tuesday, no, Friday at noon", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "We meet on Friday at noon.");
        let r = clean("θα έρθω την Τρίτη όχι την Παρασκευή", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "Θα έρθω την Τρίτη όχι την Παρασκευή.");
    }

    #[test]
    fn light_mode_keeps_meaningful_words() {
        let r = clean("basically I mean the OpenClaw server is like fine", &opts(CleanupIntensity::Light));
        assert_eq!(r.text, "Basically I mean the OpenClaw server is like fine.");
        let r = clean("basically I mean the OpenClaw server is like fine", &opts(CleanupIntensity::Strong));
        assert_eq!(r.text, "The OpenClaw server is fine.");
    }

    #[test]
    fn keeps_urls_and_numbers() {
        let r = clean("go to oneclickclaw.io and pay 3.14 euros", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "Go to oneclickclaw.io and pay 3.14 euros.");
        let r = clean("ggml-large-v3-q5_0.bin", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "Ggml-large-v3-q5_0.bin".to_string().replacen("Ggml", "Ggml", 1));
    }

    #[test]
    fn questions_get_their_mark_through_the_full_cleanup() {
        let o = opts(CleanupIntensity::Normal);
        assert_eq!(clean("τι μπορούμε να κάνουμε σε αυτό το κομμάτι.", &o).text, "Τι μπορούμε να κάνουμε σε αυτό το κομμάτι;");
        assert_eq!(clean("what do you think.", &o).text, "What do you think?");
        assert_eq!(clean("Πάμε για καφέ.", &o).text, "Πάμε για καφέ.");
    }

    #[test]
    fn greek_question_mark_capitalizes_next_sentence() {
        let r = clean("τι κάνεις; είμαι καλά", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "Τι κάνεις; Είμαι καλά.");
    }

    #[test]
    fn chat_style_drops_single_trailing_period() {
        let mut o = opts(CleanupIntensity::Normal);
        o.auto_punctuate = false;
        let r = clean("τα λέμε αύριο.", &o);
        assert_eq!(r.text, "Τα λέμε αύριο");
        let r = clean("Πρώτη πρόταση. Δεύτερη πρόταση.", &o);
        assert_eq!(r.text, "Πρώτη πρόταση. Δεύτερη πρόταση.");
    }

    #[test]
    fn latin_question_mark_becomes_greek() {
        let r = clean("Τι κάνεις? Είμαι καλά", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "Τι κάνεις; Είμαι καλά.");
        let r = clean("What time is it?", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "What time is it?");
    }

    #[test]
    fn single_letter_greek_filler_and_english_em() {
        let r = clean("Ε, Ε, Ε. Νομίζω ότι, Ε, πρέπει να φύγουμε", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "Νομίζω ότι, πρέπει να φύγουμε.");
        let r = clean("Em, I think we should, ah, deploy tonight", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "I think we should, deploy tonight.");
    }

    #[test]
    fn normal_keeps_repeated_words() {
        let r = clean("this is is a test", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "This is is a test.");
    }

    #[test]
    fn normal_preserves_meaning() {
        for text in ["Θέλω καφέ όχι τσάι.", "Θα έρθω μάλλον αύριο.",
            "Θέλω να πούμε κάτι.", "Πάμε σιγά σιγά.", "This is kind of important."] {
            assert_eq!(clean(text, &opts(CleanupIntensity::Normal)).text, text);
        }
        assert_eq!(clean("Τρίτη, διόρθωση, Παρασκευή", &opts(CleanupIntensity::Normal)).text, "Παρασκευή");
        assert_eq!(clean("  γεια?  ", &opts(CleanupIntensity::Off)).text, "γεια?");
    }

    #[test]
    fn off_is_identity_except_trim() {
        let r = clean("  εε γεια  ", &DetOptions { intensity: CleanupIntensity::Off, remove_fillers: false, resolve_self_corrections: false, auto_punctuate: false, auto_capitalize: false, language: "el".into() });
        assert_eq!(r.text, "εε γεια");
    }
}
