use super::*;

impl imp::KpTextView {
    pub(super) fn set_scroll_position(&self, line_number: f64) {
        self.scroll_position.set(line_number);
        self.obj().queue_allocate();
    }

    pub(super) fn scroll_animation(&self) -> adw::TimedAnimation {
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

    pub(super) fn update_scroll_position(&self) {
        let original = self.obj().original_text();
        let typed = self.obj().typed_text();

        let (line, _) = self.label.layout().index_to_line_x(
            validate_with_whsp_markers(&original, &typed).len() as i32,
            false,
        );

        if line != self.line.get() {
            self.line.set(line);
            self.animate_to_line(match line {
                0 | 1 => 0,
                num => num.try_into().unwrap(),
            });
        }
    }

    pub(super) fn animate_to_line(&self, line: usize) {
        let y: i32 = self
            .label
            .layout()
            .lines()
            .iter()
            .take(line.checked_sub(1).unwrap_or(0))
            .map(|l| l.extents().1.height() / pango::SCALE)
            .sum();

        let scroll_animation = self.scroll_animation();

        let current_position = self.obj().scroll_position();

        scroll_animation.set_value_from(current_position);
        scroll_animation.set_value_to(y as f64);

        scroll_animation.play();
    }
}
