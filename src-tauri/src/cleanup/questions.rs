//! Question marks from wording alone. Speech recognition hears words and
//! never intonation, so a spoken question usually arrives with a period. The
//! rules here are the high-confidence rows of docs/QUESTION-RULES.md: a
//! sentence gets `;` (Greek) or `?` (English) only when its words make it a
//! question; yes/no questions without any lexical cue are an accepted miss.

use super::deterministic::looks_greek;

// ---------- Greek ----------

const WH_EL: &[&str] = &[
    "ποιος", "ποια", "ποιο", "ποιοι", "ποιες", "ποιον", "ποιαν", "ποιου", "ποιας", "ποιων", "ποιους", "ποιανού", "ποιανής",
    "ποιανών", "ποιανούς", "τίνος", "τίνων", "πόσος", "πόση", "πόσο", "πόσου", "πόσης", "πόσοι", "πόσες", "πόσα", "πόσων",
    "πόσους", "τι", "πού", "πώς", "πότε", "γιατί",
];
const PREP_EL: &[&str] = &["σε", "με", "για", "από", "μέχρι", "ως", "έως", "προς", "κατά", "χωρίς", "μετά", "πριν", "εναντίον"];
/// Vocatives and discourse openers that may precede the question word.
const OPENERS_EL: &[&str] = &["λοιπόν", "και", "αλλά", "ε", "ρε", "βρε", "μα", "τελικά", "εντάξει", "ναι", "όχι", "οκ", "οκέι", "τώρα", "άρα"];
/// Second-person matrix verbs that make an indirect question a real one.
const MATRIX_2P_EL: &[&str] = &["ξέρεις", "ξέρετε", "θυμάσαι", "θυμάστε", "καταλαβαίνεις", "κατάλαβες", "βλέπεις", "είδες", "άκουσες", "έμαθες"];
const MATRIX_2P_PHRASES_EL: &[&[&str]] = &[
    &["μου", "λες"], &["μου", "λέτε"], &["μπορείς", "να", "μου", "πεις"], &["μπορείτε", "να", "μου", "πείτε"],
    &["θέλεις", "να", "μάθεις"], &["έχεις", "ιδέα"], &["έχεις", "καταλάβει"],
];
const TAGS_EL: &[&[&str]] = &[
    &["έτσι", "δεν", "είναι"], &["δεν", "είναι", "έτσι"], &["ή", "όχι"], &["δεν", "νομίζεις"], &["δε", "νομίζεις"],
    &["δεν", "συμφωνείς"], &["δε", "συμφωνείς"],
];
/// Exclamative collocations: sentence-initial τι or πόσο followed by these is
/// an exclamation ("Τι ωραία!"), never a question.
const EXCL_TI_EL: &[&str] = &["ωραία", "ωραίο", "ωραίος", "όμορφα", "καλά", "κρίμα", "υπέροχα", "φοβερό", "χαρά", "βλακεία", "ντροπή", "θαυμάσια", "κι", "και"];
const EXCL_POSO_EL: &[&str] = &["σε", "χαίρομαι", "μου", "ωραία", "όμορφα", "καλά", "πολύ", "λυπάμαι", "μάλλον"];
const IDIOMS_EL: &[&[&str]] = &[
    &["πού", "και", "πού"], &["πώς", "και", "πώς"], &["πού", "να", "ξέρω"], &["πού", "να", "το", "ξέρω"], &["πού", "να", "ξέρεις"],
    &["πού", "να", "σου", "τα", "λέω"], &["πού", "να", "σας", "τα", "λέω"], &["πώς", "όχι"],
];

// ---------- English ----------

const WH_EN: &[&str] = &["who", "whom", "whose", "what", "which", "when", "where", "why", "how"];
const PREP_EN: &[&str] = &["to", "in", "for", "at", "by", "from", "with", "about", "on"];
const AUX_EN: &[&str] = &[
    "do", "does", "did", "am", "is", "are", "was", "were", "have", "has", "had", "can", "could", "will", "would", "shall",
    "should", "may", "might", "must", "isn't", "aren't", "wasn't", "weren't", "don't", "doesn't", "didn't", "can't",
    "couldn't", "won't", "wouldn't", "shouldn't", "haven't", "hasn't", "hadn't",
];
const SUBJ_EN: &[&str] = &["i", "you", "he", "she", "it", "we", "they", "there", "this", "that"];
const OPENERS_EN: &[&str] = &["so", "and", "but", "okay", "ok", "well", "now", "then", "also"];
const BLOCK_FRAMES_EN: &[&[&str]] = &[
    &["i", "wonder"], &["i'm", "not", "sure"], &["i", "don't", "know"], &["i", "know"], &["nobody", "knows"], &["it", "depends"],
    &["that's", "why"], &["this", "is", "how"], &["here", "is", "what"], &["no", "matter"], &["whatever"], &["whoever"],
    &["whenever"], &["wherever"], &["however"], &["tell", "me"], &["let", "me", "know"], &["ask"], &["asked"], &["explain"],
    &["show", "me"], &["remember", "to"], &["find", "out"], &["figure", "out"],
];

