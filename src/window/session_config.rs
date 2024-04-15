use super::*;
use crate::custom_text_dialog::KpCustomTextDialog;
use glib::GString;

impl imp::KpWindow {
    pub(super) fn setup_session_config(&self) {
        let session_type_model = gtk::StringList::new(&["Simple", "Advanced", "Custom"]);
        let session_type_dropdown = self.session_type_dropdown.get();
        session_type_dropdown.set_model(Some(&session_type_model));
        session_type_dropdown.set_selected(self.session_type.get() as u32);
        session_type_dropdown.connect_selected_item_notify(
            glib::clone!(@weak self as imp => move |_| {
                imp.update_original_text();
                imp.focus_text_view();
            }),
        );

        let duration_model = gtk::StringList::new(&[
            "15 seconds",
            "30 seconds",
            "1 minute",
            "5 minutes",
            "10 minutes",
        ]);
        let duration_dropdown = self.duration_dropdown.get();

        duration_dropdown.set_model(Some(&duration_model));

        duration_dropdown.set_selected(self.duration.get() as u32);

        duration_dropdown.connect_selected_item_notify(
            glib::clone!(@weak self as imp => move |_| {
                imp.update_time();
                imp.focus_text_view();
            }),
        );

        self.custom_button
            .connect_clicked(glib::clone!(@weak self as imp => move |_| {
                let current_text = imp.custom_text.borrow();
                imp.show_custom_text_dialog(&current_text);
            }));
    }

    pub(super) fn update_original_text(&self) {
        let session_type = match self.session_type().as_str() {
            "Simple" => SessionType::Simple,
            "Advanced" => SessionType::Advanced,
            "Custom" => SessionType::Custom,
            _ => panic!("invalid mode selected in dropdown"),
        };

        let custom = self.custom_text.borrow();

        let text = match session_type {
            SessionType::Simple => "lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore et dolore magnam aliquam quaerat voluptatem ut enim aeque doleamus animo cum corpore dolemus fieri tamen permagna accessio potest si aliquod aeternum et infinitum impendere malum nobis opinemur quod idem licet transferre in voluptatem ut",
            SessionType::Advanced => "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magnam aliquam quaerat voluptatem. Ut enim aeque doleamus animo, cum corpore dolemus, fieri tamen permagna accessio potest, si aliquod aeternum et infinitum impendere malum nobis opinemur. Quod idem licet transferre in voluptatem, ut.",
            SessionType::Custom => custom.as_str(),
        };

        let config_widget = match session_type {
            SessionType::Simple | SessionType::Advanced => {
                self.duration_dropdown.get().upcast::<gtk::Widget>()
            }
            SessionType::Custom => self.custom_button.get().upcast::<gtk::Widget>(),
        };

        self.session_type.set(session_type);
        self.text_view.set_original_text(text);
        self.secondary_config_stack
            .set_visible_child(&config_widget);
    }

    pub(super) fn session_type(&self) -> GString {
        self.session_type_dropdown
            .selected_item()
            .expect("dropdowns have been set up")
            .downcast_ref::<gtk::StringObject>()
            .expect("dropdown contains string items")
            .string()
    }

    pub(super) fn update_time(&self) {
        let selected_idx = self.duration_dropdown.selected();

        let duration = match selected_idx {
            0 => SessionDuration::Sec15,
            1 => SessionDuration::Sec30,
            2 => SessionDuration::Min1,
            3 => SessionDuration::Min5,
            4 => SessionDuration::Min10,
            _ => panic!("not all dropdown options covered"),
        };

        self.duration.set(duration);
    }

    pub fn show_custom_text_dialog(&self, initial_text: &str) {
        let current_text = self.custom_text.borrow();

        let dialog = KpCustomTextDialog::new(&current_text, &initial_text);

        dialog.connect_local("save", true, glib::clone!(@weak self as imp => @default-return None, move |values| {
            let text: &str = values.get(1).expect("save signal contains text to be saved").get().expect("value from save signal is string");
            *imp.custom_text.borrow_mut() = text.to_string();
            imp.update_original_text();

            None
        }));

        dialog.connect_local("discard", true, glib::clone!(@weak self as imp => @default-return None, move |values| {
            let discarded_text: String = values.get(1).expect("save signal contains text to be saved").get::<&str>().expect("value from save signal is string").into();

            let toast = adw::Toast::builder()
                .title("Changes discarded")
                .button_label("Restore")
                .build();
            toast.connect_button_clicked(glib::clone!(@weak imp => move |_| {
                imp.show_custom_text_dialog(&discarded_text);
            }));

            imp.toast_overlay.add_toast(toast);

            None
        }));

        dialog.connect_closed(glib::clone!(@weak self as imp => move |_| {
            imp.open_dialog.set(false);
            imp.focus_text_view();
        }));

        self.open_dialog.set(true);
        dialog.present(self.obj().upcast_ref::<gtk::Widget>());
    }
}
