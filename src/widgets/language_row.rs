use crate::enums::Language;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::{Cell, OnceCell};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct KpLanguageRow {
        pub check_button: OnceCell<gtk::CheckButton>,
        pub language: Cell<Language>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpLanguageRow {
        const NAME: &'static str = "KpLanguageRow";
        type Type = super::KpLanguageRow;
        type ParentType = adw::ActionRow;
    }

    impl ObjectImpl for KpLanguageRow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let check_button = self.check_button();
            obj.add_prefix(&check_button);
            obj.set_activatable_widget(Some(&check_button));
        }
    }
    impl WidgetImpl for KpLanguageRow {}
    impl ListBoxRowImpl for KpLanguageRow {}
    impl PreferencesRowImpl for KpLanguageRow {}
    impl ActionRowImpl for KpLanguageRow {}

    impl KpLanguageRow {
        pub(super) fn check_button(&self) -> gtk::CheckButton {
            self.check_button
                .get_or_init(|| gtk::CheckButton::new())
                .to_owned()
        }

        pub(super) fn set_language(&self, language: Language) {
            self.language.set(language);
            self.obj().set_title(language.pretty_name());
        }
    }
}

glib::wrapper! {
    pub struct KpLanguageRow(ObjectSubclass<imp::KpLanguageRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow;
}

impl KpLanguageRow {
    pub fn new(language: Language) -> Self {
        let obj = glib::Object::new::<Self>();
        obj.imp().set_language(language);
        obj
    }

    pub fn language(&self) -> Language {
        self.imp().language.get()
    }

    pub fn check_button(&self) -> gtk::CheckButton {
        self.imp().check_button()
    }
}
