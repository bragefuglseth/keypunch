use super::*;

impl imp::KpTextView {
    pub(super) fn update_accessible_state(&self) {
        let typed_len = self.typed_text.borrow().chars().count();

        let original_text = self.original_text.borrow();

        let current_word_start = original_text.chars().take(typed_len).enumerate().fold(
            0,
            |previous_index, (index, c)| {
                if c.is_whitespace() {
                    index + 1
                } else {
                    previous_index
                }
            },
        );

        let current_word: String = original_text
            .chars()
            .skip(current_word_start)
            .take_while(|c| !c.is_whitespace())
            .collect();

        let pos_in_current_word = typed_len.checked_sub(current_word_start).unwrap_or(0);
        if pos_in_current_word == 0 {
            let obj = self.obj();
            obj.update_property(&[gtk::accessible::Property::Label(&current_word)]);
        }
    }
}
