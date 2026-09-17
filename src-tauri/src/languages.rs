//! The languages a user can pick as their own, and the check that keeps the
//! engine from wandering off to a third one.
//!
//! In the mixed mode the engine is told "work out the language yourself", and
//! it chooses among all ninety-nine it knows. A short Greek phrase said quickly
//! came back as Czech on 11 September 2026 ("Do tímhle světu", in the History
//! of the machine this was written on). The user had chosen Greek and English;
//! nothing else was ever on the table. So a result in any other language is
//! treated as a wrong guess and the audio is transcribed again with the user's
//! own language forced.

/// Languages offered as "your language" in Settings. Only the ones the model
/// hears well enough to dictate in are listed; the engine knows more, and
/// auto-detect still reaches all of them. Each entry is the ISO code and the
/// name in that language, which needs no translation.
pub const CHOICES: &[(&str, &str)] = &[
    ("el", "Ελληνικά"),
    ("en", "English"),
    ("de", "Deutsch"),
    ("fr", "Français"),
    ("es", "Español"),
    ("it", "Italiano"),
    ("pt", "Português"),
    ("nl", "Nederlands"),
    ("sv", "Svenska"),
    ("da", "Dansk"),
    ("no", "Norsk"),
    ("fi", "Suomi"),
    ("pl", "Polski"),
    ("cs", "Čeština"),
    ("sk", "Slovenčina"),
    ("hu", "Magyar"),
    ("ro", "Română"),
    ("bg", "Български"),
    ("sr", "Српски"),
    ("hr", "Hrvatski"),
    ("uk", "Українська"),
    ("ru", "Русский"),
    ("tr", "Türkçe"),
    ("ar", "العربية"),
    ("he", "עברית"),
    ("hi", "हिन्दी"),
    ("id", "Bahasa Indonesia"),
    ("ms", "Bahasa Melayu"),
    ("vi", "Tiếng Việt"),
    ("th", "ไทย"),
    ("ja", "日本語"),
    ("ko", "한국어"),
    ("zh", "中文"),
];

/// True when the code is one the Settings list offers.
pub fn is_choice(code: &str) -> bool {
    CHOICES.iter().any(|(c, _)| *c == code)
}

/// whisper.cpp reports full names ("greek", "czech"); the app uses ISO codes.
/// Anything unknown comes back unchanged, lowercased.
pub fn code_from_name(s: &str) -> String {
    let s = s.trim().to_ascii_lowercase();
    let code = match s.as_str() {
        "english" => "en", "chinese" => "zh", "german" => "de", "spanish" => "es", "russian" => "ru",
        "korean" => "ko", "french" => "fr", "japanese" => "ja", "portuguese" => "pt", "turkish" => "tr",
        "polish" => "pl", "catalan" => "ca", "dutch" => "nl", "arabic" => "ar", "swedish" => "sv",
        "italian" => "it", "indonesian" => "id", "hindi" => "hi", "finnish" => "fi", "vietnamese" => "vi",
        "hebrew" => "he", "ukrainian" => "uk", "greek" => "el", "malay" => "ms", "czech" => "cs",
        "romanian" => "ro", "danish" => "da", "hungarian" => "hu", "tamil" => "ta", "norwegian" => "no",
        "thai" => "th", "urdu" => "ur", "croatian" => "hr", "bulgarian" => "bg", "lithuanian" => "lt",
        "latin" => "la", "maori" => "mi", "malayalam" => "ml", "welsh" => "cy", "slovak" => "sk",
        "telugu" => "te", "persian" => "fa", "latvian" => "lv", "bengali" => "bn", "serbian" => "sr",
        "azerbaijani" => "az", "slovenian" => "sl", "kannada" => "kn", "estonian" => "et",
        "macedonian" => "mk", "breton" => "br", "basque" => "eu", "icelandic" => "is", "armenian" => "hy",
        "nepali" => "ne", "mongolian" => "mn", "bosnian" => "bs", "kazakh" => "kk", "albanian" => "sq",
        "swahili" => "sw", "galician" => "gl", "marathi" => "mr", "punjabi" => "pa", "sinhala" => "si",
        "khmer" => "km", "shona" => "sn", "yoruba" => "yo", "somali" => "so", "afrikaans" => "af",
        "occitan" => "oc", "georgian" => "ka", "belarusian" => "be", "tajik" => "tg", "sindhi" => "sd",
        "gujarati" => "gu", "amharic" => "am", "yiddish" => "yi", "lao" => "lo", "uzbek" => "uz",
        "faroese" => "fo", "haitian creole" => "ht", "pashto" => "ps", "turkmen" => "tk", "nynorsk" => "nn",
        "maltese" => "mt", "sanskrit" => "sa", "luxembourgish" => "lb", "myanmar" | "burmese" => "my",
        "tibetan" => "bo", "tagalog" => "tl", "malagasy" => "mg", "assamese" => "as", "tatar" => "tt",
        "hawaiian" => "haw", "lingala" => "ln", "hausa" => "ha", "bashkir" => "ba", "javanese" => "jw",
        "sundanese" => "su", "cantonese" => "yue",
        other => other,
    };
    code.to_string()
}

