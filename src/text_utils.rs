use std::iter::zip;
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;

// Whitespace markers
pub const WHSP_MARKERS: [(&'static str, &'static str); 1] = [("\n", "↲\n")];

pub fn insert_whsp_markers(string: &str) -> String {
    let mut s = string.to_string();
    for entry in WHSP_MARKERS {
        s = s.replace(entry.0, entry.1);
    }
    s
}

// Gets the corresponding whitespace marker of a str, if any
pub fn whsp_marker(s: &str) -> Option<&'static str> {
    WHSP_MARKERS
        .iter()
        .find(|(whitespace, _)| *whitespace == s)
        .map(|(_, marker)| *marker)
}

// The returned tuples contain a "correct" boool, as well as the line number + start/end indices
// the bool applies to. We have to use the exact byte indices because GtkTextBuffer's `iter_at_offset()`
// function doesn't align perfectly with the `graphemes()` from the unicode_segmentation crate.
// The function accounts for whitespace markers.
pub fn validate_with_whsp_markers(original: &str, typed: &str) -> Vec<(bool, usize, usize, usize)> {
    original
        .split_inclusive("\n")
        .enumerate()
        .flat_map(|(line_num, line)| {
            line.grapheme_indices(true)
                .map(move |grapheme| (line_num, grapheme))
        })
        .zip(typed.graphemes(true))
        .map(
            |((line_num, (original_grapheme_idx, original_grapheme)), typed_grapheme)| {
                let correct = original_grapheme == typed_grapheme;
                let end_idx = if let Some(whsp_marker) = whsp_marker(original_grapheme) {
                    original_grapheme_idx + whsp_marker.trim().len()
                } else {
                    original_grapheme_idx + original_grapheme.len()
                };

                (correct, line_num, original_grapheme_idx, end_idx)
            },
        )
        .collect()
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

    correct as f64 / total as f64
}
