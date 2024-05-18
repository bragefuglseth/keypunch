use crate::enums::Language;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::{Cell, OnceCell};
use strum::EnumMessage;

mod imp {
    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::KpLanguageRow)]
    pub struct KpLanguageRow {
        pub check_button: OnceCell<gtk::CheckButton>,
        pub language: Cell<Language>,

        #[property(get, set)]
        pub checked: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpLanguageRow {
        const NAME: &'static str = "KpLanguageRow";
        type Type = super::KpLanguageRow;
        type ParentType = adw::ActionRow;
    }

    impl ObjectImpl for KpLanguageRow {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            let check_button = self.check_button();
            obj.add_prefix(&check_button);
            obj.set_activatable_widget(Some(&check_button));

            check_button
                .bind_property("active", obj.upcast_ref::<glib::Object>(), "checked")
                .bidirectional()
                .sync_create()
                .build();
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
            self.obj().set_title(
                language
                    .get_message()
                    .expect("all languages have names set"),
            );
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
