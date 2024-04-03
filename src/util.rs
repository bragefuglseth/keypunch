use unicode_segmentation::UnicodeSegmentation;

pub fn pop_grapheme(string: &mut String) {
    let mut v = string.graphemes(true).collect::<Vec<_>>();
    v.pop();
    *string = v.into_iter().collect()
}
