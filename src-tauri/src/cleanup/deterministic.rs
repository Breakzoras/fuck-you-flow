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
/// A dot glued to a small letter after it opens a domain ending (".ai",
/// ".com", ".onion") and keeps the space in front of it: "όνομα σε .gr" came
/// out as "όνομα σε.gr" (2 October 2026). A dot followed by a space, another
/// mark, a capital or the end of the text is a full stop and loses the space.
static SPACE_BEFORE_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+([,;:!?…]|\.(?!\p{Ll}))").unwrap());
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

    let before = text.clone();
    text = drop_piece_openers(&text);
    if text != before {
        applied.push("joined a sentence cut in two".into());
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
        let before = text.clone();
        text = super::questions::soften_inner_semicolons(&text);
        if text != before {
            applied.push("pause mark inside a sentence".into());
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
        if looks_greek(&text) {
            let before = text.clone();
            text = lower_inner_function_words(&text);
            if text != before {
                applied.push("lower case inside a sentence".into());
            }
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
    // Subtitle credits Whisper learned from videos and writes after a pause.
    // Nobody dictates them, so they go wherever they appear.
    static CREDITS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\s*(?:υπότιτλοι\s+authorwave|subtitles by the amara\.org community)\.?").unwrap());
    // The credit sometimes comes back cut short as a last lone word. Removed
    // only after a credit was found, so a sentence that really ends in
    // "υπότιτλοι" keeps it.
    static CREDIT_TAIL: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\s+υπότιτλο[ιί]?\.?\s*$").unwrap());
    let mut t = super::replace_all_or_keep(&TAGS, text, " ").to_string();
    if CREDITS.is_match(&t).unwrap_or(false) {
        t = super::replace_all_or_keep(&CREDITS, &t, " ").to_string();
        t = super::replace_all_or_keep(&CREDIT_TAIL, &t, "").to_string();
    }
    let t = t.trim_start_matches(|c: char| c == '-' || c == '–' || c == ' ');
    t.to_string()
}

/// A long dictation is heard in pieces while it is spoken. When a cut falls
/// inside a sentence, the engine opens the next piece with "..." glued to its
/// first word ("στο... ...σπίτι", "αρχεία, ...τα"). The later rules made that
/// "στο.σπίτι" and "αρχεία.τα" (11 of 1054 real dictations, 27 September
/// 2026). The opening mark goes before those rules run; a pause mark in front
/// of it is then handled like any other pause. Only before a lower-case word:
/// before a capital the old sentence break stays.
fn drop_piece_openers(text: &str) -> String {
    static OPENER: Lazy<Regex> = Lazy::new(|| Regex::new(r"(^|\s)(?:\.{3,}|…)(?=\p{Ll})").unwrap());
    super::replace_all_or_keep(&OPENER, text, "$1").trim_start().to_string()
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
    // a filler that opened a sentence ("… changelog. Ε, what") leaves its
    // comma right after the full stop
    static COMMA_AFTER_END: Lazy<Regex> = Lazy::new(|| Regex::new(r"([.!?;])\s*,\s*").unwrap());
    t = super::replace_all_or_keep(&ORPHAN_COMMA, &t, ",").to_string();
    t = super::replace_all_or_keep(&LEADING_COMMA, &t, "").to_string();
    t = super::replace_all_or_keep(&COMMA_BEFORE_END, &t, "$1").to_string();
    t = super::replace_all_or_keep(&COMMA_AFTER_END, &t, "$1 ").to_string();
    t
}

fn resolve_self_corrections(text: &str) -> String {
    let t = super::replace_all_or_keep(&SELF_CORR_EN, text, "$2").to_string();
    super::replace_all_or_keep(&SELF_CORR_EL, &t, "$2").to_string()
}

fn normalize_spacing(text: &str) -> String {
    let t = super::replace_all_or_keep(&WS, text.trim(), " ").to_string();
    // A pause in the middle of a sentence arrives as "..." with the next word
    // in lower case. The rules below folded it into one full stop and the next
    // word got a capital: a sentence break nobody said (102 of 1037 real
    // dictations, 27 September 2026). The pause simply goes.
    static MID_ELLIPSIS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*(?:\.{3,}|…)(?:\s*(?:\.{3,}|…))*\s+(?=\p{Ll})").unwrap());
    let t = super::replace_all_or_keep(&MID_ELLIPSIS, &t, " ").to_string();
    let t = super::replace_all_or_keep(&SPACE_BEFORE_PUNCT, &t, "$1").to_string();
    let t = super::replace_all_or_keep(&DOUBLE_PUNCT, &t, "$1").to_string();
    // ensure a space after sentence punctuation when followed by a letter; for the
    // period only when an uppercase letter follows, so "example.com" and
    // "file.bin" stay intact
    static AFTER_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"([!?;,])(\p{L})|(\.)(\p{Lu})").unwrap());
    let t = super::replace_all_or_keep(&AFTER_PUNCT, &t, "$1$3 $2$4").to_string();
    // "ό,τι" is one Greek word that is written with a comma inside it; the
    // rule above just split it. Only the accented ό exists in that word.
    static O_TI: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?<!\p{L})([όΌ]), ([τΤ]ι)(?!\p{L})").unwrap());
    let t = super::replace_all_or_keep(&O_TI, &t, "$1,$2").to_string();
    // repair URLs and domains damaged by the previous rule ("example. com")
    static DOMAIN_FIX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b([a-z0-9-]+)\. (com|gr|io|net|org|eu|ai|dev|app|co|uk|de|fr|info)\b").unwrap());
    super::replace_all_or_keep(&DOMAIN_FIX, &t, "$1.$2").to_string()
}