/// Spoken punctuation for the cases wording cannot decide: "ερωτηματικό" or
/// "question mark" at the end of a sentence becomes the mark, and the word
/// disappears. Same for "θαυμαστικό" / "exclamation mark".
pub fn spoken_marks(text: &str) -> String {
    static SPOKEN: once_cell::sync::Lazy<fancy_regex::Regex> = once_cell::sync::Lazy::new(|| {
        // After an article ("το ερωτηματικό", "a question mark") the word is a
        // noun in a sentence about punctuation, so it stays.
        fancy_regex::Regex::new(r"(?i)[\s,]*(?<!\b(?:το|στο|του|ένα|τα|a|the)\s)\b(ερωτηματικό|ερωτηματικο|question mark|θαυμαστικό|θαυμαστικο|exclamation mark)\b[.!?;]*(?=\s|$)").unwrap()
    });
    let greek = looks_greek(text);
    super::replace_all_or_keep(&SPOKEN, text, |caps: &fancy_regex::Captures<'_, str>| {
        let word = caps.get(1).map(|m| m.as_str().to_lowercase()).unwrap_or_default();
        if word.starts_with("θαυμ") || word.starts_with("exclam") {
            "!".to_string()
        } else if greek || word.starts_with("ερωτ") {
            ";".to_string()
        } else {
            "?".to_string()
        }
    })
    .to_string()
}

/// Adds a question mark to every sentence whose wording makes it a question.
pub fn mark_questions(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 4);
    let mut sentence = String::new();
    let chars: Vec<char> = text.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        sentence.push(c);
        let terminal = matches!(c, '.' | '!' | '?' | ';') && (i + 1 == chars.len() || chars[i + 1].is_whitespace());
        if terminal {
            out.push_str(&question_form(&sentence));
            sentence.clear();
        }
    }
    if !sentence.trim().is_empty() {
        out.push_str(&question_form(&sentence));
    }
    out
}

fn words(sentence: &str) -> Vec<String> {
    sentence
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'' && c != '’').to_lowercase().replace('’', "'"))
        .filter(|w| !w.is_empty())
        .collect()
}

fn starts_with_phrase(words: &[String], phrase: &[&str]) -> bool {
    words.len() >= phrase.len() && words.iter().zip(phrase.iter()).all(|(w, p)| w == p)
}

fn ends_with_phrase(words: &[String], phrase: &[&str]) -> bool {
    words.len() >= phrase.len() && words[words.len() - phrase.len()..].iter().zip(phrase.iter()).all(|(w, p)| w == p)
}

