use super::*;

impl imp::KpTextView {
    pub(super) fn setup_color_scheme(&self) {
        let obj = self.obj();

        let text_view = self.text_view.get();
        let buf = text_view.buffer();

        // The subsequent color values were nonchalantly aquired by taking a screenshot of the
        // "Style classes" section of the Adwaita Demo app and using a
        // screen color picker to extract the needed colors. Each RGB value
        // has been divided by 256 to work as a floating point value.

        let tag_untyped = buf.create_tag(Some("untyped"), &[]).unwrap();
        tag_untyped.set_foreground_rgba(Some(&gdk::RGBA::new(0.547, 0.547, 0.547, 1.)));

        let tag_typed = buf.create_tag(Some("typed"), &[]).unwrap();
        tag_typed.set_foreground_rgba(Some(&gdk::RGBA::new(0.180, 0.180, 0.180, 1.)));

        let tag_mistake = buf.create_tag(Some("mistake"), &[]).unwrap();
        tag_mistake.set_foreground_rgba(Some(&gdk::RGBA::new(0.875, 0.105, 0.141, 1.)));
        tag_mistake.set_background_rgba(Some(&gdk::RGBA::new(0.965, 0.891, 0.895, 1.)));

        let tag_untyped_dark = buf.create_tag(Some("untyped-dark"), &[]).unwrap();
        tag_untyped_dark.set_foreground_rgba(Some(&gdk::RGBA::new(0.484, 0.484, 0.484, 1.)));

        let tag_typed_dark = buf.create_tag(Some("typed-dark"), &[]).unwrap();
        tag_typed_dark.set_foreground_rgba(Some(&gdk::RGBA::new(1., 1., 1., 1.)));

        let tag_mistake_dark = buf.create_tag(Some("mistake-dark"), &[]).unwrap();
        tag_mistake_dark.set_foreground_rgba(Some(&gdk::RGBA::new(0.961, 0.379, 0.316, 1.)));
        tag_mistake_dark.set_background_rgba(Some(&gdk::RGBA::new(0.223, 0.164, 0.156, 1.)));

        let style = adw::StyleManager::default();
        style.connect_dark_notify(glib::clone!(@weak obj => move |_| {
            obj.imp().update_colors();
        }));

        self.update_colors();
    }

    pub(super) fn update_colors(&self) {
        let obj = self.obj();

        let original = obj.original_text();
        let typed = obj.typed_text();
        let comparison = validate_with_whsp_markers(&original, &typed);

        let text_view = self.text_view.get();
        let buf = text_view.buffer();

        let style = adw::StyleManager::default();
        let (tag_untyped, tag_typed, tag_mistake, caret_color) = if style.is_dark() {
            ("untyped-dark", "typed-dark", "mistake-dark", (1., 1., 1.))
        } else {
            ("untyped", "typed", "mistake", (0.180, 0.180, 0.180))
        };

        // All tags are removed and reapplied on each recoloring
        // for the sake of keeping things simple
        buf.remove_all_tags(&buf.start_iter(), &buf.end_iter());

        // To color as little text as possible, we start 2 lines above
        // the currently active one (just enough to account for the scrolling animation)
        let typed_iter = buf.iter_at_offset(comparison.len() as i32);
        let mut color_start_iter = typed_iter.clone();
        text_view.backward_display_line(&mut color_start_iter);
        text_view.backward_display_line(&mut color_start_iter);
        text_view.backward_display_line_start(&mut color_start_iter);
        let color_start_offset = color_start_iter.offset();

        comparison
            .iter()
            .enumerate()
            .skip(color_start_offset as usize)
            .for_each(|(n, &correct)| {
                let start_iter = buf.iter_at_offset(n as i32);
                let end_iter = buf.iter_at_offset(n as i32 + 1);

                // Avoid applying the tag line breaks, which leads to some weird side effects
                // in the text view. Might be a GTK bug.
                if !start_iter.ends_line() {
                    let tag = if correct { tag_typed } else { tag_mistake };

                    buf.apply_tag_by_name(tag, &start_iter, &end_iter);
                }
            });

        // To color as little text as possible, we end the coloring 2 lines below
        // the currently active one (just enough to account for the scrolling animation)
        let mut color_end_iter = typed_iter.clone();
        text_view.forward_display_line(&mut color_end_iter);
        text_view.forward_display_line(&mut color_end_iter);
        text_view.forward_display_line_end(&mut color_end_iter);
        let color_end_offset = color_end_iter.offset();

        for n in comparison.len()..color_end_offset as usize {
            let start_iter = buf.iter_at_offset(n as i32);
            let end_iter = buf.iter_at_offset(n as i32 + 1);

            if !start_iter.ends_line() {
                buf.apply_tag_by_name(tag_untyped, &start_iter, &end_iter);
            }
        }

        self.caret_rgb.set(caret_color);
    }
}
