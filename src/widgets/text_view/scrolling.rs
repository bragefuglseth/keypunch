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

    pub(super) fn update_scroll_position(&self, force: bool) {
        let obj = self.obj();

        let original = obj.original_text();
        let typed = obj.typed_text();
        let current_offset = validate_with_whsp_markers(&original, &typed).len();

        let text_view = self.text_view.get();

        let buffer = text_view.buffer();
        let  iter = buffer.iter_at_offset(current_offset as i32);

        let location = text_view.iter_location(&iter);
        let y = (location.y() + location.height() / 2)
            .checked_sub(obj.height() / 2)
            .unwrap_or(0)
            as f64;

        let current_position = self
            .text_view
            .vadjustment()
            .expect("text view always has vadjustment")
            .value();

        let scroll_animation = self.scroll_animation();
        if force {
            self.text_view.vadjustment().expect("text view has vadjustment").set_value(y);
        } else if y != scroll_animation.value_to() {
            scroll_animation.set_value_from(current_position);
            scroll_animation.set_value_to(y);
            scroll_animation.play();
        }
    }
}
