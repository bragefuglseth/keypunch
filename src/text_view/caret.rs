use super::*;

impl imp::RcwTextView {
    pub(super) fn update_caret_position(&self) {
        let session = self.typing_session.borrow();
        let current_index = session.validate_with_whsp_markers().len();

        let layout = self.label.get().layout();
        let layout_width = layout.width() / pango::SCALE;

        let (line_index, ltr_x) = layout.index_to_line_x(current_index as i32, false);
        let ltr_x = ltr_x / pango::SCALE;

        let line_width = layout
            .line(line_index)
            .expect("line exists at index")
            .extents()
            .1
            .width()
            / pango::SCALE;

        let line_direction = layout
            .line(line_index)
            .expect("line exists at index")
            .resolved_direction();

        let x = if line_direction == pango::Direction::Rtl {
            layout_width - line_width + ltr_x
        } else {
            ltr_x
        };

        let x = if x == 0 {
            1
        } else if x == layout_width {
            layout_width - 1
        } else {
            x
        };

        let reference_line = if line_index == 0 { 0 } else { 1 };
        let line_start_index = layout
            .line(reference_line)
            .map(|l| l.start_index())
            .unwrap_or(0);
        let y = layout.index_to_pos(line_start_index).y() / pango::SCALE;

        let obj = self.obj();

        let caret_x_animation = self.caret_x_animation();
        caret_x_animation.set_value_from(obj.caret_x());
        caret_x_animation.set_value_to(x as f64);
        caret_x_animation.play();

        let caret_y_animation = self.caret_y_animation();
        caret_y_animation.set_value_from(obj.caret_y());
        caret_y_animation.set_value_to(y as f64);
        caret_y_animation.play();

        let caret_rect = gdk::Rectangle::new(x, y, 1, layout.baseline() / pango::SCALE + 2);

        if let Some(input_context) = &*self.input_context.borrow() {
            input_context.set_cursor_location(&caret_rect);
        }
    }
}
