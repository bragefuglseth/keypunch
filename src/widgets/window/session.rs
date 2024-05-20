use super::*;
use crate::text_generation;
use crate::widgets::{KpCustomTextDialog, KpTextLanguageDialog};
use gettextrs::gettext;
use glib::ControlFlow;
use gtk::pango;
use i18n_format::i18n_fmt;
use std::iter::once;
use strum::IntoEnumIterator;
use text_generation::CHUNK_GRAPHEME_COUNT;
use unicode_segmentation::UnicodeSegmentation;

impl imp::KpWindow {
    pub(super) fn setup_session_config(&self) {
        let session_type_model: gtk::StringList = SessionType::iter()
            .map(|session_type| session_type.ui_string())
            .collect();

        let session_type_dropdown = self.session_type_dropdown.get();
        session_type_dropdown.set_model(Some(&session_type_model));
        let selected_type_index = SessionType::iter()
            .position(|session_type| session_type == self.session_type.get())
            .unwrap();
        session_type_dropdown.set_selected(selected_type_index as u32);
        session_type_dropdown.connect_selected_item_notify(
            glib::clone!(@weak self as imp => move |_| {
                imp.update_original_text();
                imp.focus_text_view();
            }),
        );

        setup_ellipsizing_dropdown_factory(&session_type_dropdown);

        let duration_model: gtk::StringList = SessionDuration::iter()
            .map(|session_type| session_type.ui_string())
            .collect();

        let duration_dropdown = self.duration_dropdown.get();
        duration_dropdown.set_model(Some(&duration_model));
        let selected_duration_index = SessionDuration::iter()
            .position(|duration| duration == self.duration.get())
            .unwrap();
        duration_dropdown.set_selected(selected_duration_index as u32);
        duration_dropdown.connect_selected_item_notify(
            glib::clone!(@weak self as imp => move |_| {
                imp.update_time();
                imp.focus_text_view();
            }),
        );

        setup_ellipsizing_dropdown_factory(&duration_dropdown);

        self.custom_button
            .connect_clicked(glib::clone!(@weak self as imp => move |_| {
                let current_text = imp.custom_text.borrow();
                imp.show_custom_text_dialog(&current_text);
            }));

        self.add_recent_language(self.language.get());
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

                    // Translators: The `{}` blocks will be replaced with the current word count and the total word count.
                    // Do not translate them! The slash sign is a special unicode character, if your language doesn't
                    // use a completely different sign, you should probably copy and paste it from the original string.
                    imp.running_title.set_title(&i18n_fmt! { i18n_fmt("{} ⁄ {}", current_word, total_words) });
                }
            }
        }));
    }

    pub(super) fn update_original_text(&self) {
        let session_type = SessionType::iter()
            .nth(self.session_type_dropdown.selected() as usize)
            .expect("dropdown contains valid `SessionType` values");

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
        let selected = SessionDuration::iter()
            .nth(self.duration_dropdown.selected() as usize)
            .expect("dropdown only contains valid `SessionDuration` values");

        self.duration.set(selected);
    }

    pub(super) fn show_text_language_dialog(&self) {
        let dialog =
            KpTextLanguageDialog::new(self.language.get(), &self.recent_languages.borrow_mut());

        dialog.connect_local(
            "language-changed",
            true,
            glib::clone!(@weak self as imp => @default-return None, move |values| {
                let dialog: KpTextLanguageDialog = values
                    .get(0)
                    .expect("signal contains value at index 0")
                    .get()
                    .expect("value sent with signal is dialog");

                imp.language.set(dialog.selected_language());
                imp.update_original_text();

                None
            }),
        );

        self.block_text_view_unfocus.set(true);

        dialog.connect_closed(glib::clone!(@weak self as imp => move |dialog| {
            imp.add_recent_language(dialog.selected_language());
            imp.block_text_view_unfocus.set(false);
            imp.focus_text_view();
        }));

        dialog.present(self.obj().upcast_ref::<gtk::Widget>());
    }

    fn add_recent_language(&self, language: Language) {
        let mut recent_languages = self.recent_languages.borrow_mut();

        *recent_languages = once(language)
            .chain(
                recent_languages
                    .iter()
                    .filter(|&recent_language| *recent_language != language)
                    .map(|p| *p),
            )
            .take(3)
            .collect();
    }

    pub fn show_custom_text_dialog(&self, initial_text: &str) {
        let current_text = self.custom_text.borrow();

        let dialog = KpCustomTextDialog::new(&current_text, &initial_text);

        dialog.connect_local(
            "save",
            true,
            glib::clone!(@weak self as imp => @default-return None, move |values| {
                let text: &str = values
                    .get(1)
                    .expect("save signal contains text to be saved")
                    .get().expect("value from save signal is string");

                *imp.custom_text.borrow_mut() = text.to_string();
                imp.update_original_text();

                None
            }),
        );

        dialog.connect_local(
            "discard",
            true,
            glib::clone!(@weak self as imp => @default-return None, move |values| {
                let discarded_text: String = values
                    .get(1)
                    .expect("save signal contains text to be saved")
                    .get::<&str>().expect("value from save signal is string")
                    .into();

                let toast = adw::Toast::builder()
                    .title(&gettext("Changes discarded"))
                    .button_label(&gettext("Restore"))
                    .build();

                toast.connect_button_clicked(glib::clone!(@weak imp => move |_| {
                    imp.show_custom_text_dialog(&discarded_text);
                }));

                imp.toast_overlay.add_toast(toast);

                None
            }),
        );

        self.block_text_view_unfocus.set(true);

        dialog.connect_closed(glib::clone!(@weak self as imp => move |_| {
            imp.block_text_view_unfocus.set(false);
            imp.focus_text_view();
        }));

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
            // Translators: The format of the timer. The first `{}` block will be replaced
            // with the minutes passed, and the second one will be replaced with the seconds
            // passed. Do not translate the `{}` blocks. Note that the `∶` sign is a special
            // Unicode character; if your language doesn't use something completely different,
            // you should probably copy and paste it from the original string.
            i18n_fmt! { i18n_fmt("{}∶{}", seconds / 60, format!("0{}", seconds % 60)) }
        } else if seconds >= 60 {
            i18n_fmt! { i18n_fmt("{}∶{}", seconds / 60, seconds % 60) }
        } else {
            seconds.to_string()
        };

        self.running_title.set_title(&text);
    }
}