fn greek_question(all: &[String], ends_with_bang: bool) -> bool {
    if all.is_empty() {
        return false;
    }
    // G5: άραγε anywhere
    if all.iter().any(|w| w == "άραγε") {
        return true;
    }
    // G8a: sentence-final multiword tag
    if TAGS_EL.iter().any(|t| ends_with_phrase(all, t) && all.len() > t.len()) {
        return true;
    }
    // vocative / discourse openers before the question word
    let mut skip = 0;
    while skip < 2 && skip + 1 < all.len() && OPENERS_EL.contains(&all[skip].as_str()) {
        skip += 1;
    }
    let w = &all[skip..];
    let Some(first) = w.first() else { return false };
    // G4, G6, G7
    if first == "μήπως" || starts_with_phrase(w, &["μπας", "και"]) || starts_with_phrase(w, &["λες", "να"]) || starts_with_phrase(w, &["λέτε", "να"]) {
        return w.len() >= 2;
    }
    // G15 and idiom blockers
    if starts_with_phrase(w, &["τι", "κι", "αν"]) || starts_with_phrase(w, &["τι", "και", "αν"]) || starts_with_phrase(w, &["πόσο", "μάλλον"]) {
        return false;
    }
    if IDIOMS_EL.iter().any(|p| starts_with_phrase(w, p)) {
        return false;
    }
    // G1, G2, G3: question word first, or after a preposition
    let wh_at = if WH_EL.contains(&first.as_str()) {
        Some(0)
    } else if PREP_EL.contains(&first.as_str()) && w.len() >= 2 && WH_EL.contains(&w[1].as_str()) {
        Some(1)
    } else {
        None
    };
    if let Some(at) = wh_at {
        let wh = w[at].as_str();
        let next = w.get(at + 1).map(|s| s.as_str());
        // G14: exclamatives with τι / πόσο
        if wh == "τι" && (ends_with_bang || next.map(|n| EXCL_TI_EL.contains(&n)).unwrap_or(false)) {
            return false;
        }
        if wh == "πόσο" && (ends_with_bang || next.map(|n| EXCL_POSO_EL.contains(&n)).unwrap_or(false)) {
            return false;
        }
        // "τι λες" alone is a common question; longer "τι λες ..." is ambiguous
        if wh == "τι" && next == Some("λες") && w.len() > 2 {
            return false;
        }
        return true;
    }
    // G9: second-person knowing/asking verb, then a question word or αν within four tokens
    let matrix_len = if MATRIX_2P_EL.contains(&first.as_str()) {
        Some(1)
    } else {
        MATRIX_2P_PHRASES_EL.iter().find(|p| starts_with_phrase(w, p)).map(|p| p.len())
    };
    if let Some(n) = matrix_len {
        return w.iter().skip(n).take(4).any(|t| t == "αν" || WH_EL.contains(&t.as_str()));
    }
    false
}

fn english_question(all: &[String], ends_with_bang: bool) -> bool {
    if all.is_empty() {
        return false;
    }
    // E4a: tag at the end
    if all.len() >= 3 {
        let n = all.len();
        if all[n - 1] == "right" || (AUX_EN.contains(&all[n - 2].as_str()) && SUBJ_EN.contains(&all[n - 1].as_str())) {
            return true;
        }
    }
    let mut skip = 0;
    while skip < 2 && skip + 1 < all.len() && OPENERS_EN.contains(&all[skip].as_str()) {
        skip += 1;
    }
    let w = &all[skip..];
    let Some(first) = w.first() else { return false };
    // E6: embedding frames
    if BLOCK_FRAMES_EN.iter().any(|f| starts_with_phrase(w, f)) {
        return false;
    }
    // E7: exclamatives
    if starts_with_phrase(w, &["what", "a"]) || starts_with_phrase(w, &["what", "an"]) {
        return false;
    }
    if first == "how" && ends_with_bang {
        return false;
    }
    if starts_with_phrase(w, &["any", "idea"]) {
        return true;
    }
    // E1, E2, E8
    let wh_at = if WH_EN.contains(&first.as_str()) {
        Some(0)
    } else if PREP_EN.contains(&first.as_str()) && w.len() >= 2 && WH_EN.contains(&w[1].as_str()) {
        Some(1)
    } else {
        None
    };
    if let Some(at) = wh_at {
        // "how nice" with nothing verbal after it reads as an exclamation
        if w[at] == "how" && w.len() == 2 {
            return false;
        }
        let _ = at;
        return true;
    }
    // E3: auxiliary + pronoun subject
    if AUX_EN.contains(&first.as_str()) && w.len() >= 2 && SUBJ_EN.contains(&w[1].as_str()) {
        return true;
    }
    false
}

