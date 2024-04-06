use super::*;

impl imp::RcwTextView {
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
        let caret_x = obj.caret_x() as f32;
        let caret_y = obj.caret_y() as f32;
        let path_builder = gsk::PathBuilder::new();
        path_builder.move_to(caret_x, caret_y);
        path_builder.line_to(
            caret_x,
            caret_y + (self.label.layout().baseline() / pango::SCALE) as f32 + 2.,
        );
        let path = path_builder.to_path();

        let stroke = gsk::Stroke::new(1.);

        let clr = self.color_scheme.get();
        let color = gdk::RGBA::new(clr.caret.0, clr.caret.1, clr.caret.2, 1.);

        (path, stroke, color)
    }

    pub(super) fn update_caret_position(&self) {
        let session = self.typing_session.borrow();
        let current_index = session.validate_with_whsp_markers().len();

        let layout = self.label.get().layout();
        let layout_width = layout.width() / pango::SCALE;

        // Find `x`
        let (line_index, ltr_x) = layout.index_to_line_x(current_index as i32, false);
        let ltr_x = ltr_x / pango::SCALE; // X value measured from the left edge of the line's area

        let line = layout.line(line_index).expect("line exists at index");

        let line_width = line.extents().1.width() / pango::SCALE;

        let line_direction = line.resolved_direction();

        let x = if line_direction == pango::Direction::Rtl {
            layout_width - line_width + ltr_x
        } else {
            ltr_x
        };

        // Some x overrides
        let x = if x == layout_width
            && current_index as i32 > line.start_index()
            && line_direction == pango::Direction::Rtl
        {
            layout_width - line_width // fix RTL quirk where cursor jumps back to the right edge of the view after last grapheme
        } else if current_index == 0 && line_direction == pango::Direction::Rtl {
            layout_width + 1 // hide caret when no text has been entered (RTL)
        } else if current_index == 0 {
            -1 // hide caret when no text has been entered
        } else if x == 0 {
            1 // avoid clipping the caret at the view edge
        } else if x == layout_width {
            layout_width - 1 // avoid cliping the caret at the view edge
        } else {
            x
        };

        // Find `y`
        // First, check which of the visible lines the caret is being drawn on
        let reference_line = layout
            .line(if line_index == 0 { 0 } else { 1 })
            .expect("requested line exists");
        let y = layout.index_to_pos(reference_line.start_index()).y() / pango::SCALE;

        let obj = self.obj();

        let caret_x_animation = self.caret_x_animation();
        caret_x_animation.set_value_from(obj.caret_x());
        caret_x_animation.set_value_to(x as f64);
        caret_x_animation.play();

        let caret_y_animation = self.caret_y_animation();
        caret_y_animation.set_value_from(obj.caret_y());
        caret_y_animation.set_value_to(y as f64);
        caret_y_animation.play();

        // Update virtual caret to accomodate software input methods (e.g. Pinyin)
        let caret_rect = gdk::Rectangle::new(x, y, 1, layout.baseline() / pango::SCALE + 2);
        if let Some(input_context) = &*self.input_context.borrow() {
            input_context.set_cursor_location(&caret_rect);
        }
    }
}