/// The writing system a language uses, coarse enough to tell "the engine
/// switched alphabets" from "the user said an English word".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Script {
    Latin,
    Greek,
    Cyrillic,
    Arabic,
    Hebrew,
    Devanagari,
    Thai,
    Cjk,
    Hangul,
    Other,
}

fn script_of_language(code: &str) -> Script {
    match code {
        "el" => Script::Greek,
        "ru" | "uk" | "bg" | "sr" | "mk" | "be" | "kk" | "mn" | "tg" | "tt" | "ba" => Script::Cyrillic,
        "ar" | "fa" | "ur" | "ps" | "sd" => Script::Arabic,
        "he" | "yi" => Script::Hebrew,
        "hi" | "mr" | "ne" | "sa" => Script::Devanagari,
        "th" => Script::Thai,
        "ja" | "zh" | "yue" => Script::Cjk,
        "ko" => Script::Hangul,
        // Everything on the Settings list not named above writes in Latin
        // letters. Languages off the list get "Other", which switches the
        // alphabet check off rather than guessing.
        c if is_choice(c) => Script::Latin,
        _ => Script::Other,
    }
}

fn script_of_char(c: char) -> Option<Script> {
    if !c.is_alphabetic() {
        return None;
    }
    let u = c as u32;
    Some(match u {
        0x0000..=0x024F | 0x1E00..=0x1EFF => Script::Latin,
        0x0370..=0x03FF | 0x1F00..=0x1FFF => Script::Greek,
        0x0400..=0x052F => Script::Cyrillic,
        0x0600..=0x06FF | 0x0750..=0x077F => Script::Arabic,
        0x0590..=0x05FF => Script::Hebrew,
        0x0900..=0x097F => Script::Devanagari,
        0x0E00..=0x0E7F => Script::Thai,
        0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF => Script::Cjk,
        0xAC00..=0xD7AF | 0x1100..=0x11FF => Script::Hangul,
        _ => Script::Other,
    })
}

/// For a language with its own alphabet, whether that alphabet outnumbers the
/// Latin letters in the text. None for languages written in Latin letters,
/// where the letters cannot tell them from English.
pub fn writes_mostly_in(primary: &str, text: &str) -> Option<bool> {
    let own = script_of_language(primary);
    if matches!(own, Script::Latin | Script::Other) {
        return None;
    }
    let (mut mine, mut latin) = (0usize, 0usize);
    for s in text.chars().filter_map(script_of_char) {
        if s == own {
            mine += 1;
        } else if s == Script::Latin {
            latin += 1;
        }
    }
    Some(mine > latin)
}

