use super::*;
use crate::text_utils::{line_offset_with_replacements, GraphemeState};
use gtk::pango;
use unicode_segmentation::UnicodeSegmentation;

impl imp::KpTextView {
    pub(super) fn setup_color_scheme(&self) {
        let obj = self.obj();

        let text_view = self.text_view.get();
        let buf = text_view.buffer();

        // The subsequent color values were nonchalantly aquired by taking a screenshot of the
        // "Style classes" section of the Adwaita Demo app and using a
        // screen color picker to extract the needed colors. Each RGB value
        // has been divided by 256 to work as a floating point value.

        /////////////
        // Default //
        /////////////

        let tag_untyped = buf.create_tag(Some("untyped"), &[]).unwrap();
        tag_untyped.set_foreground_rgba(Some(&gdk::RGBA::new(0.547, 0.547, 0.547, 1.)));

        let tag_unfinished = buf.create_tag(Some("unfinished"), &[]).unwrap();
        tag_unfinished.set_foreground_rgba(Some(&gdk::RGBA::new(0.547, 0.547, 0.547, 1.)));
        tag_unfinished.set_underline(pango::Underline::Low);

        let tag_typed = buf.create_tag(Some("typed"), &[]).unwrap();
        tag_typed.set_foreground_rgba(Some(&gdk::RGBA::new(0.180, 0.180, 0.180, 1.)));

        let tag_mistake = buf.create_tag(Some("mistake"), &[]).unwrap();
        tag_mistake.set_foreground_rgba(Some(&gdk::RGBA::new(0.875, 0.105, 0.141, 1.)));
        tag_mistake.set_background_rgba(Some(&gdk::RGBA::new(0.965, 0.891, 0.895, 1.)));

        ///////////////////////////
        // Default high contrast //
        ///////////////////////////

        let tag_untyped_hc = buf.create_tag(Some("untyped-hc"), &[]).unwrap();
        tag_untyped_hc.set_foreground_rgba(Some(&gdk::RGBA::new(0.380, 0.380, 0.380, 1.)));

        let tag_unfinished_hc = buf.create_tag(Some("unfinished-hc"), &[]).unwrap();
        tag_unfinished_hc.set_foreground_rgba(Some(&gdk::RGBA::new(0.380, 0.380, 0.380, 1.)));
        tag_unfinished_hc.set_underline(pango::Underline::Low);

        let tag_typed_hc = buf.create_tag(Some("typed-hc"), &[]).unwrap();
        tag_typed_hc.set_foreground_rgba(Some(&gdk::RGBA::new(0., 0., 0., 1.)));

        let tag_mistake_hc = buf.create_tag(Some("mistake-hc"), &[]).unwrap();
        tag_mistake_hc.set_foreground_rgba(Some(&gdk::RGBA::new(0.976, 0.976, 0.976, 1.)));
        tag_mistake_hc.set_background_rgba(Some(&gdk::RGBA::new(0.875, 0.105, 0.141, 1.)));

        //////////
        // Dark //
        //////////

        let tag_untyped_dark = buf.create_tag(Some("untyped-dark"), &[]).unwrap();
        tag_untyped_dark.set_foreground_rgba(Some(&gdk::RGBA::new(0.484, 0.484, 0.484, 1.)));

        let tag_unfinished_dark = buf.create_tag(Some("unfinished-dark"), &[]).unwrap();
        tag_unfinished_dark.set_foreground_rgba(Some(&gdk::RGBA::new(0.484, 0.484, 0.484, 1.)));
        tag_unfinished_dark.set_underline(pango::Underline::Low);

        let tag_typed_dark = buf.create_tag(Some("typed-dark"), &[]).unwrap();
        tag_typed_dark.set_foreground_rgba(Some(&gdk::RGBA::new(1., 1., 1., 1.)));

        let tag_mistake_dark = buf.create_tag(Some("mistake-dark"), &[]).unwrap();
        tag_mistake_dark.set_foreground_rgba(Some(&gdk::RGBA::new(0.961, 0.379, 0.316, 1.)));
        tag_mistake_dark.set_background_rgba(Some(&gdk::RGBA::new(0.223, 0.164, 0.156, 1.)));

        ////////////////////////
        // Dark high contrast //
        ////////////////////////

        let tag_untyped_dark_hc = buf.create_tag(Some("untyped-dark-hc"), &[]).unwrap();
        tag_untyped_dark_hc.set_foreground_rgba(Some(&gdk::RGBA::new(0.600, 0.600, 0.600, 1.)));

        let tag_unfinished_dark_hc = buf.create_tag(Some("unfinished-dark-hc"), &[]).unwrap();
        tag_unfinished_dark_hc.set_foreground_rgba(Some(&gdk::RGBA::new(0.600, 0.600, 0.600, 1.)));
        tag_unfinished_dark_hc.set_underline(pango::Underline::Low);

        let tag_typed_dark_hc = buf.create_tag(Some("typed-dark-hc"), &[]).unwrap();
        tag_typed_dark_hc.set_foreground_rgba(Some(&gdk::RGBA::new(1., 1., 1., 1.)));

        let tag_mistake_dark_hc = buf.create_tag(Some("mistake-dark-hc"), &[]).unwrap();
        tag_mistake_dark_hc.set_foreground_rgba(Some(&gdk::RGBA::new(0.140, 0.140, 0.140, 1.)));
        tag_mistake_dark_hc.set_background_rgba(Some(&gdk::RGBA::new(0.961, 0.379, 0.316, 1.)));

        let style = adw::StyleManager::default();
        style.connect_dark_notify(glib::clone!(@weak obj => move |_| {
            obj.imp().update_colors();
        }));
        style.connect_high_contrast_notify(glib::clone!(@weak obj => move |_| {
            obj.imp().update_colors();
        }));

        self.update_colors();
    }

