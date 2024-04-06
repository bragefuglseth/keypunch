use crate::util::WHSP_MARKERS;
use std::cell::RefCell;
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
enum SessionType {
    #[default]
    LengthBased,
    TimeBased(Duration),
}

#[derive(Default)]
pub struct TypingSession {
    session_type: SessionType,
    original_text: String,
    typed_text: RefCell<String>,
}

impl TypingSession {
    pub fn new(original_text: String) -> Self {
        TypingSession {
            original_text,

            ..Default::default()
        }
    }

    pub fn original_text(&self) -> &str {
        &self.original_text
    }

    pub fn typed_text_len(&self) -> usize {
        self.typed_text.borrow().len()
    }

    pub fn push_to_typed_text(&self, s: &str) {
        self.typed_text.borrow_mut().push_str(s);
    }

    pub fn pop_typed_text(&self) {
        let mut typed_text = self.typed_text.borrow_mut();
        let mut v = typed_text.graphemes(true).collect::<Vec<_>>();
        v.pop();
        *typed_text = v.into_iter().collect();
    }

    pub fn validate_with_whsp_markers(&self) -> Vec<bool> {
        let original = &self.original_text;
        let typed = self.typed_text.borrow();

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
}