/// Whether a transcript that was meant to be `primary` or English must be
/// redone with `primary` forced.
///
/// Two tests, either one is enough. The engine's own verdict: a detected
/// language that is neither the user's nor English. And the alphabet: a word
/// written in a system that neither of the two languages uses, which catches
/// the cases where the verdict is missing or wrong about itself.
/// Latin letters are never a reason on their own, because an English word in
/// a Greek sentence is the whole point of the mixed mode; a Latin word in a
/// third language is left to the verdict.
///
/// One word is enough. The first version asked for a quarter of all letters,
/// and on 17 September 2026 a 72-second Greek dictation carried "Епитопол."
/// straight through it: eight Cyrillic letters against four hundred Greek ones.
pub fn needs_lock(primary: &str, detected: Option<&str>, text: &str) -> bool {
    if let Some(d) = detected {
        let d = code_from_name(d);
        if !d.is_empty() && d != "auto" && d != primary && d != "en" {
            return true;
        }
    }
    let allowed = [script_of_language(primary), Script::Latin];
    text.split(|c: char| !c.is_alphabetic()).any(|word| {
        let mut foreign = 0usize;
        let mut whole_word_scripts = false;
        for s in word.chars().filter_map(script_of_char) {
            if s != Script::Other && !allowed.contains(&s) {
                foreign += 1;
                whole_word_scripts |= matches!(s, Script::Cjk | Script::Hangul);
            }
        }
        // A lone letter from another alphabet is noise, two make a word. One
        // Chinese, Japanese or Korean character is already a word.
        foreign >= 2 || (foreign == 1 && whole_word_scripts)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_verdict_in_a_third_language_locks() {
        assert!(needs_lock("el", Some("czech"), "Do tímhle světu."));
        assert!(needs_lock("el", Some("tr"), "Evet, tamam."));
        assert!(needs_lock("tr", Some("el"), "Καλημέρα σας."));
    }

    #[test]
    fn the_two_chosen_languages_pass() {
        assert!(!needs_lock("el", Some("el"), "Καλημέρα, τι κάνεις;"));
        assert!(!needs_lock("el", Some("en"), "Send it to Luram."));
        assert!(!needs_lock("el", None, "Καλημέρα, send it to Luram."));
        assert!(!needs_lock("tr", Some("turkish"), "Evet, tamam."));
    }

    #[test]
    fn a_foreign_alphabet_locks_even_without_a_verdict() {
        assert!(needs_lock("el", None, "สวัสดีครับ ทุกคน"));
        assert!(needs_lock("el", None, "Привет, как дела?"));
        assert!(needs_lock("en", None, "Καλημέρα σε όλους."));
        // one stray letter is not a language
        assert!(!needs_lock("el", None, "Καλημέρα я"));
    }

    #[test]
    fn one_foreign_word_in_a_long_sentence_locks() {
        // 17 September 2026: a 72-second Greek dictation with one Cyrillic word
        // in it, eight letters against more than four hundred Greek ones.
        let long = "Λοιπόν, θέλω να δεις το history στο dictation και να μου πεις τι έγινε με τις γλώσσες, \
                    γιατί πάλι βγήκαν μπερδεμένες. Епитопол. Και μετά πάμε στο επόμενο κομμάτι της δουλειάς.";
        assert!(needs_lock("el", None, long));
        assert!(needs_lock("el", Some("greek"), long), "a Greek verdict does not excuse Cyrillic letters");
        // a single Hangul syllable is a whole word
        assert!(needs_lock("el", None, "Καλά. 연"));
    }

    #[test]
    fn verdicts_heard_on_short_greek_words() {
        // what the engine answered on 17 September 2026 for single Greek words
        // said on their own, in auto mode
        assert!(needs_lock("el", Some("indonesian"), "Terima kasih."));
        assert!(needs_lock("el", Some("spanish"), "Endaxi."));
        assert!(needs_lock("el", Some("polish"), "Wromia."));
        assert!(needs_lock("el", Some("korean"), "연"));
        // silence with VAD comes back empty and labelled English
        assert!(!needs_lock("el", Some("english"), ""));
    }

    #[test]
    fn latin_letters_never_lock_on_their_own() {
        assert!(!needs_lock("el", None, "Do tímhle světu."), "without the verdict a Latin sentence could be English");
        assert!(!needs_lock("ru", None, "Hello world"));
    }

    #[test]
    fn names_become_codes() {
        assert_eq!(code_from_name("Greek"), "el");
        assert_eq!(code_from_name("czech"), "cs");
        assert_eq!(code_from_name("el"), "el");
        assert_eq!(code_from_name("klingon"), "klingon");
    }
}
