use crate::enums::Language;
use crate::widgets::KpLanguageRow;
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::glib;
use std::cell::{Cell, OnceCell};
use std::sync::OnceLock;
use strum::{EnumMessage, IntoEnumIterator};
use unidecode::unidecode;

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
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub group_recent: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub group_other: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub search_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub no_results_box: TemplateChild<gtk::Box>,

        pub check_button_group: OnceCell<gtk::CheckButton>,

        pub selected_language: Cell<Language>,
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
            SIGNALS.get_or_init(|| vec![Signal::builder("language-changed").build()])
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

            self.search_entry
                .connect_search_changed(glib::clone!(@weak self as imp => move |_| {
                    imp.update_search_state();
                }));
        }
    }
    impl WidgetImpl for KpTextLanguageDialog {}
    impl AdwDialogImpl for KpTextLanguageDialog {}

    impl KpTextLanguageDialog {
        pub(super) fn populate_list(&self, current: Language, recent: &[Language]) {
            let current_language_row = KpLanguageRow::new(current);
            self.check_button_group
                .set(current_language_row.check_button())
                .expect(
                    "check button group field hasn't been written to yet when list is populated",
                );

            current_language_row.set_checked(true);
            self.connect_row_checked(&current_language_row);

            self.group_recent.add(&current_language_row);

            let recent_without_current = recent
                .iter()
                .filter(|&&recent_language| recent_language != current);
            for language in recent_without_current {
                let row = KpLanguageRow::new(*language);
                row.check_button()
                    .set_group(Some(self.check_button_group.get().unwrap()));
                self.group_recent.add(&row);
                self.connect_row_checked(&row);
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
            languages_without_recent_or_current.sort_by_key(|language| language.get_message().expect("all languages have names set"));

            for language in languages_without_recent_or_current {
                let row = KpLanguageRow::new(language);
                row.check_button()
                    .set_group(Some(self.check_button_group.get().unwrap()));
                self.group_other.add(&row);
                self.connect_row_checked(&row);
            }
        }

        pub(super) fn update_search_state(&self) {
            let query = self.search_entry.text();

            if query.is_empty() {
                self.no_results_lock_height(true);
                self.stack.set_visible_child_name("list");
            } else {
                let normalized_query = unidecode(&query.to_lowercase());
                let results: Vec<Language> = Language::iter()
                    .filter(|language| {
                        unidecode(&language.get_message().expect("all languages have names set").to_lowercase())
                            .contains(&normalized_query)
                    })
                    .collect();

                if results.is_empty() {
                    self.no_results_lock_height(false);
                    self.stack.set_visible_child_name("no-results");
                } else {
                    let search_list = self.search_list.get();

                    search_list.remove_all();

                    let check_button_group = gtk::CheckButton::new();

                    for result in results {
                        let row = KpLanguageRow::new(result);
                        if self.selected_language.get() == result {
                            row.set_checked(true);
                        }

                        self.connect_row_checked(&row);
                        row.check_button().set_group(Some(&check_button_group));
                        search_list.append(&row);
                    }

                    self.no_results_lock_height(true);
                    self.stack.set_visible_child_name("search-results");
                }
            }
        }

        pub(super) fn connect_row_checked(&self, row: &KpLanguageRow) {
            row.connect_checked_notify(glib::clone!(@weak self as imp => move |row| {
                if row.checked() {
                    imp.selected_language.set(row.language());
                    imp.obj().emit_by_name::<()>("language-changed", &[]);
                }
            }));

            self.obj().connect_local(
                "language-changed",
                false,
                glib::clone!(@weak row, @weak self as imp => @default-return None, move |_| {
                    if row.language() == imp.selected_language.get() && !row.checked() {
                        row.set_checked(true);
                    }

                    None
                }),
            );
        }

        pub(super) fn no_results_lock_height(&self, lock: bool) {
            let no_results = self.no_results_box.get();

            no_results.set_height_request(-1);
            no_results.set_valign(gtk::Align::Fill);

            if lock {
                let height = if let Some(rect) = no_results.compute_bounds(&no_results) {
                    rect.height().trunc() as i32
                } else {
                    0
                };

                no_results.set_height_request(height);
                no_results.set_valign(gtk::Align::Start);
            }
        }
    }
}

glib::wrapper! {
    pub struct KpTextLanguageDialog(ObjectSubclass<imp::KpTextLanguageDialog>)
        @extends gtk::Widget, adw::Dialog;
}

impl KpTextLanguageDialog {
    pub fn new(current: Language, recent: &[Language]) -> Self {
        let obj: Self = glib::Object::new();

        let imp = obj.imp();

        imp.populate_list(current, recent);
        imp.selected_language.set(current);

        obj
    }

    pub fn selected_language(&self) -> Language {
        self.imp().selected_language.get()
    }
}