fn question_form(sentence: &str) -> String {
    let trimmed = sentence.trim_end();
    let trailing_ws = &sentence[trimmed.len()..];
    let last = trimmed.chars().last();
    // keep whatever mark the engine wrote itself
    if matches!(last, Some(';') | Some('?')) {
        return sentence.to_string();
    }
    let greek = looks_greek(trimmed);
    let all = words(trimmed);
    let bang = last == Some('!');
    let is_q = if greek { greek_question(&all, bang) } else { english_question(&all, bang) };
    if !is_q {
        return sentence.to_string();
    }
    let mark = if greek { ';' } else { '?' };
    let body = trimmed.trim_end_matches(|c: char| matches!(c, '.' | '!' | '?' | ';' | '…'));
    format!("{body}{mark}{trailing_ws}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(s: &str) -> String {
        mark_questions(s)
    }

    #[test]
    fn greek_question_words_get_the_mark() {
        assert_eq!(q("Τι μπορούμε να κάνουμε σε αυτό το κομμάτι."), "Τι μπορούμε να κάνουμε σε αυτό το κομμάτι;");
        assert_eq!(q("Ξέρεις τι παρατηρώ!"), "Ξέρεις τι παρατηρώ;");
        assert_eq!(q("Πώς σου φαίνεται. Πάμε για καφέ."), "Πώς σου φαίνεται; Πάμε για καφέ.");
        assert_eq!(q("Σε ποιον το έδωσες."), "Σε ποιον το έδωσες;");
        assert_eq!(q("Λοιπόν, τι κάνουμε τώρα."), "Λοιπόν, τι κάνουμε τώρα;");
        assert_eq!(q("Γιατί"), "Γιατί;");
        assert_eq!(q("Μήπως είδες το κλειδί."), "Μήπως είδες το κλειδί;");
        assert_eq!(q("Λες να έρθει."), "Λες να έρθει;");
        assert_eq!(q("Γιατί άραγε αργεί τόσο."), "Γιατί άραγε αργεί τόσο;");
        assert_eq!(q("Είναι ωραία εδώ, έτσι δεν είναι."), "Είναι ωραία εδώ, έτσι δεν είναι;");
        assert_eq!(q("Θυμάσαι πού το έβαλες."), "Θυμάσαι πού το έβαλες;");
        assert_eq!(q("Μπορείς να μου πεις πότε φεύγει."), "Μπορείς να μου πεις πότε φεύγει;");
        assert_eq!(q("Ξέρεις αν έρχεται."), "Ξέρεις αν έρχεται;");
        assert_eq!(q("Τι λες."), "Τι λες;");
    }

    #[test]
    fn greek_lookalikes_stay_statements() {
        for s in [
            "Πως θα έρθω το είπα.", "Που να ήξερες.", "Αυτό που σου είπα.", "Είπε πως θα έρθει.",
            "Μου είπε τι ώρα είναι.", "Πες μου πώς το έκανες.", "Δεν ξέρω αν θα έρθει.", "Δεν ήρθα γιατί έβρεχε.",
            "Τι ωραία!", "Πόσο σε αγαπώ!", "Τι κι αν έβρεχε, βγήκαμε.", "Πού και πού πάω.", "Πόσο μάλλον τώρα.",
            "Ποτέ δεν το είπα.", "Όσο περιμένω, διαβάζω.", "Φοβάμαι μήπως βρέξει.", "Ναι.",
        ] {
            assert_eq!(q(s), s, "{s}");
        }
        // the engine's own question mark is kept as is
        assert_eq!(q("Τι κάνεις;"), "Τι κάνεις;");
    }

    #[test]
    fn spoken_marks_replace_the_word() {
        assert_eq!(spoken_marks("Θα πάρει πολύ ώρα ακόμα ερωτηματικό."), "Θα πάρει πολύ ώρα ακόμα;");
        assert_eq!(spoken_marks("Θα πάρει πολύ ώρα ακόμα, ερωτηματικό"), "Θα πάρει πολύ ώρα ακόμα;");
        assert_eq!(spoken_marks("Είσαι σίγουρος ερωτηματικό Πάμε."), "Είσαι σίγουρος; Πάμε.");
        assert_eq!(spoken_marks("Τέλεια θαυμαστικό"), "Τέλεια!");
        assert_eq!(spoken_marks("Are you sure question mark"), "Are you sure?");
        assert_eq!(spoken_marks("Το ερωτηματικό είναι σημείο στίξης."), "Το ερωτηματικό είναι σημείο στίξης.");
        assert_eq!(spoken_marks("Δεν καταλαβαίνει το ερωτηματικό."), "Δεν καταλαβαίνει το ερωτηματικό.");
    }

    #[test]
    fn english_questions_and_lookalikes() {
        assert_eq!(q("What do you think."), "What do you think?");
        assert_eq!(q("Do you have a minute."), "Do you have a minute?");
        assert_eq!(q("Isn't it late."), "Isn't it late?");
        assert_eq!(q("To whom did you send it."), "To whom did you send it?");
        assert_eq!(q("You did it, right."), "You did it, right?");
        assert_eq!(q("You do care, don't you."), "You do care, don't you?");
        assert_eq!(q("Why"), "Why?");
        assert_eq!(q("So, where are we."), "So, where are we?");
        for s in [
            "I think so.", "Do the dishes.", "Have a seat.", "I wonder what time it is.", "Tell me what you want.",
            "What a day!", "How nice!", "That's why I left.", "The man who called.", "Okay.",
        ] {
            assert_eq!(q(s), s, "{s}");
        }
    }
}