/// Abbreviations that announce what comes next, written here without their
/// last full stop. The sentence always goes on after them, so that full stop
/// ends nothing. "κλπ." and "etc." are left out on purpose: they close a list,
/// and a list often closes the sentence.
const LEADING_ABBREVIATIONS: &[&str] = &["π.χ", "πχ", "δηλ", "βλ", "e.g", "i.e", "vs", "cf"];

/// True when `before` (the text up to a full stop) ends in one of the
/// abbreviations above, standing as a word of its own.
fn ends_in_leading_abbreviation(before: &str) -> bool {
    let lower = before.to_lowercase();
    LEADING_ABBREVIATIONS.iter().any(|a| {
        lower.strip_suffix(a).is_some_and(|head| !head.chars().next_back().is_some_and(|c| c.is_alphanumeric() || c == '.'))
    })
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
            if c == '.' && ends_in_leading_abbreviation(&out) {
                // "π.χ. ένα κουμπί": the full stop belongs to the abbreviation
                // (3 of 3 such dictations got a capital, 1 October 2026).
                capitalize_next = false;
            } else if matches!(chars.peek(), Some(' ') | None) && c != ';' {
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

/// Greek articles, prepositions, conjunctions and particles. None of them is
/// ever a name, so inside a sentence they are always written in lower case.
/// One-letter words (ο, η) are left out: a capital one can be a label.
const INNER_LOWER_EL: &[&str] = &[
    "το", "τα", "τον", "την", "τη", "του", "της", "των", "τους", "τις", "οι", "ένα", "ένας", "μια", "μία", "με", "σε", "στο",
    "στον", "στη", "στην", "στα", "στους", "στις", "για", "από", "προς", "και", "κι", "αλλά", "να", "θα", "δεν", "δε", "μην",
    "μη", "που", "πως", "ότι", "αν", "άμα", "όταν", "αυτό", "αυτά", "αυτή", "όμως", "έτσι",
];

/// A long dictation is heard in pieces, and a piece can open with a capital
/// although the sentence goes on ("θα το δούμε αύριο αλλά Για την ώρα ...").
/// A word of the list above with a capital right after a lower-case letter or
/// a comma goes back to lower case (14 of 1142 real dictations, 29 September
/// 2026). After a full stop or a question mark nothing changes.
fn lower_inner_function_words(text: &str) -> String {
    static INNER: Lazy<Regex> = Lazy::new(|| {
        let caps: Vec<String> = INNER_LOWER_EL
            .iter()
            .map(|w| {
                let mut c = w.chars();
                let first: String = c.next().map(|f| f.to_uppercase().collect()).unwrap_or_default();
                fancy_regex::escape(&format!("{first}{}", c.as_str())).into_owned()
            })
            .collect();
        Regex::new(&format!(r"(?<=[\p{{Ll}},]) ({})(?!\p{{L}})", caps.join("|"))).unwrap()
    });
    super::replace_all_or_keep(&INNER, text, |c: &fancy_regex::Captures<'_, str>| format!(" {}", c[1].to_lowercase())).to_string()
}

pub fn looks_greek(text: &str) -> bool {
    let greek = text.chars().filter(|c| ('\u{0370}'..='\u{03FF}').contains(c) || ('\u{1F00}'..='\u{1FFF}').contains(c)).count();
    let latin = text.chars().filter(|c| c.is_ascii_alphabetic()).count();
    greek > latin
}

fn ensure_terminal_punctuation(text: &str, _language: &str) -> String {
    // A dictation that stops on a comma ("…, ίσως,") ended as ",." (10 of
    // 1037 real dictations, 27 September 2026). The comma gives way to the stop.
    let t = text.trim_end().trim_end_matches(',').trim_end();
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

    /// 26 September 2026: "…changelog. Ε, what the fuck" came out as
    /// "…changelog., what the fuck": the filler went, its comma stayed.
    #[test]
    fn a_filler_that_opens_a_sentence_takes_its_comma_along() {
        let r = clean("Διόρθωσέ το changelog. Ε, what the fuck, ρε φίλε", &opts(CleanupIntensity::Normal));
        assert!(!r.text.contains(".,"), "{}", r.text);
        assert!(r.text.contains("changelog. "), "{}", r.text);
        let r = clean("Πάμε. Εμ, τώρα λοιπόν", &opts(CleanupIntensity::Normal));
        assert!(!r.text.contains(".,"), "{}", r.text);
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

    /// "ό,τι" is one Greek word written with a comma. The space-after-comma rule
    /// split it into "ό, τι" in 19 of 19 dictations that held it, 13 to 23
    /// September 2026.
    #[test]
    fn the_greek_word_o_ti_keeps_its_comma_closed() {
        let r = clean("κάνε ό,τι θες, Ό,τι πεις", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "Κάνε ό,τι θες, Ό,τι πεις.");
        let r = clean("ό,τι να είναι", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "Ό,τι να είναι.");
    }

    /// Whisper learned "Υπότιτλοι AUTHORWAVE" from subtitled videos and writes
    /// it after a pause. Alone it was already treated as silence; tacked onto a
    /// real dictation it went into the text (7, 12 and 18 September 2026).
    #[test]
    fn a_subtitle_credit_after_real_words_is_dropped() {
        let r = clean("να ετοιμάσουμε; Υπότιτλοι AUTHORWAVE", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "Να ετοιμάσουμε;");
        let r = clean("θα το θέσω ως παράδειγμα.\n Υπότιτλοι AUTHORWAVE\n Υπότιτλο", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "Θα το θέσω ως παράδειγμα.");
        let r = clean("οι υπότιτλοι της ταινίας ήταν καλοί", &opts(CleanupIntensity::Normal));
        assert_eq!(r.text, "Οι υπότιτλοι της ταινίας ήταν καλοί.");
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

    /// 27 September 2026: "όταν ανοίγεις... παλιά αρχεία" came out as
    /// "Όταν ανοίγεις. Παλιά αρχεία", a sentence break nobody said.
    #[test]
    fn a_pause_inside_a_sentence_is_not_a_full_stop() {
        let o = opts(CleanupIntensity::Normal);
        assert_eq!(clean("όταν ανοίγεις... παλιά αρχεία, κράτα αντίγραφο", &o).text, "Όταν ανοίγεις παλιά αρχεία, κράτα αντίγραφο.");
        assert_eq!(clean("πήγα στο... ... στο γραφείο", &o).text, "Πήγα στο στο γραφείο.");
        assert_eq!(clean("I was… thinking about it", &o).text, "I was thinking about it.");
        // before a capital it still ends the sentence, as it did
        assert_eq!(clean("Περίμενε... Τώρα πάμε", &o).text, "Περίμενε. Τώρα πάμε.");
    }

    /// 27 September 2026: a piece of a long dictation that starts inside a
    /// sentence opens with "..." glued to its first word, and "στο... ...σπίτι"
    /// came out as "στο.σπίτι".
    #[test]
    fn a_piece_that_opens_inside_a_sentence_joins_without_a_stop() {
        let o = opts(CleanupIntensity::Normal);
        assert_eq!(clean("θα το στείλω στο... ...γραφείο αύριο το πρωί", &o).text, "Θα το στείλω στο γραφείο αύριο το πρωί.");
        assert_eq!(clean("έλεγξε τα αρχεία, ...τα οποία άλλαξαν χθες", &o).text, "Έλεγξε τα αρχεία, τα οποία άλλαξαν χθες.");
        assert_eq!(clean("we moved it to the… …shared folder", &o).text, "We moved it to the shared folder.");
        // at the very start, also with the filler rules off
        let mut plain = opts(CleanupIntensity::Normal);
        plain.remove_fillers = false;
        assert_eq!(clean("...και μετά κλείνουμε", &plain).text, "Και μετά κλείνουμε.");
        // before a capital it still ends the sentence, as it did
        assert_eq!(clean("Περίμενε... ...Τώρα πάμε", &o).text, "Περίμενε. Τώρα πάμε.");
    }

    /// 27 September 2026: Whisper wrote "μόνο σου; να το ελέγχεις" for a pause
    /// and the cleanup made it "μόνο σου; Να το ελέγχεις".
    #[test]
    fn the_engines_pause_semicolon_inside_a_greek_sentence() {
        let o = opts(CleanupIntensity::Normal);
        assert_eq!(clean("θέλω να το στέλνεις μόνο σου; να το ελέγχεις πρώτα", &o).text, "Θέλω να το στέλνεις μόνο σου, να το ελέγχεις πρώτα.");
        assert_eq!(clean("ψάξε μέσα από το; αρχείο τους", &o).text, "Ψάξε μέσα από το αρχείο τους.");
        // a real question keeps its mark and the next sentence its capital
        assert_eq!(clean("δεν ξέρω τι λες, για ποιο αρχείο μιλάς; πες μου", &o).text, "Δεν ξέρω τι λες, για ποιο αρχείο μιλάς; Πες μου.");
        assert_eq!(clean("μήπως φταίει το δίκτυο; και μετά ξαναδοκίμασε", &o).text, "Μήπως φταίει το δίκτυο; Και μετά ξαναδοκίμασε.");
        // "γιατί" after a comma is "because", no question there
        assert_eq!(clean("έμεινα μέσα, γιατί έβρεχε; και μετά βγήκα", &o).text, "Έμεινα μέσα, γιατί έβρεχε, και μετά βγήκα.");
        // English keeps its semicolons
        assert_eq!(clean("it works; we ship it", &o).text, "It works; we ship it.");
    }

    #[test]
    fn a_dictation_that_stops_on_a_comma_ends_with_one_stop() {
        let o = opts(CleanupIntensity::Normal);
        assert_eq!(clean("θα το δούμε αύριο, ίσως,", &o).text, "Θα το δούμε αύριο, ίσως.");
        assert_eq!(clean("we check it, then,", &o).text, "We check it, then.");
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
    fn a_function_word_inside_a_sentence_loses_its_capital() {
        let o = opts(CleanupIntensity::Normal);
        assert_eq!(clean("Θα το στείλω σήμερα αλλά Για την ώρα περιμένουμε.", &o).text, "Θα το στείλω σήμερα αλλά για την ώρα περιμένουμε.");
        assert_eq!(clean("Φέραμε τα ποτήρια, Τα πιάτα και το ψωμί.", &o).text, "Φέραμε τα ποτήρια, τα πιάτα και το ψωμί.");
        assert_eq!(clean("Άνοιξε το report Και πες μου τι βλέπεις.", &o).text, "Άνοιξε το report και πες μου τι βλέπεις.");
        // names, sentence starts and one-letter words keep their capital
        for text in ["Πήγαμε στη Θεσσαλονίκη με τον Νίκο.", "Τελείωσε. Και μετά φύγαμε.", "Το σχέδιο τύπου Α δουλεύει.", "Έφτασε; Για πόσο θα μείνει;"] {
            assert_eq!(clean(text, &o).text, text);
        }
    }

    /// 1 October 2026: "..., π.χ. ένα κουμπί" came out as "..., π.χ. Ένα κουμπί".
    #[test]
    fn the_word_after_an_abbreviation_that_announces_it_keeps_its_small_letter() {
        let o = opts(CleanupIntensity::Normal);
        assert_eq!(clean("Θέλω κάτι απλό, π.χ. ένα κουμπί στη μέση.", &o).text, "Θέλω κάτι απλό, π.χ. ένα κουμπί στη μέση.");
        assert_eq!(clean("Βάλε μια σκούρα απόχρωση, δηλ. κάτι κοντά στο μπλε.", &o).text, "Βάλε μια σκούρα απόχρωση, δηλ. κάτι κοντά στο μπλε.");
        assert_eq!(clean("We need a fallback, e.g. a cached copy of the page.", &o).text, "We need a fallback, e.g. a cached copy of the page.");
        // The abbreviation itself still opens a sentence with a capital.
        assert_eq!(clean("π.χ. αυτό το κουμπί δεν φαίνεται.", &o).text, "Π.χ. αυτό το κουμπί δεν φαίνεται.");
        // A name after it keeps the capital it came with.
        assert_eq!(clean("Ρώτα κάποιον, π.χ. τον Νίκο.", &o).text, "Ρώτα κάποιον, π.χ. τον Νίκο.");
    }

    #[test]
    fn a_full_stop_that_only_looks_like_such_an_abbreviation_still_ends_the_sentence() {
        let o = opts(CleanupIntensity::Normal);
        // "devs" ends in "vs", "κλπ." closes a list and here the sentence too.
        assert_eq!(clean("I spoke with the devs. they will ship on Monday.", &o).text, "I spoke with the devs. They will ship on Monday.");
        assert_eq!(clean("Πήραμε καρέκλες, τραπέζια κλπ. μετά βάψαμε τον τοίχο.", &o).text, "Πήραμε καρέκλες, τραπέζια κλπ. Μετά βάψαμε τον τοίχο.");
    }

    /// 2 October 2026: a domain ending said on its own ("σε .gr") lost the
    /// space in front of it and stuck to the word before ("σε.gr").
    #[test]
    fn a_domain_ending_said_on_its_own_keeps_the_space_before_it() {
        let o = opts(CleanupIntensity::Normal);
        assert_eq!(clean("θέλω ένα όνομα σε .gr ή σε .io για το μαγαζί", &o).text, "Θέλω ένα όνομα σε .gr ή σε .io για το μαγαζί.");
        assert_eq!(clean("βρήκα δύο ελεύθερα σε .net, σε .org. Μετά τα κλείνουμε", &o).text, "Βρήκα δύο ελεύθερα σε .net, σε .org. Μετά τα κλείνουμε.");
        assert_eq!(clean("we could take the name on .dev or .app", &o).text, "We could take the name on .dev or .app.");
        // a full stop with a space in front still loses that space
        assert_eq!(clean("έφτασε το πακέτο . μετά φύγαμε", &o).text, "Έφτασε το πακέτο. Μετά φύγαμε.");
        assert_eq!(clean("the build is done .", &o).text, "The build is done.");
        assert_eq!(clean("έκλεισε .Το άλλο μένει", &o).text, "Έκλεισε. Το άλλο μένει.");
        // a domain written in one piece stays in one piece
        assert_eq!(clean("άνοιξε το example.com τώρα", &o).text, "Άνοιξε το example.com τώρα.");
    }

    /// Replays real dictations kept outside the repository. FYF_REPLAY_IN holds
    /// one raw transcript per line as a JSON string; the cleaned text of each
    /// goes to FYF_REPLAY_OUT in the same order. Run it on two commits and diff
    /// the outputs to see every text a cleanup change would touch.
    #[test]
    #[ignore]
    fn replay_history() {
        let (Ok(input), Ok(output)) = (std::env::var("FYF_REPLAY_IN"), std::env::var("FYF_REPLAY_OUT")) else { return };
        let o = opts(CleanupIntensity::Normal);
        let cleaned: Vec<String> = std::fs::read_to_string(input)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<String>(line).unwrap())
            .map(|raw| serde_json::to_string(&clean(&raw, &o).text).unwrap())
            .collect();
        std::fs::write(output, cleaned.join("\n")).unwrap();
    }

    #[test]
    fn off_is_identity_except_trim() {
        let r = clean("  εε γεια  ", &DetOptions { intensity: CleanupIntensity::Off, remove_fillers: false, resolve_self_corrections: false, auto_punctuate: false, auto_capitalize: false, language: "el".into() });
        assert_eq!(r.text, "εε γεια");
    }
}
