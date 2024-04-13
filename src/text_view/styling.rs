use super::*;

#[derive(Default, Clone, Copy)]
pub(super) struct TextViewColorScheme {
    pub untyped: (u16, u16, u16),
    pub typed: (u16, u16, u16),
    pub mistake: (u16, u16, u16),
    pub mistake_bg: (u16, u16, u16),
    pub caret: (f32, f32, f32),
}

const COLOR_SCHEME_LIGHT: TextViewColorScheme = TextViewColorScheme {
    untyped: (41472, 41472, 41472),
    typed: (12800, 12800, 12800),
    mistake: (49152, 7168, 10240),
    mistake_bg: (63232, 58368, 58624),
    caret: (0.2, 0.2, 0.2),
};

const COLOR_SCHEME_DARK: TextViewColorScheme = TextViewColorScheme {
    untyped: (33792, 33792, 33792),
    typed: (65280, 65280, 65280),
    mistake: (65280, 31488, 25344),
    mistake_bg: (14592, 10752, 10240),
    caret: (1., 1., 1.),
};

impl imp::KpTextView {
    pub(super) fn setup_color_scheme(&self) {
        let obj = self.obj();
        let style = adw::StyleManager::default();
        style.connect_dark_notify(glib::clone!(@weak obj => move |_| {
            obj.imp().update_color_scheme();
        }));

        self.update_color_scheme();
    }

    pub(super) fn update_color_scheme(&self) {
        let style = adw::StyleManager::default();

        self.color_scheme.set(if style.is_dark() {
            COLOR_SCHEME_DARK
        } else {
            COLOR_SCHEME_LIGHT
        });

        self.update_text_styling();
    }

    pub(super) fn update_text_styling(&self) {
        let clr = self.color_scheme.get();

        let original = self.obj().original_text();
        let typed = self.obj().typed_text();

        let comparison = validate_with_whsp_markers(&original, &typed);

        let attr_list = pango::AttrList::new();

        let untyped_attr =
            pango::AttrColor::new_foreground(clr.untyped.0, clr.untyped.1, clr.untyped.2);
        attr_list.insert(untyped_attr);

        let mut typed_attr =
            pango::AttrColor::new_foreground(clr.typed.0, clr.typed.1, clr.typed.2);
        typed_attr.set_start_index(0);
        typed_attr.set_end_index(comparison.len() as u32);
        attr_list.insert(typed_attr);

        comparison
            .iter()
            .enumerate()
            .filter(|(_, &correctly_typed)| !correctly_typed)
            .map(|(n, _)| {
                let mut mistake_fg_attr =
                    pango::AttrColor::new_foreground(clr.mistake.0, clr.mistake.1, clr.mistake.2);
                mistake_fg_attr.set_start_index(n as u32);
                mistake_fg_attr.set_end_index(n as u32 + 1);

                let mut mistake_bg_attr =
                    pango::AttrColor::new_background(clr.mistake_bg.0, clr.mistake_bg.1, clr.mistake_bg.2);
                mistake_bg_attr.set_start_index(n as u32);
                mistake_bg_attr.set_end_index(n as u32 + 1);

                (mistake_fg_attr, mistake_bg_attr)
            })
            .for_each(|(attr_1, attr_2)| {
                attr_list.insert(attr_1);
                attr_list.insert(attr_2);
            });

        self.label.set_attributes(Some(&attr_list));
    }
}
