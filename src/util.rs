use unicode_segmentation::UnicodeSegmentation;

pub fn pop_grapheme(string: &mut String) {
    let mut v = string.graphemes(true).collect::<Vec<_>>();
    v.pop();
    *string = v.into_iter().collect()
}

pub const WHSP_MARKERS: [(&'static str, &'static str); 1] = [("\n", "↲\n")];

pub fn insert_whsp_markers(string: &str) -> String {
    let mut s = string.to_string();
    for entry in WHSP_MARKERS {
        s = s.replace(entry.0, entry.1);
    }
    s
}
