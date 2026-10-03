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
/// G10 matrices: a first- or third-person or negated verb of knowing or
/// asking. The question after one of them is reported, and the sentence ends
/// with a full stop (docs/QUESTION-RULES.md, section 1.8).
const REPORTING_EL: &[&[&str]] = &[
    &["δεν", "ξέρω"], &["δε", "ξέρω"], &["ξέρω"], &["δεν", "θυμάμαι"], &["δε", "θυμάμαι"], &["αναρωτιέμαι"], &["απορώ"],
    &["μου", "είπε"], &["με", "ρώτησε"], &["είναι", "ζήτημα"], &["εξαρτάται"], &["δεν", "έχει", "σημασία"], &["δε", "έχει", "σημασία"],
];
const TAGS_EL: &[&[&str]] = &[
    &["έτσι", "δεν", "είναι"], &["δεν", "είναι", "έτσι"], &["ή", "όχι"], &["δεν", "νομίζεις"], &["δε", "νομίζεις"],
    &["δεν", "συμφωνείς"], &["δε", "συμφωνείς"],
];
/// Exclamative collocations: sentence-initial τι or πόσο followed by these is
/// an exclamation ("Τι ωραία!"), never a question.
const EXCL_TI_EL: &[&str] = &["ωραία", "ωραίο", "ωραίος", "όμορφα", "καλά", "κρίμα", "υπέροχα", "φοβερό", "χαρά", "βλακεία", "ντροπή", "θαυμάσια", "κι", "και"];
const EXCL_POSO_EL: &[&str] = &["σε", "χαίρομαι", "μου", "ωραία", "όμορφα", "καλά", "πολύ", "λυπάμαι", "μάλλον"];
/// G19: sentence-initial γιατί followed by one of these means "because"
/// ("Γιατί άμα το δεις, θα καταλάβεις."). Only words that never open a
/// real "why" question: clause openers, a first- or third-person opinion,
/// and sentence adverbs of certainty or attitude ("Γιατί 100% θα το βρεις.").
const BECAUSE_NEXT_EL: &[&str] = &[
    "άμα", "αν", "εάν", "όταν", "όσο", "επειδή", "αλλιώς", "νομίζω", "θεωρώ", "νιώθω", "πιστεύω", "απλά", "απλώς", "πρακτικά",
    "βασικά", "προφανώς", "νόμιζα", "νόμιζε", "νομίζει", "νομίζουμε", "πίστευα", "πιστεύει", "πιστεύουμε", "θεωρεί", "θεωρούμε",
    "σίγουρα", "100%", "δυστυχώς", "ευτυχώς", "όντως", "ειλικρινά", "ξεκάθαρα", "μάλλον", "πιθανότατα", "πιθανώς", "λογικά",
    "ουσιαστικά", "οποιοδήποτε", "οποιαδήποτε", "οποιοσδήποτε", "οποιονδήποτε",
    // 3 October 2026, from Lu's history: every "Γιατί" sentence opening with
    // one of these was "because" ("Γιατί θα είναι έτοιμο αύριο."), while the
    // real questions opened with other verbs, να or a number.
    "θα", "και", "κι", "είμαι", "είμαστε", "θέλω", "είδα", "δες", "κοίτα", "άκου",
];
/// Answer words that, in front of γιατί, make it "because": "Όχι, γιατί θέλω
/// να μείνει τοπικό." gives the reason for the answer.
const ANSWER_BEFORE_BECAUSE_EL: &[&str] = &["όχι", "ναι"];
/// Words that open a piece which carries on the sentence before it. A voice
/// that rose before such a piece paused in the middle of a sentence.
const CONTINUES_EL: &[&str] = &[
    "και", "κι", "αλλά", "όμως", "ή", "που", "ότι", "πως", "να", "γιατί", "επειδή", "οπότε", "άρα", "αν", "άμα", "όταν", "ώστε",
    "ενώ", "μέχρι", "δηλαδή", "για", "σε", "στο", "στον", "στη", "στην", "στα", "στους", "στις", "με", "από", "χωρίς",
];
const CONTINUES_EN: &[&str] = &["and", "but", "or", "so", "because", "that", "which", "who", "then", "to", "with", "for", "of", "if", "when"];
/// Words a sentence cannot stop on: a pause mark after one of them simply goes
/// ("από το; αρχείο" -> "από το αρχείο").
const JOINERS_EL: &[&str] = &[
    "ο", "η", "το", "οι", "τα", "τον", "την", "τη", "του", "της", "των", "τους", "τις", "ένα", "ένας", "μια", "μία", "ενός",
    "μιας", "σε", "στο", "στον", "στη", "στην", "στα", "στους", "στις", "στου", "στης", "με", "για", "από", "προς", "ως",
    "χωρίς", "οποίο", "οποία", "οποίος", "οποίοι", "οποίες", "οποίου", "οποίας", "οποίων", "να", "θα", "ότι", "που", "πως",
    "και",
];
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

