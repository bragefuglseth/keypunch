use std::iter::zip;
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;

// String replacements for when text is displayed in the text view
const REPLACEMENTS: &'static [(&'static str, &'static str)] = &[
    ("\n", "↲\n"),            // Visually indicate enter
    ("\u{0020}", "\u{2004}"), // Use a fixed-width space to avoid shifting widths with Arabic
];

#[derive(Clone, Copy, PartialEq)]
pub enum GraphemeState {
    Correct,
    Unfinished,
    Mistake,
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

// The returned tuples contain a "correct" bool, as well as the line number + start/end indices
// the bool applies to. We have to use the exact byte indices because GtkTextBuffer's `iter_at_offset()`
// function doesn't align perfectly with `graphemes()` from the unicode_segmentation crate.
// The function accounts for replacements.
pub fn validate_with_replacements(
    original: &str,
    typed: &str,
) -> Vec<(GraphemeState, usize, usize, usize)> {
    let last_typed_grapheme_offset = typed.graphemes(true).count().checked_sub(1).unwrap_or(0);

    original
        .split_inclusive("\n")
        .enumerate()
        .flat_map(|(line_num, line)| {
            line.grapheme_indices(true)
                .map(move |grapheme| (line_num, grapheme))
        })
        .zip(typed.graphemes(true))
        .enumerate()
        .scan(
            (0, 0),
            |(accumulator_line, accumulated_offset),
             (offset, ((line_num, (original_grapheme_idx, original_grapheme)), typed_grapheme))| {
                let state = if original_grapheme == typed_grapheme {
                    GraphemeState::Correct
                } else if original_grapheme.starts_with(typed_grapheme) && offset == last_typed_grapheme_offset {
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

// Get the line / byte offset of a char, taking replacements into account. This
// function is used to e.g. position the caret
pub fn line_offset_with_replacements(original: &str, grapheme_idx: usize) -> (usize, usize) {
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

pub fn pop_grapheme(s: &str) -> String {
    let mut v = s.graphemes(true).collect::<Vec<_>>();
    v.pop();
    v.into_iter().collect()
}

pub fn calculate_wpm(duration: Duration, typed: &str) -> f64 {
    let minutes: f64 = duration.as_secs_f64() / 60.;
    let words = typed.graphemes(true).count() / 5;

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