    pub(super) fn update_colors(&self) {
        let original = self.original_text.borrow();
        let typed = self.typed_text.borrow();

        let input_context = self.input_context.borrow();
        let (preedit, _, _) = input_context.as_ref().unwrap().preedit_string();

        let text_view = self.text_view.get();
        let buf = text_view.buffer();

        let style = adw::StyleManager::default();
        let (tag_untyped, tag_unfinished, tag_typed, tag_mistake, caret_color) =
            match (style.is_dark(), style.is_high_contrast()) {
                (false, false) => (
                    "untyped",
                    "unfinished",
                    "typed",
                    "mistake",
                    (0.180, 0.180, 0.180),
                ),
                (false, true) => (
                    "untyped-hc",
                    "unfinished-hc",
                    "typed-hc",
                    "mistake-hc",
                    (0.180, 0.180, 0.180),
                ),
                (true, false) => (
                    "untyped-dark",
                    "unfinished-dark",
                    "typed-dark",
                    "mistake-dark",
                    (1., 1., 1.),
                ),
                (true, true) => (
                    "untyped-dark-hc",
                    "unfinished-dark-hc",
                    "typed-dark-hc",
                    "mistake-dark-hc",
                    (1., 1., 1.),
                ),
            };

        // All tags are removed and reapplied on each recoloring
        // for the sake of keeping things simple
        buf.remove_all_tags(&buf.start_iter(), &buf.end_iter());

        let (typed_line, typed_offset) =
            line_offset_with_replacements(&original, &typed, preedit.graphemes(true).count());
        let typed_iter = buf
            .iter_at_line_index(typed_line as i32, typed_offset as i32)
            .unwrap_or(buf.end_iter());

        // To color as little text as possible, we start 2 lines above
        // the currently active one (just enough to account for the scrolling animation in all cases)
        let mut color_start_iter = typed_iter.clone();
        text_view.backward_display_line(&mut color_start_iter);
        text_view.backward_display_line(&mut color_start_iter);
        text_view.backward_display_line(&mut color_start_iter);
        text_view.backward_display_line_start(&mut color_start_iter);

        let color_start_offset = buf
            .text(&buf.start_iter(), &color_start_iter, true)
            .graphemes(true)
            .count();

        let comparison =
            validate_with_replacements(&original, &typed, preedit.as_str().graphemes(true).count());

        comparison
            .iter()
            .skip(color_start_offset as usize)
            .for_each(|(state, line, start_idx, end_idx)| {
                let start_iter_option = buf.iter_at_line_index(*line as i32, *start_idx as i32);
                let end_iter_option = buf.iter_at_line_index(*line as i32, *end_idx as i32);

                if let (Some(start_iter), Some(end_iter)) = (start_iter_option, end_iter_option) {
                    // Avoid applying the tag to line breaks, which leads to some weird side effects
                    // in the text view. Might be a GTK bug.
                    if !start_iter.ends_line() {
                        let tag = match *state {
                            GraphemeState::Correct => tag_typed,
                            GraphemeState::Unfinished => tag_unfinished,
                            GraphemeState::Mistake => tag_mistake,
                        };

                        buf.apply_tag_by_name(tag, &start_iter, &end_iter);
                    }
                }
            });

        // To color as little text as possible, we end the coloring 2 lines below
        // the currently active one (just enough to account for the scrolling animation in all cases)
        let mut color_end_iter = typed_iter.clone();
        text_view.forward_display_line(&mut color_end_iter);
        text_view.forward_display_line(&mut color_end_iter);
        text_view.forward_display_line(&mut color_end_iter);
        text_view.forward_display_line_end(&mut color_end_iter);
        let color_end_offset = color_end_iter.offset();

        for n in typed_iter.offset()..color_end_offset {
            let start_iter = buf.iter_at_offset(n as i32);
            let end_iter = buf.iter_at_offset(n as i32 + 1);

            if !start_iter.ends_line() {
                buf.apply_tag_by_name(tag_untyped, &start_iter, &end_iter);
            }
        }

        self.caret_rgb.set(caret_color);
    }
}
