use unicode_segmentation::UnicodeSegmentation;
pub const WHSP_MARKERS: [(&'static str, &'static str); 1] = [("\n", "↲\n")];

pub fn insert_whsp_markers(string: &str) -> String {
    let mut s = string.to_string();
    for entry in WHSP_MARKERS {
        s = s.replace(entry.0, entry.1);
    }
    s
}

pub fn validate_with_whsp_markers(original: &str, typed: &str) -> Vec<bool> {
    original
        .graphemes(true)
        .zip(typed.graphemes(true))
        .map(|(og, tg)| {
            let matches = og == tg;
            // check if the typed grapheme exists in the whitespace char database
            // used by the text view. if that's the case, match the length of the
            // indicator used.
            if let Some((_, val)) = WHSP_MARKERS.iter().find(|(key, _)| *key == og) {
                vec![matches; val.len()]
            } else {
                vec![matches; og.len()]
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
