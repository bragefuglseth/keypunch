use std::cell::RefCell;
use std::rc::Rc;
use crate::util::insert_whsp_markers;

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

    pub fn validate(&self) -> Vec<bool> {
        validate(&self.original_text, &self.typed_text.borrow())
    }

    pub fn validate_with_whsp_markers(&self) -> Vec<bool> {
        validate(&insert_whsp_markers(&self.original_text), &insert_whsp_markers(&self.typed_text.borrow()))
    }
}

pub fn validate(s1: &str, s2: &str) -> Vec<bool> {
    s1.bytes()
        .zip(s2.bytes())
        .map(|(c1, c2)| c1 == c2)
        .collect()
}
