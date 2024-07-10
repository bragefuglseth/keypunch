use std::iter::zip;
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;

// String replacements for when text is displayed in the text view
const REPLACEMENTS: &'static [(&'static str, &'static str)] = &[
    ("\n", "↲\n"),            // Visually indicate enter
];

// Accepted alternate ways to type out certain characters
const ALIASES: &'static [(&'static str, &'static str)] = &[
    ("Æ", "Ae"), // The French use this to type out æ
    ("æ", "ae"),
    ("Œ", "Oe"), // The French use this to type out œ
    ("œ", "oe"),
    ("«", "\""), // Guillemet quotation marks
    ("»", "\""),
    ("„", "\""), // Typographic quotation marks
    ("”", "\""),
    ("“", "\""),
    ("’", "'"),               // Typoographic apostrophe
    ("\u{00A0}", "\u{0020}"), // Non-breaking spaces made typable as regular ones
    ("\u{202F}", "\u{0020}"),
];

// The largest grapheme count of any current alias, manually kept track of for performance reasons.
// If this is too large, nothing will happen. If it's too small, larger aliases won't be recognized.
const ALIAS_MAX_SIZE: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphemeState {
    Correct,
    Unfinished,
    Mistake,
}

pub fn process_custom_text(s: &str) -> String {
    let mut s: String = s.lines().map(|l| l.trim().to_owned() + "\n").collect();

    // Remove last `/n`
    s.pop();

    s
}

pub fn insert_replacements(string: &str) -> String {
    let mut s = string.to_string();
    for (original, replacement) in REPLACEMENTS {
        s = s.replace(original, replacement);
    }
    s
}

// Gets the corresponding whitespace marker of a str, if any
pub fn replacement(s: &str) -> Option<&'static str> {
    REPLACEMENTS
        .iter()
        .find(|(whitespace, _)| *whitespace == s)
        .map(|(_, marker)| *marker)
}

fn alias(s: &str) -> Option<&'static str> {
    ALIASES
        .iter()
        .find(|(letter, _)| *letter == s)
        .map(|(_, alias)| *alias)
}

// Finds out if there is a full or partial alias (alternate typing method) being typed out
// now, with a corresponding character at that position in the original text
pub fn end_alias(original: &str, typed: &str) -> Option<(String, String, bool)> {
    let original_cropped: Vec<(usize, &str)> = original
        .graphemes(true)
        .enumerate()
        .skip(
            // Any current valid letter is in `original[typed-ALIAS_MAX_SIZE..typed]`
            typed
                .graphemes(true)
                .count()
                .checked_sub(ALIAS_MAX_SIZE)
                .unwrap_or(0),
        )
        .take(ALIAS_MAX_SIZE)
        .collect();

    let potential_positions: Vec<(usize, &str, &str)> = original_cropped
        .iter()
        .rev()
        .filter_map(|(n, original_letter)| {
            alias(original_letter).map(|alias| (*n, *original_letter, alias))
        })
        .collect();

    for (pos, letter, alias) in potential_positions {
        if let Some(start_index) = typed.grapheme_indices(true).nth(pos).map(|(i, _)| i) {
            let potential_alias = &typed[start_index..];

            if alias == potential_alias {
                return Some((letter.to_string(), potential_alias.to_string(), true));
            } else if alias.starts_with(&potential_alias) {
                return Some((letter.to_string(), potential_alias.to_string(), false));
            }
        }
    }

    None
}

