use super::*;

impl imp::KpTextView {
    pub(super) fn set_caret_x(&self, caret_x: f64) {
        self.caret_x.set(caret_x);
        self.obj().queue_draw();
    }

    pub(super) fn set_caret_y(&self, caret_y: f64) {
        self.caret_y.set(caret_y);
        self.obj().queue_draw();
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

    pub(super) fn caret_stroke_data(&self) -> (gsk::Path, gsk::Stroke, gdk::RGBA) {
        let obj = self.obj();
        let (r, g, b) = self.caret_rgb.get();
        let caret_x = obj.caret_x() as f32;
        let caret_y = obj.caret_y() as f32;
        let caret_height = obj.caret_height() as f32;
        let path_builder = gsk::PathBuilder::new();

        path_builder.move_to(caret_x, caret_y + caret_height);
        path_builder.line_to(caret_x, caret_y);

        let path = path_builder.to_path();

        let stroke = gsk::Stroke::new(1.);

        let color = gdk::RGBA::new(r, g, b, 1.);

        (path, stroke, color)
    }

    pub(super) fn update_caret_position(&self) {
        // Calculates where the caret currently should be,
        // and runs an animation to get it there

        let obj = self.obj();

        let original = obj.original_text();
        let typed = obj.typed_text();
        let current_offset = validate_with_whsp_markers(&original, &typed).len();

        let text_view = self.text_view.get();
        let buffer = text_view.buffer();

        // Calculate x position
        let caret_iter = buffer.iter_at_offset(current_offset as i32);
        let (pos, _) = text_view.cursor_locations(Some(&caret_iter));
        let (mut x, _) =
            text_view.buffer_to_window_coords(gtk::TextWindowType::Widget, pos.x(), pos.y());

        let width = obj.width();
        if x == 0 || x == width {
            let start_iter = buffer.start_iter();
            let start_is_rtl = text_view.iter_location(&start_iter).x() > 0;

            x = match (self.running.get(), start_is_rtl) {
                (false, false) => -10,
                (false, true) => width + 10,
                (true, false) => 1,
                (true, true) => width - 1,
            };
        }

        // Calculate y position

        // If we can't move the iter backwards one display line, that must mean
        // we're at the first one
        let is_first_line = !text_view.backward_display_line(&mut caret_iter.clone());

        let y = if is_first_line {
            text_view.cursor_locations(Some(&buffer.start_iter())).1.y()
        } else {
            let mut line_1_iter = buffer.start_iter();
            text_view.forward_display_line(&mut line_1_iter);
            text_view.cursor_locations(Some(&line_1_iter)).1.y()
        };

        self.caret_height.set(pos.height() as f64);

        let caret_x_animation = self.caret_x_animation();
        caret_x_animation.set_value_from(obj.caret_x());
        caret_x_animation.set_value_to(x as f64);
        caret_x_animation.play();

        let caret_y_animation = self.caret_y_animation();
        caret_y_animation.set_value_from(obj.caret_y());
        caret_y_animation.set_value_to(y as f64);
        caret_y_animation.play();

        // Update virtual caret to accomodate software input methods (e.g. Pinyin)
        if let Some(input_context) = &*self.input_context.borrow() {
            let caret_rect = gdk::Rectangle::new(x, y, 1, pos.height());
            input_context.set_cursor_location(&caret_rect);
            input_context.reset();
        }
    }
}
