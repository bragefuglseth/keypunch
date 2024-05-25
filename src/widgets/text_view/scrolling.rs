use super::*;
use crate::text_utils::line_offset_with_replacements;
use unicode_segmentation::UnicodeSegmentation;

impl imp::KpTextView {
    pub(super) fn scroll_animation(&self) -> adw::TimedAnimation {
        self.scroll_animation
            .get_or_init(|| {
                let text_view = self.text_view.get();
                let vadjustment = self
                    .text_view
                    .vadjustment()
                    .expect("text view has vadjustment");

                adw::TimedAnimation::builder()
                    .duration(300)
                    .widget(&text_view)
                    .target(&adw::PropertyAnimationTarget::new(&vadjustment, "value"))
                    .build()
            })
            .clone()
    }

    // Updates the scroll position according to the text view and the length of the typed text so far.
    // If `force` is true, the change will be made unconditionally and without an animation.
    pub(super) fn update_scroll_position(&self, force: bool) {
        let obj = self.obj();

        let original = obj.original_text();
        let typed = obj.typed_text();
        // Validation is performed on typed text with one added character, to get the start index
        // of the next character.
        let (caret_line, caret_idx) =
            line_offset_with_replacements(&original, typed.graphemes(true).count());

        let text_view = self.text_view.get();

        let buf = text_view.buffer();
        let iter = buf
            .iter_at_line_index(caret_line as i32, caret_idx as i32)
            .unwrap_or(buf.end_iter());

        let location = text_view.iter_location(&iter);

        let y = if typed.is_empty() {
            0.
        } else {
            (location.y() + location.height() / 2)
                .checked_sub(obj.height() / 2)
                .unwrap_or(0) as f64
        };

        let current_position = self
            .text_view
            .vadjustment()
            .expect("text view always has vadjustment")
            .value();

        let scroll_animation = self.scroll_animation();
        if force {
            self.text_view
                .vadjustment()
                .expect("text view should have vadjustment")
                .set_value(y);
        } else {
            let line_has_changed = (scroll_animation.value_to() - y).abs() > 10.;

            if line_has_changed {
                scroll_animation.set_value_from(current_position);
                scroll_animation.set_value_to(y);
                scroll_animation.play();
            }
        }
    }
}

