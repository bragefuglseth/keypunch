use crate::enums::Language;
use include_dir::{include_dir, Dir};
use rand::prelude::*;
use rand::seq::index::sample;
use unicode_segmentation::UnicodeSegmentation;

static EMBEDDED_WORD_LIST_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data/word_lists");
const EMBEDDED_WORD_LIST_LEN: usize = 200;
pub const CHUNK_GRAPHEME_COUNT: usize = 400;

#[derive(Clone, Copy)]
struct Punctuation<'a> {
    prefix: Option<&'a str>,
    suffix: Option<&'a str>,
    ends_sentence: bool, // Whether the next word should start with a capitalized letter or not
    weight: f64,         // How much the type of punctuation should be used compared to other ones
}

impl<'a> Punctuation<'a> {
    // Convenience function for punctuation that only has a suffix part (like periods or commas)
    fn suffix(suffix: &'a str, ends_sentence: bool, weight: f64) -> Self {
        Punctuation {
            prefix: None,
            suffix: Some(suffix),
            ends_sentence,
            weight,
        }
    }

    // Convenience function for punctuation that "wraps" a word (like quotation marks or brackets)
    fn wrapping(prefix: &'a str, suffix: &'a str, ends_sentence: bool, weight: f64) -> Self {
        Punctuation {
            prefix: Some(prefix),
            suffix: Some(suffix),
            ends_sentence,
            weight,
        }
    }
}

fn words_from_lang_code(lang_code: &str) -> [&'static str; EMBEDDED_WORD_LIST_LEN] {
    let s = EMBEDDED_WORD_LIST_DIR
        .get_file(&format!("{lang_code}.txt"))
        .expect(&format!("word list for \"{}\" exists", lang_code))
        .contents_utf8()
        .expect("file has valid utf8 contents");

    s.lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .try_into()
        .expect("word list vec can be parsed into [&str; 200]")
}

// Only lowercase letters, no punctuation or numbers
pub fn simple(language: Language) -> String {
    match language {
        Language::EnglishUS => simple_generic("en_US"),
        Language::NorwegianBokmaal => simple_generic("nb_NO"),
        Language::NorwegianNynorsk => simple_generic("nn_NO"),
        Language::Spanish => simple_generic("es_ES"),
        Language::Swedish => simple_generic("sv_SE"),
    }
}

// Some capitalized letters, punctuation and numbers
pub fn advanced(language: Language) -> String {
    match language {
        Language::EnglishUS => advanced_generic(
            "en_US",
            &[
                Punctuation::suffix(".", true, 0.6),
                Punctuation::suffix(",", false, 1.0),
                Punctuation::suffix(";", false, 0.1),
                Punctuation::suffix(":", false, 0.2),
                Punctuation::suffix("!", true, 0.3),
                Punctuation::suffix("?", true, 0.3),
                Punctuation::wrapping("\"", "\"", false, 0.2),
                Punctuation::wrapping("(", ")", false, 0.1),
            ],
        ),
        Language::NorwegianBokmaal => advanced_generic(
            "nb_NO",
            &[
                Punctuation::suffix(".", true, 0.6),
                Punctuation::suffix(",", false, 1.0),
                Punctuation::suffix(";", false, 0.1),
                Punctuation::suffix(":", false, 0.2),
                Punctuation::suffix("!", true, 0.3),
                Punctuation::suffix("?", true, 0.3),
                Punctuation::wrapping("\"", "\"", false, 0.2),
                Punctuation::wrapping("(", ")", false, 0.1),
            ],
        ),
        Language::NorwegianNynorsk => advanced_generic(
            "nn_NO",
            &[
                Punctuation::suffix(".", true, 0.6),
                Punctuation::suffix(",", false, 1.0),
                Punctuation::suffix(";", false, 0.1),
                Punctuation::suffix(":", false, 0.2),
                Punctuation::suffix("!", true, 0.3),
                Punctuation::suffix("?", true, 0.3),
                Punctuation::wrapping("\"", "\"", false, 0.2),
                Punctuation::wrapping("(", ")", false, 0.1),
            ],
        ),
        Language::Spanish => advanced_generic(
            "es_ES",
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
        Language::Swedish => advanced_generic(
            "sv_SE",
            &[
                Punctuation::suffix(".", true, 0.6),
                Punctuation::suffix(",", false, 1.0),
                Punctuation::suffix(";", false, 0.1),
                Punctuation::suffix(":", false, 0.2),
                Punctuation::suffix("!", true, 0.3),
                Punctuation::suffix("?", true, 0.3),
                Punctuation::wrapping("\"", "\"", false, 0.2),
                Punctuation::wrapping("(", ")", false, 0.1),
            ],
        ),
    }
}

// Should work for most languages
fn simple_generic(lang_code: &str) -> String {
    let word_list = words_from_lang_code(lang_code);

    let mut rng = thread_rng();
    let mut s = String::new();
    while s.graphemes(true).count() < CHUNK_GRAPHEME_COUNT {
        s.push_str(
            word_list
                .choose(&mut rng)
                .expect("word list contains at least 1 word"),
        );
        s.push(' ');
    }

    s
}

// Should work for most languages
fn advanced_generic(lang_code: &str, punctuations: &[Punctuation]) -> String {
    let word_list = words_from_lang_code(lang_code);

    let mut rng = thread_rng();

    let mut generated: Vec<String> = Vec::new();
    while generated
        .iter()
        .map(|s| s.graphemes(true))
        .flatten()
        .count()
        < CHUNK_GRAPHEME_COUNT
    {
        generated.push(
            word_list
                .choose(&mut rng)
                .expect("word list contains at least 1 word")
                .to_string(),
        );
    }

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

    generated.into_iter().map(|s| format!("{s} ")).collect()
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
