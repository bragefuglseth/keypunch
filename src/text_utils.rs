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

// Get the corresponding whitespace marker
pub fn whsp_marker(s: &str) -> Option<&'static str> {
    WHSP_MARKERS
        .iter()
        .find(|(whitespace, _)| *whitespace == s)
        .map(|(_, marker)| *marker)
}

pub fn validate_with_whsp_markers(original: &str, typed: &str) -> Vec<bool> {
    zip(original.graphemes(true), typed.graphemes(true))
        .map(|(og, tg)| {
            let matches = og == tg;
            if let Some(marker) = whsp_marker(og) {
                vec![matches; marker.graphemes(true).count()]
            } else {
                vec![matches]
            }
        })
        .flatten()
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
