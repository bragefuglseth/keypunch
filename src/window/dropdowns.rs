use super::*;

impl imp::KpWindow {
    pub(super) fn setup_dropdowns(&self) {
        let mode_model = gtk::StringList::new(&["Simple", "Advanced", "Custom"]);
        let mode_dropdown = self.mode_dropdown.get();
        mode_dropdown.set_model(Some(&mode_model));
        mode_dropdown.connect_selected_item_notify(glib::clone!(@weak self as imp => move |_| {
            imp.update_original_text();
            imp.focus_text_view();
        }));

        let time_model = gtk::StringList::new(&[
            "15 seconds",
            "30 seconds",
            "1 minute",
            "5 minutes",
            "10 minutes",
        ]);
        let time_dropdown = self.time_dropdown.get();
        time_dropdown.set_model(Some(&time_model));
        time_dropdown.connect_selected_item_notify(glib::clone!(@weak self as imp => move |_| {
            imp.update_time();
            imp.focus_text_view();
        }));
    }

    pub(super) fn update_original_text(&self) {
        let mode_string = self
            .mode_dropdown
            .selected_item()
            .expect("dropdowns have been set up")
            .downcast_ref::<gtk::StringObject>()
            .expect("dropdown contains string items")
            .string();

        let text_type = match mode_string.as_str() {
            "Simple" => TextType::Simple,
            "Advanced" => TextType::Advanced,
            "Custom" => TextType::Custom,
            _ => panic!("invalid mode selected in dropdown"),
        };

        let text = match text_type {
            TextType::Simple => "lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore et dolore magnam aliquam quaerat voluptatem ut enim aeque doleamus animo cum corpore dolemus fieri tamen permagna accessio potest si aliquod aeternum et infinitum impendere malum nobis opinemur quod idem licet transferre in voluptatem ut",
            TextType::Advanced => "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magnam aliquam quaerat voluptatem. Ut enim aeque doleamus animo, cum corpore dolemus, fieri tamen permagna accessio potest, si aliquod aeternum et infinitum impendere malum nobis opinemur. Quod idem licet transferre in voluptatem, ut.",
            TextType::Custom => "The quick, brown fox jumped over the lazy dog.",
        };

        self.text_type.set(text_type);
        self.text_view.set_original_text(text);
    }

    pub(super) fn update_time(&self) {
        let time_string = self
            .time_dropdown
            .selected_item()
            .expect("dropdowns have been set up")
            .downcast_ref::<gtk::StringObject>()
            .expect("dropdown contains string items")
            .string();

        let duration = match time_string.as_str() {
            "15 seconds" => Duration::from_secs(15),
            "30 seconds" => Duration::from_secs(30),
            "1 minute" => Duration::from_secs(60),
            "5 minutes" => Duration::from_secs(5 * 60),
            "10 minutes" => Duration::from_secs(10 * 60),
            _ => panic!("invalid time selected in dropdown"),
        };

        self.duration.set(duration);
    }
}
