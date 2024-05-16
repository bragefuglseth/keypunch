use crate::enums::Language;
use crate::widgets::KpLanguageRow;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::RefCell;
use strum::IntoEnumIterator;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/widgets/text_language_dialog.blp")]
    #[properties(wrapper_type = super::KpTextLanguageDialog)]
    pub struct KpTextLanguageDialog {
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub group_recent: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub group_other: TemplateChild<adw::PreferencesGroup>,

        #[property(get, set)]
        pub selected_language: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpTextLanguageDialog {
        const NAME: &'static str = "KpTextLanguageDialog";
        type Type = super::KpTextLanguageDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for KpTextLanguageDialog {
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

            let header_bar = self.header_bar.get();
            self.scrolled_window
                .vadjustment()
                .bind_property("value", &header_bar, "show-title")
                .transform_to(|_, scroll_position: f64| Some(scroll_position > 0.))
                .sync_create()
                .build();
        }
    }
    impl WidgetImpl for KpTextLanguageDialog {}
    impl AdwDialogImpl for KpTextLanguageDialog {}

    impl KpTextLanguageDialog {
        pub(super) fn populate_list(&self, current: Language, recent: &Vec<Language>) {
            let current_language_row = KpLanguageRow::new(current);
            let check_button_group = current_language_row.check_button();
            current_language_row.set_checked(true);
            self.connect_row_clicked(&current_language_row);

            self.group_recent.add(&current_language_row);

            let recent_without_current = recent
                .iter()
                .filter(|&&recent_language| recent_language != current);
            for language in recent_without_current {
                let row = KpLanguageRow::new(*language);
                row.check_button().set_group(Some(&check_button_group));
                self.group_recent.add(&row);
                self.connect_row_clicked(&row);
            }

            let mut languages_without_recent_or_current: Vec<Language> = Language::iter()
                .filter(|language| {
                    !recent
                        .iter()
                        .chain([&current])
                        .any(|recent_language| recent_language == language)
                })
                .collect();

            // Sort alphabetically
            languages_without_recent_or_current.sort_by_key(|language| language.pretty_name());

            for language in languages_without_recent_or_current {
                let row = KpLanguageRow::new(language);
                row.check_button().set_group(Some(&check_button_group));
                self.group_other.add(&row);
                self.connect_row_clicked(&row);
            }
        }

        pub(super) fn connect_row_clicked(&self, row: &KpLanguageRow) {
            let obj = self.obj();
            row.connect_checked_notify(glib::clone!(@weak obj => move |row| {
                if row.checked() {
                    obj.set_selected_language(row.language().to_string());
                }
            }));
        }
    }
}

glib::wrapper! {
    pub struct KpTextLanguageDialog(ObjectSubclass<imp::KpTextLanguageDialog>)
        @extends gtk::Widget, adw::Dialog;
}

impl KpTextLanguageDialog {
    pub fn new(current: Language, recent: &Vec<Language>) -> Self {
        let obj: Self = glib::Object::new();

        obj.imp().populate_list(current, recent);

        obj
    }
}
