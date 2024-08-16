use include_dir::{include_dir, Dir};
use rand::prelude::*;
use rand::seq::index::sample;
use strum_macros::{Display as EnumDisplay, EnumIter, EnumMessage, EnumString};
use unicode_segmentation::UnicodeSegmentation;

static EMBEDDED_WORD_LIST_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data/word_lists");
pub const CHUNK_GRAPHEME_COUNT: usize = 400;

// All languages here MUST have a corresponding file in data/word_lists/{lang_code}.txt
#[derive(Clone, Copy, Default, EnumDisplay, EnumString, EnumIter, EnumMessage, PartialEq)]
pub enum Language {
    #[strum(message = "العربية", to_string = "ar")]
    Arabic,
    #[strum(message = "বাংলা", to_string = "bn")]
    Bangla,
    #[strum(message = "Български", to_string = "bg")]
    Bulgarian,
    #[strum(message = "Čeština", to_string = "cs")]
    Czech,
    #[strum(message = "Dansk", to_string = "da")]
    Danish,
    #[default]
    #[strum(message = "English", to_string = "en")]
    English,
    #[strum(message = "Français", to_string = "fr")]
    French,
    #[strum(message = "Deutsch", to_string = "de")]
    German,
    #[strum(message = "עברית", to_string = "he")]
    Hebrew,
    #[strum(message = "हिन्दी", to_string = "hi")]
    Hindi,
    #[strum(message = "Magyar", to_string = "hu")]
    Hungarian,
    #[strum(message = "Italiano", to_string = "it")]
    Italian,
    #[strum(message = "Kinyarwanda", to_string = "rw")]
    Kinyarwanda,
    #[strum(message = "한국어", to_string = "ko")]
    Korean,
    #[strum(message = "नेपाली", to_string = "ne")]
    Nepali,
    #[strum(message = "Norsk bokmål", to_string = "nb")]
    NorwegianBokmaal,
    #[strum(message = "Norsk nynorsk", to_string = "nn")]
    NorwegianNynorsk,
    #[strum(message = "Occitan", to_string = "oc")]
    Occitan,
    #[strum(message = "Polski", to_string = "pl")]
    Polish,
    // Blocked on <https://github.com/bragefuglseth/keypunch/issues/46>
    // #[strum(message = "Português", to_string = "pt")]
    // Portuguese,
    #[strum(message = "Русский", to_string = "ru")]
    Russian,
    #[strum(message = "Español", to_string = "es")]
    Spanish,
    #[strum(message = "Swahili", to_string = "sw")]
    Swahili,
    #[strum(message = "Svenska", to_string = "sv")]
    Swedish,
    #[strum(message = "Schweizerdeutsch", to_string = "de_CH")]
    SwissGerman,
    #[strum(message = "Українська", to_string = "uk")]
    Ukranian,
}

#[derive(Clone, Copy)]
struct Punctuation<'a> {
    prefix: Option<&'a str>,
    suffix: Option<&'a str>,
    ends_sentence: bool, // Whether the next word should start with a capitalized letter or not
    weight: f64,         // How much the type of punctuation should be used compared to other ones
}

impl<'a> Punctuation<'a> {
    // Convenience function for punctuation that only has a suffix part (like periods or commas)
    const fn suffix(suffix: &'a str, ends_sentence: bool, weight: f64) -> Self {
        Punctuation {
            prefix: None,
            suffix: Some(suffix),
            ends_sentence,
            weight,
        }
    }

    // Convenience function for punctuation that "wraps" a word (like quotation marks or brackets)
    const fn wrapping(prefix: &'a str, suffix: &'a str, ends_sentence: bool, weight: f64) -> Self {
        Punctuation {
            prefix: Some(prefix),
            suffix: Some(suffix),
            ends_sentence,
            weight,
        }
    }
}

// A set of punctuation that works fine for most western languages
const GENERIC_PUNCTUATION: &'static [Punctuation] = &[
    Punctuation::suffix(".", true, 0.6),
    Punctuation::suffix(",", false, 1.0),
    Punctuation::suffix(";", false, 0.1),
    Punctuation::suffix(":", false, 0.2),
    Punctuation::suffix("!", true, 0.3),
    Punctuation::suffix("?", true, 0.3),
    Punctuation::wrapping("\"", "\"", false, 0.2),
    Punctuation::wrapping("(", ")", false, 0.1),
];