/// Whisper also writes `;` for a pause inside a Greek sentence, with the next
/// word in lower case ("το στέλνεις; να το ελέγχεις"). Capitalizing after it made a
/// false question and a false new sentence (65 of 1037 real dictations, 27
/// September 2026). Such a `;` stays only when the words before it make a
/// question; otherwise it becomes a comma, or goes after a word a sentence
/// cannot stop on. See docs/QUESTION-RULES.md, section 5.
pub fn soften_inner_semicolons(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut sentence = String::new();
    for (i, &c) in chars.iter().enumerate() {
        let ends = matches!(c, '.' | '!' | '?' | ';') && (i + 1 == chars.len() || chars[i + 1].is_whitespace());
        if c == ';' && ends {
            let next_letter = chars[i + 1..].iter().find(|ch| !ch.is_whitespace());
            if next_letter.map(|ch| ch.is_lowercase()).unwrap_or(false) && !is_question_so_far(&sentence) {
                let last = words(&sentence).pop().unwrap_or_default();
                if !JOINERS_EL.contains(&last.as_str()) {
                    sentence.push(',');
                }
                continue;
            }
        }
        sentence.push(c);
        if ends {
            out.push_str(&sentence);
            sentence.clear();
        }
    }
    out.push_str(&sentence);
    out
}

/// G10 for the voice: true when the last sentence of `text`, or the part of it
/// after its last comma, opens with a reporting verb ("δεν ξέρω", "αναρωτιέμαι")
/// and a question word or αν follows within four words ("Δεν ξέρω ακόμα πού
/// θα πάμε."). A rising voice at the end of such a sentence does not make it a
/// question. Without a question word after it ("Δεν ξέρω;") nothing is decided
/// here.
pub fn reported_question(text: &str) -> bool {
    let body = text.trim().trim_end_matches(|c: char| matches!(c, '.' | '!' | '?' | ';' | '…'));
    let chars: Vec<(usize, char)> = body.char_indices().collect();
    let start = chars
        .windows(2)
        .filter(|w| matches!(w[0].1, '.' | '!' | '?' | ';') && w[1].1.is_whitespace())
        .map(|w| w[1].0)
        .last()
        .unwrap_or(0);
    let sentence = &body[start..];
    let clause = sentence.rfind(',').map(|at| &sentence[at + 1..]).unwrap_or(sentence);
    let all = words(clause);
    let mut skip = 0;
    while skip < 2 && skip + 1 < all.len() && OPENERS_EL.contains(&all[skip].as_str()) {
        skip += 1;
    }
    let w = &all[skip..];
    REPORTING_EL
        .iter()
        .any(|p| starts_with_phrase(w, p) && w.iter().skip(p.len()).take(4).any(|t| t == "αν" || WH_EL.contains(&t.as_str())))
}

