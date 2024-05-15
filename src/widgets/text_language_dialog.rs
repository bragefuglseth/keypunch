use crate::enums::Language;
use crate::widgets::KpLanguageRow;
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::glib;
use std::sync::OnceLock;
use strum::IntoEnumIterator;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "src/widgets/text_language_dialog.blp")]
    pub struct KpTextLanguageDialog {
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub group_recent: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub group_other: TemplateChild<adw::PreferencesGroup>,
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
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![Signal::builder("change-language")
                    .param_types([str::static_type()])
                    .build()]
            })
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
            current_language_row.check_button().set_active(true);

            self.group_recent.add(&current_language_row);

            let recent_without_current = recent
                .iter()
                .filter(|&&recent_language| recent_language != current);
            for language in recent_without_current {
                let row = KpLanguageRow::new(*language);
                row.check_button().set_group(Some(&check_button_group));
                self.group_recent.add(&row);
            }

            let languages_without_recent_or_current = Language::iter().filter(|language| {
                !recent
                    .iter()
                    .chain([&current])
                    .any(|recent_language| recent_language == language)
            });
            for language in languages_without_recent_or_current {
                let row = KpLanguageRow::new(language);
                row.check_button().set_group(Some(&check_button_group));
                self.group_other.add(&row);
            }
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
