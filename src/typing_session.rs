use crate::util::{insert_whsp_markers, WHSP_MARKERS};
use std::cell::RefCell;
use std::rc::Rc;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct TypingSession {
    pub original_text: String,

    typed_text: Rc<RefCell<String>>,
}

impl TypingSession {
    pub fn new(original_text: String) -> Self {
        TypingSession {
            original_text,

            ..Self::default()
        }
    }

    pub fn typed_text(&self) -> Rc<RefCell<String>> {
        Rc::clone(&self.typed_text)
    }

    pub fn typed_text_len(&self) -> usize {
        self.typed_text.borrow().len()
    }

    pub fn typed_text_len_whsp_markers(&self) -> usize {
        insert_whsp_markers(&self.typed_text.borrow()).len()
    }

    // we'll have to see what we do with this. for actual post-session validation,
    // we need to match graphemes anyways.
    pub fn validate(&self) -> Vec<bool> {
        let original = &self.original_text;
        let typed = self.typed_text.borrow();

        original
            .graphemes(true)
            .zip(typed.graphemes(true))
            .map(|(og, tg)| {
                if og == tg {
                    vec![true; og.len()]
                } else {
                    vec![false; og.len()]
                }
            })
            .flatten()
            .collect()
    }

    // Take white space markers into account when validating.
    pub fn validate_with_whsp_markers(&self) -> Vec<bool> {
        let original = &self.original_text;
        let typed = self.typed_text.borrow();

        original
            .graphemes(true)
            .zip(typed.graphemes(true))
            .map(|(og, tg)| {
                if og == tg {
                    // check if the typed grapheme exists in the whitespace char database
                    // used by the text view. if that's the case, match the length of the
                    // indicator used.
                    if let Some((_, val)) = WHSP_MARKERS.iter().find(|(key, _)| *key == tg) {
                        vec![true; val.len()]
                    } else {
                        vec![true; og.len()]
                    }
                } else {
                    // match the length of the grapheme that was supposed to be typed,
                    // so the error coloring works
                    vec![false; og.len()]
                }
            })
            .flatten()
            .collect()
    }
}