/// True when `next`, the piece heard after a pause, carries on the sentence
/// the piece before it was in: it opens with a lower-case letter, with a
/// joining word ("και", "που", "για"), or with a relative ("το οποίο"). A voice
/// that rose before it paused mid-sentence, and that rise asks nothing: 7 of
/// the 9 false question marks of 1 October sat at such a pause.
pub fn continues_sentence(next: &str) -> bool {
    let t = next.trim_start();
    if t.chars().next().is_some_and(|c| c.is_lowercase()) {
        return true;
    }
    let w = words(t);
    let Some(first) = w.first() else { return false };
    CONTINUES_EL.contains(&first.as_str())
        || CONTINUES_EN.contains(&first.as_str())
        || w.get(1).is_some_and(|second| second.starts_with("οποί"))
}

/// True when `text` stops on a word no sentence can end on ("τα οποία", "στο"):
/// the speaker paused mid-phrase, so a rising voice there asks nothing.
pub fn ends_on_joiner(text: &str) -> bool {
    let body = text.trim().trim_end_matches(|c: char| matches!(c, '.' | '!' | '?' | ';' | '…' | ','));
    words(body).last().is_some_and(|last| JOINERS_EL.contains(&last.as_str()) || last.starts_with("οποί"))
}

/// G20 for the voice: true when the last sentence of `text` corrects itself
/// ("Δεν είναι η Τρίτη, είναι η Τετάρτη."): the part after its last comma
/// opens, after at most two openers, with the same word that follows δεν or δε
/// in the part just before it. The voice rises on the corrected word for
/// emphasis, and the sentence is a statement. Found on 29 September 2026: the
/// only two sentences of this shape among 145 marks the voice had added in
/// six days were both statements.
pub fn corrective_contrast(text: &str) -> bool {
    let body = text.trim().trim_end_matches(|c: char| matches!(c, '.' | '!' | '?' | ';' | '…'));
    let chars: Vec<(usize, char)> = body.char_indices().collect();
    let start = chars
        .windows(2)
        .filter(|w| matches!(w[0].1, '.' | '!' | '?' | ';') && w[1].1.is_whitespace())
        .map(|w| w[1].0)
        .last()
        .unwrap_or(0);
    let parts: Vec<&str> = body[start..].split(',').collect();
    if parts.len() < 2 {
        return false;
    }
    let last = words(parts[parts.len() - 1]);
    let before = words(parts[parts.len() - 2]);
    let mut skip = 0;
    while skip < 2 && skip + 1 < last.len() && OPENERS_EL.contains(&last[skip].as_str()) {
        skip += 1;
    }
    let Some(verb) = last.get(skip) else { return false };
    before.windows(2).any(|w| (w[0] == "δεν" || w[0] == "δε") && &w[1] == verb)
}

/// A question by its words: the whole sentence so far, or the part after its
/// last comma ("Δεν ξέρω τι λες, για ποιο αρχείο μιλάς"). A part that opens
/// with γιατί after a comma is "because" (G17), so it does not count.
fn is_question_so_far(sentence: &str) -> bool {
    let tail = after_last_comma(sentence);
    greek_question(&words(sentence), false) || (tail.first().map(|w| w != "γιατί").unwrap_or(false) && greek_question(&tail, false))
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
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'' && c != '’' && c != '%').to_lowercase().replace('’', "'"))
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
        // G19: "Γιατί άμα ..." / "Γιατί νομίζω ..." is "because"
        if wh == "γιατί" && at == 0 && next.map(|n| BECAUSE_NEXT_EL.contains(&n)).unwrap_or(false) {
            return false;
        }
        // "Γιατί δεν ξέρω ..." gives a reason; "Γιατί δεν έρχεσαι;" stays a question.
        if wh == "γιατί" && at == 0 && starts_with_phrase(&w[1..], &["δεν", "ξέρω"]) {
            return false;
        }
        // "Όχι, γιατί θέλω ..." answers with a reason. "Ναι, γιατί;" alone is a question.
        if wh == "γιατί" && at == 0 && skip > 0 && ANSWER_BEFORE_BECAUSE_EL.contains(&all[skip - 1].as_str()) && w.len() > 1 {
            return false;
        }
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

