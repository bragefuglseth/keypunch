use super::*;

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

    pub(super) fn update_scroll_position(&self) {
        let obj = self.obj();

        let original = obj.original_text();
        let typed = obj.typed_text();
        let current_offset = validate_with_whsp_markers(&original, &typed).len();

        let text_view = self.text_view.get();

        let buffer = text_view.buffer();
        let mut iter = buffer.iter_at_offset(current_offset as i32);

        let mut line = 0;

        while text_view.backward_display_line(&mut iter) {
            line += 1;
        }

        if line != self.line.get() {
            self.line.set(line);
            self.animate_to_line(match line {
                0 | 1 => 0,
                num => (num - 1).try_into().unwrap(),
            });
        }
    }

    pub(super) fn animate_to_line(&self, line: usize) {
        let obj = self.obj();

        let text_view = self.text_view.get();
        let buffer = text_view.buffer();

        let mut iter = buffer.start_iter();
        for _ in 0..line + 1 {
            text_view.forward_display_line(&mut iter);
        }

        // To get the alignment to be proper, we have to calculate the y position
        // of the *vertical center* of the next line, and then subtract half of the
        // widget display height.

        let location = text_view.iter_location(&iter);
        let y = (location.y() + location.height() / 2)
            .checked_sub(obj.height() / 2)
            .unwrap_or(0);

        let current_position = self
            .text_view
            .vadjustment()
            .expect("text view always has vadjustment")
            .value();

        let scroll_animation = self.scroll_animation();
        scroll_animation.set_value_from(current_position);
        scroll_animation.set_value_to(y as f64);
        scroll_animation.play();
    }
}