// The returned tuples contain a "correct" bool, as well as the line number + start/end indices
// the bool applies to. We have to use the exact byte indices because GtkTextBuffer's `iter_at_offset()`
// function doesn't align perfectly with `graphemes()` from the unicode_segmentation crate.
// The function accounts for replacements.
pub fn validate_with_replacements(
    original: &str,
    typed: &str,
    unfinished_letter_length: usize,
) -> Vec<(GraphemeState, usize, usize, usize)> {
    let (typed, unfinished_letter_length, unfinished_overshoot) =
        if let Some((letter, potential_alias, false)) = end_alias(original, typed) {
            let split_index = typed
                .grapheme_indices(true)
                .rev()
                .nth(potential_alias.graphemes(true).count() - 1)
                .map(|(i, _)| i)
                .unwrap_or(0);

            (
                &typed[..split_index],
                letter.graphemes(true).count(),
                potential_alias.graphemes(true).count(),
            )
        } else if unfinished_letter_length > 0 {
            (typed, unfinished_letter_length, unfinished_letter_length)
        } else {
            (typed, unfinished_letter_length, 0)
        };

    let last_typed_grapheme_offset = typed.graphemes(true).count().checked_sub(1).unwrap_or(0);

    original
        .split_inclusive("\n")
        .enumerate()
        .flat_map(|(line_num, line)| {
            line.grapheme_indices(true)
                .map(move |grapheme| (line_num, grapheme))
        })
        .zip(typed.graphemes(true).chain(vec![" "; unfinished_letter_length]))
        .enumerate()
        .scan(
            (0, 0),
            |(accumulator_line, accumulated_offset),
             (offset, ((line_num, (original_grapheme_idx, original_grapheme)), typed_grapheme))| {
                let potentially_unfinished = (original_grapheme.starts_with(typed_grapheme) && offset == last_typed_grapheme_offset)
                    || offset + unfinished_overshoot > last_typed_grapheme_offset;

                let state = if original_grapheme == typed_grapheme {
                    GraphemeState::Correct
                } else if potentially_unfinished {
                    GraphemeState::Unfinished
                } else {
                    GraphemeState::Mistake
                };

                if *accumulator_line != line_num {
                    *accumulator_line = line_num;
                    *accumulated_offset = 0;
                }

                if let Some(replacement) = replacement(original_grapheme) {
                    let vec = Some(
                        replacement
                            .grapheme_indices(true)
                            .map(|(idx, grapheme)| {
                                let adjusted_idx = idx + *accumulated_offset + original_grapheme_idx;
                                (
                                    state,
                                    line_num,
                                    adjusted_idx,
                                    adjusted_idx + grapheme.len(),
                                )
                            })
                            .collect::<Vec<_>>(),
                    );

                    *accumulated_offset += replacement.len().abs_diff(original_grapheme.len());

                    vec
                } else {
                    let adjusted_idx = original_grapheme_idx + *accumulated_offset;
                    Some(vec![(
                        state,
                        line_num,
                        adjusted_idx,
                        adjusted_idx + original_grapheme.len(),
                    )])
                }
            },
        )
        .flatten()
        .collect()
}

// Get the line / byte offset of a char, taking replacements and aliases into account. This
// function is used to e.g. position the caret
pub fn line_offset_with_replacements(
    original: &str,
    typed: &str,
    unfinished_letter_length: usize,
) -> (usize, usize) {
    let typed_graphemes = typed.graphemes(true).count();

    let grapheme_idx = if let Some((letter, potential_alias, _)) = end_alias(original, typed) {
        typed_graphemes - potential_alias.graphemes(true).count() + letter.graphemes(true).count()
    } else if unfinished_letter_length > 0 {
        typed_graphemes + unfinished_letter_length
    } else {
        typed_graphemes
    };

    let line_num = original
        .split_inclusive("\n")
        .enumerate()
        .flat_map(|(num, line)| line.graphemes(true).map(move |_| num))
        .nth(grapheme_idx)
        .unwrap_or(0);

    let line_grapheme_offset = original
        .split_inclusive("\n")
        .take(line_num)
        .flat_map(|line| line.graphemes(true))
        .count();

    let byte_offset = original
        .graphemes(true)
        .take(grapheme_idx)
        .skip(line_grapheme_offset)
        .map(|grapheme| {
            if let Some(replacement) = replacement(grapheme) {
                replacement.len()
            } else {
                grapheme.len()
            }
        })
        .sum();

    (line_num, byte_offset)
}

pub fn pop_grapheme_in_place(s: &mut String, graphemes: usize) {
    for _ in 0..graphemes {
        let last_chars = s.graphemes(true).last().unwrap_or("").chars().count();

        for _ in 0..last_chars {
            s.pop();
        }
    }
}

// Adjusts the typed text length so the caret is moved to the beginning of the current word
// in the original text
pub fn pop_word_in_place(original: &str, typed: &mut String) {
    let original_cutoff_len = original
        .char_indices()
        .nth(typed.chars().count())
        .map(|(i, _)| i)
        .unwrap_or(0);

    let last_word_or_whsp_char_count = original[0..original_cutoff_len]
        .split_word_bounds()
        .last()
        .map(|s| s.chars().count())
        .unwrap_or(0);

    for _ in 0..last_word_or_whsp_char_count {
        typed.pop();
    }
}

pub fn current_word(original: &str, typed_grapheme_count: usize) -> usize {
    original
        .graphemes(true)
        .take(typed_grapheme_count)
        .collect::<String>()
        .unicode_words()
        .count()
}

pub fn calculate_wpm(duration: Duration, typed: &str) -> f64 {
    let minutes: f64 = duration.as_secs_f64() / 60.;
    let words = typed.chars().count() / 5;

    words as f64 / minutes
}

pub fn calculate_accuracy(original: &str, typed: &str) -> f64 {
    let (correct, wrong) = zip(original.graphemes(true), typed.graphemes(true)).fold(
        (0, 0),
        |(correct, wrong), (og, tp)| {
            if og == tp {
                (correct + 1, wrong)
            } else {
                (correct, wrong + 1)
            }
        },
    );

    let total = correct + wrong;

    if total == 0 {
        0.
    } else {
        correct as f64 / total as f64
    }
}