/// The words after the last comma of the sentence, the only place a tag
/// question can sit. Empty when the sentence has no comma.
fn after_last_comma(sentence: &str) -> Vec<String> {
    match sentence.rfind(',') {
        Some(at) => words(&sentence[at + 1..]),
        None => Vec::new(),
    }
}

fn english_question(all: &[String], tail: &[String], ends_with_bang: bool) -> bool {
    if all.is_empty() {
        return false;
    }
    // E4a: tag at the end, set off by a comma ("You did it, right", "You do
    // care, don't you"). The comma is what makes it a tag: `words` drops
    // punctuation, and looking at the last words alone turned plain statements
    // into questions ("You are right?", "This is it?", "The problem was that?").
    if all.len() >= 3 {
        let right_tag = tail.len() == 1 && tail[0] == "right";
        let aux_tag = tail.len() == 2 && AUX_EN.contains(&tail[0].as_str()) && SUBJ_EN.contains(&tail[1].as_str());
        if right_tag || aux_tag {
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
    let is_q = if greek { greek_question(&all, bang) } else { english_question(&all, &after_last_comma(trimmed), bang) };
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

    /// 27 September 2026: "Γιατί άμα το δεις από κοντά, ..." got a
    /// question mark. γιατί opening a sentence is often "because".
    #[test]
    fn greek_because_is_not_why() {
        for s in [
            "Γιατί άμα το δεις από κοντά, θα καταλάβεις.", "Γιατί νομίζω ότι αργεί πολύ.", "Γιατί αλλιώς θα χαθεί η σειρά.",
            "Και γιατί όταν τρέχει, κολλάει όλο το σύστημα.", "Γιατί απλά δεν υπάρχει χρόνος.",
        ] {
            assert_eq!(q(s), s, "{s}");
        }
        assert_eq!(q("Γιατί δεν ήρθες χθες."), "Γιατί δεν ήρθες χθες;");
        assert_eq!(q("Γιατί να το κάνω τώρα."), "Γιατί να το κάνω τώρα;");
        assert_eq!(q("Γιατί το έστειλες έτσι."), "Γιατί το έστειλες έτσι;");
    }

    /// 28 September 2026: "Γιατί 100% θα ..." got a question mark. A word of
    /// certainty, a past or third-person opinion, or "any" after γιατί is
    /// 3 October 2026: the openings that made "because" sentences questions
    /// in Lu's history, and the real questions that must stay.
    #[test]
    fn greek_because_after_an_answer_or_with_a_reason_word() {
        assert_eq!(q("Όχι, γιατί θέλουμε να μείνει τοπικό."), "Όχι, γιατί θέλουμε να μείνει τοπικό.");
        assert_eq!(q("Γιατί θα είναι έτοιμο αύριο."), "Γιατί θα είναι έτοιμο αύριο.");
        assert_eq!(q("Γιατί και αυτό είναι προτεραιότητα."), "Γιατί και αυτό είναι προτεραιότητα.");
        assert_eq!(q("Γιατί δεν ξέρω αν θα προλάβω."), "Γιατί δεν ξέρω αν θα προλάβω.");
        assert_eq!(q("Γιατί είμαι κουρασμένος σήμερα."), "Γιατί είμαι κουρασμένος σήμερα.");
        // still questions
        assert_eq!(q("Ναι, γιατί."), "Ναι, γιατί;");
        assert_eq!(q("Γιατί δεν έρχεσαι μαζί μας."), "Γιατί δεν έρχεσαι μαζί μας;");
        assert_eq!(q("Γιατί να το κάνουμε τώρα."), "Γιατί να το κάνουμε τώρα;");
    }

    #[test]
    fn a_piece_that_carries_on_the_sentence() {
        assert!(continues_sentence("και μετά αποφασίζουμε."));
        assert!(continues_sentence("Και μετά αποφασίζουμε."));
        assert!(continues_sentence("Το οποίο ανοίγει τη λίστα."));
        assert!(continues_sentence("Για το δεύτερο κουμπί."));
        assert!(continues_sentence("and then we decide."));
        assert!(!continues_sentence("Θα το δούμε αύριο."));
        assert!(!continues_sentence("Μετά ας πούμε για το άλλο."));
        assert!(!continues_sentence(""));
        assert!(ends_on_joiner("Τους δίνουμε συστήματα τα οποία."));
        assert!(ends_on_joiner("Πάμε στο"));
        assert!(!ends_on_joiner("Θέλω ένα κουμπί εδώ."));
    }

    /// "because" too. A plain number stays a question.
    #[test]
    fn greek_because_with_certainty_is_not_why() {
        for s in [
            "Γιατί 100% θα βρεις κι άλλα.", "Γιατί σίγουρα θα αργήσει λίγο.", "Γιατί δυστυχώς έκλεισε νωρίς.",
            "Γιατί μάλλον θα βρέξει αύριο.", "Γιατί οποιοδήποτε άλλο θα κάνει το ίδιο.", "Γιατί νόμιζα ότι το είχες στείλει.",
        ] {
            assert_eq!(q(s), s, "{s}");
        }
        assert_eq!(q("Γιατί 100 ευρώ για αυτό."), "Γιατί 100 ευρώ για αυτό;");
        assert_eq!(q("Γιατί νόμιζες ότι έφυγα."), "Γιατί νόμιζες ότι έφυγα;");
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

    /// A tag question needs its comma. Without one the last words are just the
    /// end of a statement, and a question mark there lands in the user's text.
    #[test]
    fn english_tags_need_their_comma() {
        for s in [
            "You are right.", "That is right.", "Turn right.", "This is it.", "The problem was that.",
            "I think that is it.", "Well, you are right.", "Yes, this is it.", "Right, so that was it.",
        ] {
            assert_eq!(q(s), s, "{s}");
        }
        assert_eq!(q("You did it, right."), "You did it, right?");
        assert_eq!(q("You do care, don't you."), "You do care, don't you?");
        assert_eq!(q("It was late, wasn't it."), "It was late, wasn't it?");
        assert_eq!(q("Well, she is here, isn't she."), "Well, she is here, isn't she?");
    }

    #[test]
    fn reported_questions_are_found_in_the_last_sentence_or_after_the_last_comma() {
        assert!(reported_question("Δεν ξέρω ακόμα πού θα πάμε."));
        assert!(reported_question("Το έστειλα χθες. Δεν ξέρω αν το είδε."));
        assert!(reported_question("Καλά, αναρωτιέμαι γιατί άργησε τόσο."));
        assert!(reported_question("Μου είπε πότε ανοίγει το γραφείο."));
        assert!(!reported_question("Δεν ξέρω."));
        assert!(!reported_question("Δεν ξέρω, πού πάμε μετά;"));
        assert!(!reported_question("Δεν ξέρω αν έρχεται. Εσύ θα έρθεις."));
        assert!(!reported_question("Ξέρεις πού είναι;"));
        assert!(!reported_question("I don't know where it is."));
    }

    #[test]
    fn a_sentence_that_corrects_itself_is_found_by_the_repeated_word() {
        assert!(corrective_contrast("Δεν είναι η Τρίτη που είπαμε, είναι η Τετάρτη το πρωί."));
        assert!(corrective_contrast("Λοιπόν, δεν θέλω το μπλε πουκάμισο, θέλω το άσπρο με τις ρίγες."));
        assert!(corrective_contrast("Το είπα ήδη. Δε μένει στην Αθήνα, αλλά μένει στη Λάρισα με τους γονείς του."));
        // different words, one part only, or the negation in an earlier sentence
        assert!(!corrective_contrast("Δεν σου αρέσει το φαγητό εδώ, θέλεις να πάμε κάπου αλλού."));
        assert!(!corrective_contrast("Δεν είναι η Τρίτη που είπαμε."));
        assert!(!corrective_contrast("Δεν είναι η Τρίτη. Μετά, είναι η Τετάρτη το πρωί."));
        assert!(!corrective_contrast("Είναι η Τρίτη, είναι η Τετάρτη."));
    }
}
