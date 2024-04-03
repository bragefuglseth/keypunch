use std::cell::RefCell;
use std::rc::Rc;

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

    pub fn validate(&self) -> Vec<bool> {
        self.original_text.bytes()
            .zip(self.typed_text().borrow().bytes())
            .map(|(c1, c2)| c1 == c2)
            .collect()
    }
}