type Numerals = [&'static str; 10];

const WESTERN_ARABIC_NUMERALS: &'static Numerals =
    &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
const DEVANAGARI_NUMERALS: &'static Numerals = &["०", "१", "२", "३", "४", "५", "६", "७", "८", "९"];
const BANGLA_NUMERALS: &'static Numerals = &["০", "১", "২", "৩", "৪", "৫", "৬", "৭", "৮", "৯"];

// Only lowercase letters, no punctuation or numbers
pub fn simple(language: Language) -> String {
    match language {
        Language::Arabic
        | Language::Bangla
        | Language::Bulgarian
        | Language::Czech
        | Language::English
        | Language::Danish
        | Language::French
        | Language::German
        | Language::Hebrew
        | Language::Hindi
        | Language::Hungarian
        | Language::Italian
        | Language::Kinyarwanda
        | Language::Korean
        | Language::Nepali
        | Language::NorwegianBokmaal
        | Language::NorwegianNynorsk
        | Language::Occitan
        | Language::Polish
        // | Language::Portuguese
        | Language::Russian
        | Language::Spanish
        | Language::Swahili
        | Language::Swedish
        | Language::SwissGerman
        | Language::Ukranian => simple_generic(&language.to_string(), " "),
    }
}

// Some capitalized letters, punctuation and numbers
pub fn advanced(language: Language) -> String {
    match language {
        Language::Danish
        | Language::English
        | Language::German
        | Language::Hebrew
        | Language::Hungarian
        | Language::Italian
        | Language::Kinyarwanda
        | Language::Korean
        | Language::NorwegianBokmaal
        | Language::NorwegianNynorsk
        | Language::Occitan
        | Language::Polish
        // | Language::Portuguese
        | Language::Russian
        | Language::Swahili
        | Language::Swedish
        | Language::SwissGerman
        | Language::Ukranian => advanced_generic(
            &language.to_string(),
            " ",
            GENERIC_PUNCTUATION,
            WESTERN_ARABIC_NUMERALS,
        ),
        // Arabic has its own set of punctuation and a couple of words with vowel markers
        Language::Arabic => advanced_generic(
            "ar_advanced",
            " ",
            &[
                Punctuation::suffix(".", true, 0.6),
                Punctuation::suffix("،", false, 1.0),
                Punctuation::suffix("؛", false, 0.1),
                Punctuation::suffix(":", false, 0.2),
                Punctuation::suffix("!", true, 0.3),
                Punctuation::suffix("؟", true, 0.3),
                Punctuation::wrapping("\"", "\"", false, 0.2),
                Punctuation::wrapping("(", ")", false, 0.1),
            ],
            WESTERN_ARABIC_NUMERALS,
        ),
        // Bulgarians apparently have a pretty strong culture of using „ and “ over " and ".
        // See <https://github.com/bragefuglseth/keypunch/issues/41> if this ever comes up again
        Language::Bulgarian => advanced_generic(
            &language.to_string(),
            " ",
            &[
                Punctuation::suffix(".", true, 0.6),
                Punctuation::suffix(",", false, 1.0),
                Punctuation::suffix(";", false, 0.1),
                Punctuation::suffix(":", false, 0.2),
                Punctuation::suffix("!", true, 0.3),
                Punctuation::suffix("?", true, 0.3),
                Punctuation::wrapping("„", "“", false, 0.2),
                Punctuation::wrapping("(", ")", false, 0.1),
            ],
            WESTERN_ARABIC_NUMERALS,
        ),
        // See <https://github.com/bragefuglseth/keypunch/issues/58>. The quotation is
        // slightly different from above.
        Language::Czech => advanced_generic(
            &language.to_string(),
            " ",
            &[
                Punctuation::suffix(".", true, 0.6),
                Punctuation::suffix(",", false, 1.0),
                Punctuation::suffix(";", false, 0.1),
                Punctuation::suffix(":", false, 0.2),
                Punctuation::suffix("!", true, 0.3),
                Punctuation::suffix("?", true, 0.3),
                Punctuation::wrapping("„", "”", false, 0.2),
                Punctuation::wrapping("(", ")", false, 0.1),
            ],
            WESTERN_ARABIC_NUMERALS,
        ),
        // The French language has a space before certain punctuation marks, and keyboard layouts
        // that allow for typing guillemet quotation marks easily.
        // See <https://github.com/bragefuglseth/keypunch/issues/47>
        Language::French => advanced_generic(
            &language.to_string(),
            " ",
            &[
                Punctuation::suffix(".", true, 0.6),
                Punctuation::suffix(",", false, 1.0),
                Punctuation::suffix("\u{202F};", false, 0.1),
                Punctuation::suffix("\u{202F}:", false, 0.2),
                Punctuation::suffix("\u{202F}!", true, 0.3),
                Punctuation::suffix("\u{202F}?", true, 0.3),
                Punctuation::wrapping("«\u{202F}", "\u{202F}»", false, 0.2),
                Punctuation::wrapping("(", ")", false, 0.1),
            ],
            WESTERN_ARABIC_NUMERALS,
        ),
        // Hindi & Nepali use Devanagari punctuation
        Language::Bangla | Language::Hindi | Language::Nepali => advanced_generic(
            &language.to_string(),
            " ",
            &[
                Punctuation::suffix("।", true, 0.6),
                Punctuation::suffix(",", false, 1.0),
                Punctuation::suffix(";", false, 0.1),
                Punctuation::suffix(":", false, 0.2),
                Punctuation::suffix("!", true, 0.3),
                Punctuation::suffix("?", true, 0.3),
                Punctuation::wrapping("\"", "\"", false, 0.2),
                Punctuation::wrapping("(", ")", false, 0.1),
            ],
            if language == Language::Bangla {
                BANGLA_NUMERALS
            } else {
                DEVANAGARI_NUMERALS
            },
        ),
        // Spanish has "wrapping" exclamation points and question marks
        Language::Spanish => advanced_generic(
            &language.to_string(),
            " ",
            &[
                Punctuation::suffix(".", true, 0.6),
                Punctuation::suffix(",", false, 1.0),
                Punctuation::suffix(";", false, 0.1),
                Punctuation::suffix(":", false, 0.2),
                Punctuation::wrapping("¡", "!", true, 0.3),
                Punctuation::wrapping("¿", "?", true, 0.3),
                Punctuation::wrapping("\"", "\"", false, 0.2),
                Punctuation::wrapping("(", ")", false, 0.1),
            ],
            WESTERN_ARABIC_NUMERALS,
        ),
    }
}

// Should work for most languages
fn simple_generic(lang_code: &str, spacing: &str) -> String {
    let mut rng = thread_rng();
    let generated = random_words_from_lang_code(lang_code, &mut rng);

    generated.into_iter().map(|s| s + spacing).collect()
}

// Should work for most languages
fn advanced_generic(
    lang_code: &str,
    spacing: &str,
    punctuations: &[Punctuation],
    numerals: &Numerals,
) -> String {
    let mut rng = thread_rng();

    let mut generated = random_words_from_lang_code(lang_code, &mut rng);

    // The very first letter in the chunk should always be capitalized
    if let Some(word) = generated.get_mut(0) {
        *word = uppercase_first_letter(word);
    }

    // Swaps out some words with numbers
    let len = generated.len();
    for i in sample(&mut rng, len, len / 20) {
        if let Some(word) = generated.get_mut(i) {
            *word = random_number_weighted(&numerals, &mut rng);
        }
    }

    // Sample from the entire text except for the last word, since that gets punctuation
    // further down in the function
    for i in sample(&mut rng, len - 2, len / 4) {
        if let Some(word) = generated.get_mut(i) {
            if let Ok(punctuation) = punctuations.choose_weighted(&mut rng, |p| p.weight) {
                *word = insert_punctuation(&word, *punctuation);

                if punctuation.ends_sentence {
                    if let Some(next) = generated.get_mut(i + 1) {
                        *next = uppercase_first_letter(&next);
                    }
                }
            }
        }
    }

    // Insert random "sentence ending" punctuation on last word
    if let Some(end_punctuation) = punctuations
        .iter()
        .filter(|p| p.ends_sentence)
        .choose(&mut rng)
    {
        if let Some(word) = generated.get_mut(len - 1) {
            *word = insert_punctuation(&word, *end_punctuation);
        }
    }

    generated.into_iter().map(|s| s + spacing).collect()
}

fn uppercase_first_letter(s: &str) -> String {
    s.chars()
        .scan(false, |has_found_alphanumeric, c| {
            if c.is_alphanumeric() && !*has_found_alphanumeric {
                *has_found_alphanumeric = true;
                Some(c.to_uppercase().collect())
            } else {
                Some(c.to_string())
            }
        })
        .collect()
}

fn words_from_lang_code(lang_code: &str) -> Vec<&'static str> {
    let s = EMBEDDED_WORD_LIST_DIR
        .get_file(&format!("{lang_code}.txt"))
        .expect(&format!("word list for \"{}\" exists", lang_code))
        .contents_utf8()
        .expect("file has valid utf8 contents");

    s.lines().filter(|line| !line.is_empty()).collect()
}

