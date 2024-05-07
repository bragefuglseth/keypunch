use include_dir::{include_dir, Dir};
use rand::prelude::*;
use rand::seq::index::sample;
use unicode_segmentation::UnicodeSegmentation;

static EMBEDDED_WORD_LIST_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data/word_lists");
const EMBEDDED_WORD_LIST_LEN: usize = 200;
pub const CHUNK_GRAPHEME_COUNT: usize = 400;

type Punctuation = (Option<&'static str>, Option<&'static str>, bool, usize);

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

pub fn simple(lang_code: &str) -> Option<String> {
    match lang_code {
        "en_US" => Some(simple_generic("en_US")),
        "nb_NO" => Some(simple_generic("nb_NO")),
        "nn_NO" => Some(simple_generic("nn_NO")),
        "sv_SE" => Some(simple_generic("sv_SE")),
        _ => None,
    }
}

pub fn advanced(lang_code: &str) -> Option<String> {
    match lang_code {
        "en_US" => Some(advanced_generic(
            "en_US",
            &[
                (None, Some("."), true, 3),
                (None, Some(","), false, 5),
                (None, Some(":"), true, 1),
                (None, Some("!"), true, 2),
                (None, Some("?"), true, 2),
                (Some("\""), Some("\""), false, 1),
                (Some("("), Some(")"), false, 1),
            ],
        )),
        "nb_NO" => Some(advanced_generic(
            "nb_NO",
            &[
                (None, Some("."), true, 3),
                (None, Some(","), false, 5),
                (None, Some(":"), false, 1),
                (None, Some("!"), true, 2),
                (None, Some("?"), true, 2),
                (Some("\""), Some("\""), false, 1),
                (Some("("), Some(")"), false, 1),
            ],
        )),
        "nn_NO" => Some(advanced_generic(
            "nn_NO",
            &[
                (None, Some("."), true, 3),
                (None, Some(","), false, 5),
                (None, Some(":"), false, 1),
                (None, Some("!"), true, 2),
                (None, Some("?"), true, 2),
                (Some("\""), Some("\""), false, 1),
                (Some("("), Some(")"), false, 1),
            ],
        )),
        "sv_SE" => Some(advanced_generic(
            "sv_SE",
            &[
                (None, Some("."), true, 3),
                (None, Some(","), false, 5),
                (None, Some(":"), true, 1),
                (None, Some("!"), true, 2),
                (None, Some("?"), true, 2),
                (Some("\""), Some("\""), false, 1),
                (Some("("), Some(")"), false, 1),
            ],
        )),
        _ => None,
    }
}

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

fn advanced_generic(lang_code: &str, punctuation: &[Punctuation]) -> String {
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

    if let Some(word) = generated.get_mut(0) {
        *word = uppercase_first_letter(word);
    }

    let len = generated.len();
    for i in sample(&mut rng, len, len / 20) {
        if let Some(word) = generated.get_mut(i) {
            *word = match [1, 2, 3, 4]
                .choose_weighted(&mut rng, |n| 5 - n)
                .expect("the weighted choice is performed on a static list")
            {
                1 => rng.gen_range(0..=9).to_string(),
                2 => rng.gen_range(10..=99).to_string(),
                3 => rng.gen_range(100..=999).to_string(),
                4 => rng.gen_range(1000..=9999).to_string(),
                _ => unreachable!("range generated is 1..=4"),
            }
        }
    }

    for i in sample(&mut rng, len - 1, len / 4) {
        if let Some(word) = generated.get_mut(i) {
            if let Ok((prefix_opt, suffix_opt, ends_sentence, weight)) =
                punctuation.choose_weighted(&mut rng, |(_, _, _, w)| *w)
            {
                *word =
                    insert_punctuation(&word, (*prefix_opt, *suffix_opt, *ends_sentence, *weight));

                if *ends_sentence {
                    if let Some(next) = generated.get_mut(i + 1) {
                        *next = uppercase_first_letter(&next);
                    }
                }
            }
        }
    }

    if let Some(end_punctuation) = punctuation
        .iter()
        .filter(|(_, _, ends_sentence, _)| *ends_sentence)
        .choose(&mut rng)
    {
        if let Some(word) = generated.get_mut(len) {
            *word = insert_punctuation(&word, *end_punctuation);
        }
    }

    generated.into_iter().map(|s| format!("{s} ")).collect()
}

fn insert_punctuation(word: &str, punctuation: Punctuation) -> String {
    match punctuation {
        (Some(pre), Some(suf), _, _) => format!("{pre}{word}{suf}"),
        (Some(pre), None, _, _) => format!("{pre}{word}"),
        (None, Some(suf), _, _) => format!("{word}{suf}"),
        _ => word.to_string(),
    }
}

fn uppercase_first_letter(s: &str) -> String {
    s.chars()
        .scan(false, |has_found_uppercase, c| {
            if c.is_alphabetic() && !*has_found_uppercase {
                *has_found_uppercase = true;
                Some(c.to_uppercase().collect())
            } else {
                Some(c.to_string())
            }
        })
        .collect()
}