// Creates a custom factory for a dropdown that ellipsizes the label of the top button.
// The factory also applies a checkmark to the selected item if it's in the popover.
// This is essentially a clone of the default factory, but with ellipsizing.
// Ideally we'd do this by setting the factory of just the button part of `GtkDropDown`, but
// this isn't currently possible, and there are no plans to make it so upstream.
// See <https://gitlab.gnome.org/GNOME/gtk/-/issues/6720>
fn setup_ellipsizing_dropdown_factory(dropdown: &gtk::DropDown) {
    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(|_, obj| {
        let label = gtk::Label::builder().xalign(0.).build();
        let checkmark = gtk::Image::from_icon_name("check-plain-symbolic");

        let box_ = gtk::Box::builder().build();
        box_.append(&label);
        box_.append(&checkmark);
        obj.downcast_ref::<gtk::ListItem>()
            .unwrap()
            .set_child(Some(&box_));
    });

    factory.connect_bind(glib::clone!(@weak dropdown => move |_, obj| {
        let list_item = obj.downcast_ref::<gtk::ListItem>().unwrap();
        let child = list_item.child().unwrap();
        let box_ = child.downcast_ref::<gtk::Box>().unwrap();
        let first_child = box_.first_child().unwrap();
        let last_child = box_.last_child().unwrap();
        let string_object = list_item.item().unwrap().downcast::<gtk::StringObject>().unwrap();

        let label = first_child.downcast_ref::<gtk::Label>().unwrap();
        label.set_label(string_object.string().as_str());

        let is_in_popover = child.parent().unwrap().parent().unwrap().type_() == gtk::ListView::static_type();

        if is_in_popover {
            label.set_ellipsize(pango::EllipsizeMode::None);
            dropdown.connect_selected_item_notify(glib::clone!(@weak last_child, @weak string_object => move |dropdown| {
                last_child.set_visible(dropdown.selected_item().unwrap() == string_object);
            }));
        } else {
            label.set_ellipsize(pango::EllipsizeMode::End);
            last_child.set_visible(false);
        }
    }));

    dropdown.set_factory(Some(&factory));
    dropdown.notify("selected-item");
}
