use super::*;
use crate::text_generation;
use crate::widgets::{KpCustomTextDialog, KpTextLanguageDialog};
use glib::ControlFlow;
use text_generation::CHUNK_GRAPHEME_COUNT;
use unicode_segmentation::UnicodeSegmentation;

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

    pub(super) fn setup_text_view(&self) {
        let text_view = self.text_view.get();

        text_view.connect_running_notify(glib::clone!(@weak self as imp => move |tw| {
            if tw.running() {
                imp.start();
            }
        }));

        text_view.connect_typed_text_notify(glib::clone!(@weak self as imp => move |tw| {
            if !imp.running.get() { return; }

            let original_text = tw.original_text();
            let typed_text = tw.typed_text();

            let original_grapheme_count = original_text.graphemes(true).count();
            let typed_grapheme_count = typed_text.graphemes(true).count();

            if typed_grapheme_count >= original_grapheme_count {
                imp.finish();
            }

            if typed_grapheme_count > original_grapheme_count.checked_sub(CHUNK_GRAPHEME_COUNT / 2).unwrap_or(CHUNK_GRAPHEME_COUNT) {
                imp.extend_original_text();
            }

            match imp.session_type.get() {
                SessionType::Simple | SessionType::Advanced => (),
                SessionType::Custom => {
                    let current_word = original_text
                        .graphemes(true)
                        .take(typed_grapheme_count)
                        .collect::<String>()
                        .unicode_words()
                        .count();

                    let total_words = original_text.unicode_words().count();

                    imp.running_title.set_title(&format!("{current_word} ⁄ {total_words}"));
                }
            }
        }));
    }

    pub(super) fn update_original_text(&self) {
        let session_type = SessionType::from_i32(self.session_type_dropdown.selected() as i32)
            .expect("dropdown only contains valid `SessionType` values");

        let config_widget = match session_type {
            SessionType::Simple | SessionType::Advanced => {
                self.duration_dropdown.get().upcast::<gtk::Widget>()
            }
            SessionType::Custom => self.custom_button.get().upcast::<gtk::Widget>(),
        };

        self.session_type.set(session_type);

        let new_original = match session_type {
            SessionType::Simple => text_generation::simple(self.language.get()),
            SessionType::Advanced => text_generation::advanced(self.language.get()),
            SessionType::Custom => self.custom_text.borrow().to_string(),
        };
        self.text_view.set_original_text(&new_original);
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

    pub(super) fn show_text_language_dialog(&self) {
        let dialog =
            KpTextLanguageDialog::new(self.language.get(), &self.recent_languages.borrow());

        dialog.connect_closed(glib::clone!(@weak self as imp => move |_| {
            imp.open_dialog.set(false);
            imp.focus_text_view();
        }));

        self.open_dialog.set(true);
        dialog.present(self.obj().upcast_ref::<gtk::Widget>());
    }

    pub(super) fn extend_original_text(&self) {
        let language = self.language.get();
        let new_chunk = match self.session_type.get() {
            SessionType::Simple => text_generation::simple(language),
            SessionType::Advanced => text_generation::advanced(language),
            SessionType::Custom => {
                return;
            }
        };
        self.text_view.push_original_text(&new_chunk);
    }

    pub(super) fn start_timer(&self) {
        let duration = match self.duration.get() {
            SessionDuration::Sec15 => Duration::from_secs(15),
            SessionDuration::Sec30 => Duration::from_secs(30),
            SessionDuration::Min1 => Duration::from_secs(60),
            SessionDuration::Min5 => Duration::from_secs(5 * 60),
            SessionDuration::Min10 => Duration::from_secs(10 * 60),
        };

        self.update_timer(duration.as_secs() + 1);

        glib::timeout_add_local(
            Duration::from_millis(100),
            glib::clone!(@weak self as imp, @strong duration => @default-return ControlFlow::Break, move || {
                let start_time = imp.start_time.get().expect("start time is set when session is running");

                if !imp.running.get() { return ControlFlow::Break; };

                if let Some(diff) = duration.checked_sub(start_time.elapsed()) {
                    let seconds = diff.as_secs() + 1;

                    // add trailing zero for second values below 10
                    imp.update_timer(seconds);
                    ControlFlow::Continue
                } else {
                    imp.finish();
                    ControlFlow::Break
                }
            }),
        );
    }

    fn update_timer(&self, seconds: u64) {
        // add trailing zero for second values below 10
        let text = if seconds >= 60 && seconds % 60 < 10 {
            format!("{}∶0{}", seconds / 60, seconds % 60) // trailing zero
        } else if seconds >= 60 {
            format!("{}∶{}", seconds / 60, seconds % 60)
        } else {
            seconds.to_string()
        };

        self.running_title.set_title(&text);
    }
}
