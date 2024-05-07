use super::*;
use crate::text_generation;
use crate::widgets::KpCustomTextDialog;

impl imp::KpWindow {
    pub(super) fn setup_session_config(&self) {
        let session_type_dropdown = self.session_type_dropdown.get();
        session_type_dropdown.set_model(Some(&SessionType::string_list()));
        session_type_dropdown.set_selected(self.session_type.get() as u32);
        session_type_dropdown.connect_selected_item_notify(
            glib::clone!(@weak self as imp => move |_| {
                imp.update_original_text();
                imp.focus_text_view();
            }),
        );

        let duration_dropdown = self.duration_dropdown.get();
        duration_dropdown.set_model(Some(&SessionDuration::string_list()));
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
        let session_type = SessionType::from_i32(self.session_type_dropdown.selected() as i32)
            .expect("dropdown only contains valid `SessionType` values");

        let text = match session_type {
            SessionType::Simple => text_generation::simple("en_US").unwrap(),
            SessionType::Advanced => text_generation::advanced("en_US").unwrap(),
            SessionType::Custom => self.custom_text.borrow().to_string(),
        };

        let config_widget = match session_type {
            SessionType::Simple | SessionType::Advanced => {
                self.duration_dropdown.get().upcast::<gtk::Widget>()
            }
            SessionType::Custom => self.custom_button.get().upcast::<gtk::Widget>(),
        };

        self.session_type.set(session_type);
        self.text_view.set_original_text(&text);
        self.secondary_config_stack
            .set_visible_child(&config_widget);
    }

    pub(super) fn update_time(&self) {
        let selected = SessionDuration::from_i32(self.duration_dropdown.selected() as i32)
            .expect("dropdown only contains valid `SessionDuration` values");

        self.duration.set(selected);
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
