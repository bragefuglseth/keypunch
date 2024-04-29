use super::*;
use crate::text_generation::CHUNK_GRAPHEME_COUNT;

impl imp::KpTextView {
    pub(super) fn setup_color_scheme(&self) {
        let obj = self.obj();

        let text_view = self.text_view.get();
        let tag_table = text_view.buffer().tag_table();

        let tag_untyped = gtk::TextTag::new(Some("untyped"));
        let tag_typed = gtk::TextTag::new(Some("typed"));
        let tag_mistake = gtk::TextTag::new(Some("mistake"));
        let tag_untyped_dark = gtk::TextTag::new(Some("untyped-dark"));
        let tag_typed_dark = gtk::TextTag::new(Some("typed-dark"));
        let tag_mistake_dark = gtk::TextTag::new(Some("mistake-dark"));

        // The subsequent color values were aquired by taking a screenshot of the
        // "Style classes" section of the Adwaita Demo app and using a
        // screen color picker to extract the needed colors. Each RGB value
        // has been divided by 256 to work as a floating point value.

        tag_untyped.set_foreground_rgba(Some(&gdk::RGBA::new(0.547, 0.547, 0.547, 1.)));
        tag_typed.set_foreground_rgba(Some(&gdk::RGBA::new(0.180, 0.180, 0.180, 1.)));
        tag_mistake.set_foreground_rgba(Some(&gdk::RGBA::new(0.875, 0.105, 0.141, 1.)));
        tag_mistake.set_background_rgba(Some(&gdk::RGBA::new(0.965, 0.891, 0.895, 1.)));

        tag_untyped_dark.set_foreground_rgba(Some(&gdk::RGBA::new(0.484, 0.484, 0.484, 1.)));
        tag_typed_dark.set_foreground_rgba(Some(&gdk::RGBA::new(1., 1., 1., 1.)));
        tag_mistake_dark.set_foreground_rgba(Some(&gdk::RGBA::new(0.961, 0.379, 0.316, 1.)));
        tag_mistake_dark.set_background_rgba(Some(&gdk::RGBA::new(0.223, 0.164, 0.156, 1.)));

        tag_table.add(&tag_untyped);
        tag_table.add(&tag_typed);
        tag_table.add(&tag_mistake);
        tag_table.add(&tag_untyped_dark);
        tag_table.add(&tag_typed_dark);
        tag_table.add(&tag_mistake_dark);

        let style = adw::StyleManager::default();
        style.connect_dark_notify(glib::clone!(@weak obj => move |_| {
            obj.imp().update_text_styling();
        }));

        self.update_text_styling();
    }

    pub(super) fn update_text_styling(&self) {
        let obj = self.obj();
        let original = obj.original_text();
        let typed = obj.typed_text();

        let comparison = validate_with_whsp_markers(&original, &typed);

        let text_view = self.text_view.get();
        let buffer = text_view.buffer();
        let start_iter = buffer.start_iter();
        let typed_iter = buffer.iter_at_offset(comparison.len() as i32);
        let end_iter = buffer.end_iter();
        buffer.remove_all_tags(&start_iter, &end_iter);

        let style = adw::StyleManager::default();

        let (tag_untyped, tag_typed, tag_mistake, caret_color) = if style.is_dark() {
            ("untyped-dark", "typed-dark", "mistake-dark", (1., 1., 1.))
        } else {
            ("untyped", "typed", "mistake", (0.180, 0.180, 0.180))
        };

        buffer.apply_tag_by_name(tag_untyped, &start_iter, &end_iter);
        buffer.apply_tag_by_name(tag_typed, &start_iter, &typed_iter);

        comparison
            .iter()
            .enumerate()
            .skip(comparison.len().checked_sub(CHUNK_GRAPHEME_COUNT).unwrap_or(0))
            .filter(|(_, &correctly_typed)| !correctly_typed)
            .map(|(n, _)| n)
            .for_each(|n| {
                let mistake_start = buffer.iter_at_offset(n as i32);
                let mistake_end = buffer.iter_at_offset(n as i32 + 1);
                buffer.apply_tag_by_name(tag_mistake, &mistake_start, &mistake_end);
            });

        self.caret_rgb.set(caret_color);
    }
}
