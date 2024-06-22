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
    #[strum(message = "العربية", to_string = "ar_SA")]
    Arabic,
    #[strum(message = "Български", to_string = "bg_BG")]
    Bulgarian,
    #[strum(message = "Dansk", to_string = "da_DK")]
    Danish,
    #[default]
    #[strum(message = "English", to_string = "en_US")]
    English,
    #[strum(message = "Français", to_string = "fr_FR")]
    French,
    #[strum(message = "Deutsch", to_string = "de_DE")]
    German,
    #[strum(message = "हिन्दी", to_string = "hi_IN")]
    Hindi,
    #[strum(message = "Magyar", to_string = "hu_HU")]
    Hungarian,
    #[strum(message = "Kinyarwanda", to_string = "rw_RW")]
    Kinyarwanda,
    #[strum(message = "한국어", to_string = "ko_KR")]
    Korean,
    #[strum(message = "नेपाली", to_string = "ne_NP")]
    Nepali,
    #[strum(message = "Norsk bokmål", to_string = "nb_NO")]
    NorwegianBokmaal,
    #[strum(message = "Norsk nynorsk", to_string = "nn_NO")]
    NorwegianNynorsk,
    #[strum(message = "Español", to_string = "es")]
    Spanish,
    #[strum(message = "Swahili", to_string = "sw_KE")]
    Swahili,
    #[strum(message = "Svenska", to_string = "sv_SE")]
    Swedish,
    #[strum(message = "Schweizerdeutsch", to_string = "de_CH")]
    SwissGerman,
    #[strum(message = "Українська", to_string = "uk_UA")]
    Ukranian,
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

// Only lowercase letters, no punctuation or numbers
pub fn simple(language: Language) -> String {
    match language {
        Language::Arabic
        | Language::Bulgarian
        | Language::English
        | Language::Danish
        | Language::French
        | Language::German
        | Language::Hindi
        | Language::Hungarian
        | Language::Kinyarwanda
        | Language::Korean
        | Language::Nepali
        | Language::NorwegianBokmaal
        | Language::NorwegianNynorsk
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
        Language::Bulgarian
        | Language::Danish
        | Language::English
        | Language::German
        | Language::Hungarian
        | Language::Kinyarwanda
        | Language::Korean
        | Language::NorwegianBokmaal
        | Language::NorwegianNynorsk
        | Language::Swahili
        | Language::Swedish
        | Language::SwissGerman
        | Language::Ukranian => advanced_generic(&language.to_string(), " ", GENERIC_PUNCTUATION),
        Language::Arabic => advanced_generic(
            "ar_SA_advanced",
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
        ),
        // The French language has a space before certain punctuation marks.
        // See <https://www.frenchtoday.com/blog/french-grammar/french-punctuation/>
        Language::French => advanced_generic(
            &language.to_string(),
            " ",
            &[
                Punctuation::suffix(".", true, 0.6),
                Punctuation::suffix(",", false, 1.0),
                Punctuation::suffix(" ;", false, 0.1),
                Punctuation::suffix(" :", false, 0.2),
                Punctuation::suffix(" !", true, 0.3),
                Punctuation::suffix(" ?", true, 0.3),
                Punctuation::wrapping("\"", "\"", false, 0.2),
                Punctuation::wrapping("(", ")", false, 0.1),
            ],
        ),
        // Hindi & Nepali use Devanagari punctuation
        Language::Hindi | Language::Nepali => advanced_generic(
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
fn advanced_generic(lang_code: &str, spacing: &str, punctuations: &[Punctuation]) -> String {
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
            *word = random_number_weighted(&mut rng).to_string();
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

// Generates a number between 0 and 9999, with a bias towards smaller numbers
fn random_number_weighted(rng: &mut ThreadRng) -> usize {
    match [1, 2, 3, 4]
        // Makes the likelihood of getting a number with a certain amount of digits
        // the "inverse" of the amount of digits (so 4 digits = weight 1, while 1 digit = weight 4)
        .choose_weighted(rng, |n| 5 - n)
        .expect("the weighted choice is performed on a static list")
    {
        1 => rng.gen_range(0..=9),
        2 => rng.gen_range(10..=99),
        3 => rng.gen_range(100..=999),
        4 => rng.gen_range(1000..=9999),
        _ => unreachable!("range generated is 1..=4"),
    }
}