fn random_words_from_lang_code(lang_code: &str, rng: &mut ThreadRng) -> Vec<String> {
    let word_list = words_from_lang_code(lang_code);

    let mut generated: Vec<String> = Vec::new();
    while generated.iter().flat_map(|s| s.graphemes(true)).count() < CHUNK_GRAPHEME_COUNT {
        let new_word = word_list
            .choose(rng)
            .expect("word list contains at least 1 word");

        let unique = if let Some(previous_words) = generated.last_chunk::<2>() {
            previous_words.iter().all(|word| word != new_word)
        } else {
            true
        };

        if unique {
            generated.push(new_word.to_string());
        }
    }

    generated
}

fn insert_punctuation(word: &str, punctuation: Punctuation) -> String {
    match (punctuation.prefix, punctuation.suffix) {
        (Some(pre), Some(suf)) => format!("{pre}{word}{suf}"),
        (Some(pre), None) => format!("{pre}{word}"),
        (None, Some(suf)) => format!("{word}{suf}"),
        _ => word.to_string(),
    }
}

// Generates a number between 0 and 9999, with support for multiple numeral systems
// and a bias towards smaller numbers
fn random_number_weighted(numerals: &Numerals, rng: &mut ThreadRng) -> String {
    // The tuples consist of a number length and the proportion of numbers that should have
    // that amount of digits
    let number_length = *[(1, 0.4), (2, 0.3), (3, 0.2), (4, 0.1)]
        .choose_weighted(rng, |(_, w)| *w)
        .map(|(n, _)| n)
        .unwrap();

    let mut s = String::new();

    for _ in 0..number_length {
        let digit = numerals.choose(rng).unwrap();
        s.push_str(&digit);
    }

    s
}
