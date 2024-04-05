use super::*;

impl imp::RcwTextView {
    fn scroll_animation(&self) -> adw::TimedAnimation {
        self.scroll_animation
            .get_or_init(|| {
                let obj = self.obj().to_owned();

                adw::TimedAnimation::builder()
                    .duration(300)
                    .widget(&obj)
                    .target(&adw::PropertyAnimationTarget::new(&obj, "scroll-position"))
                    .build()
            })
            .clone()
    }

    pub(super) fn caret_x_animation(&self) -> adw::TimedAnimation {
        self.caret_x_animation
            .get_or_init(|| {
                let obj = self.obj().to_owned();

                adw::TimedAnimation::builder()
                    .duration(150)
                    .widget(&obj)
                    .target(&adw::PropertyAnimationTarget::new(&obj, "caret-x"))
                    .build()
            })
            .clone()
    }

    pub(super) fn caret_y_animation(&self) -> adw::TimedAnimation {
        self.caret_y_animation
            .get_or_init(|| {
                let obj = self.obj().to_owned();

                adw::TimedAnimation::builder()
                    .duration(150)
                    .widget(&obj)
                    .target(&adw::PropertyAnimationTarget::new(&obj, "caret-y"))
                    .build()
            })
            .clone()
    }

    pub(super) fn animate_to_line(&self, line: i32) {
        let scroll_animation = self.scroll_animation();

        let current_position = self.obj().scroll_position();

        scroll_animation.set_value_from(current_position);
        scroll_animation.set_value_to((line * LINE_HEIGHT) as f64);

        scroll_animation.play();
    }
}
