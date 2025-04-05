/* text_language_dialog.rs
 *
 * SPDX-FileCopyrightText: © 2024 Brage Fuglseth <bragefuglseth@gnome.org>
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::text_generation::Language;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use std::cell::RefCell;
use std::iter::once;
use std::str::FromStr;
use strum::{EnumMessage, IntoEnumIterator};
use unidecode::unidecode;

const LANGUAGE_REQUEST_URL: &'static str = "https://github.com/bragefuglseth/keypunch/issues/new?assignees=&labels=new+language&projects=&template=language_request.yaml&title=%5BLanguage+Request%5D%3A+";

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

        #[property(get, construct_only, nullable)]
        pub settings: RefCell<Option<gio::Settings>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpTextLanguageDialog {
        const NAME: &'static str = "KpTextLanguageDialog";
        type Type = super::KpTextLanguageDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for KpTextLanguageDialog {
        fn constructed(&self) {
            self.parent_constructed();

            let header_bar = self.header_bar.get();
            self.scrolled_window
                .vadjustment()
                .bind_property("value", &header_bar, "show-title")
                .transform_to(|_, scroll_position: f64| Some(scroll_position > 0.))
                .sync_create()
                .build();

            self.search_entry.connect_search_changed(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.update_search_state();
                }
            ));

            self.search_entry.connect_stop_search(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.obj().close();
                }
            ));
        }
    }
    impl WidgetImpl for KpTextLanguageDialog {}
    impl AdwDialogImpl for KpTextLanguageDialog {
        fn closed(&self) {
            let settings = self.obj().settings().unwrap();

            let current_language = settings.string("text-language").to_string();
            let stored_recent_languages = settings
                .value("recent-languages")
                .get::<Vec<String>>()
                .unwrap();

            let recent_languages: Vec<String> = once(current_language.clone())
                .chain(
                    stored_recent_languages
                        .into_iter()
                        .filter(|recent_language| &*recent_language != &current_language),
                )
                .take(3)
                .collect();

            settings
                .set_value("recent-languages", &recent_languages.to_variant())
                .unwrap();
        }
    }

    #[gtk::template_callbacks]
    impl KpTextLanguageDialog {
        pub(super) fn populate_list(&self, current: Language, recent: &[Language]) {
            let current_language_row = language_row(current);

            self.group_recent.add(&current_language_row);

            let recent_without_current = recent
                .iter()
                .filter(|&&recent_language| recent_language != current);
            for language in recent_without_current {
                let row = language_row(*language);
                self.group_recent.add(&row);
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
            languages_without_recent_or_current.sort_by_key(|language| {
                language
                    .get_message()
                    .expect("all languages have names set")
            });

            for language in languages_without_recent_or_current {
                let row = language_row(language);

                self.group_other.add(&row);
            }
        }

        pub(super) fn update_search_state(&self) {
            let query = self.search_entry.text();

            if query.is_empty() {
                self.no_results_lock_height(true);
                self.stack.set_visible_child_name("list");
            } else {
                let normalized_query = unidecode(&query.to_lowercase());
                let mut results: Vec<Language> = Language::iter()
                    .filter(|language| {
                        unidecode(
                            &language
                                .get_message()
                                .expect("all languages have names set")
                                .to_lowercase(),
                        )
                        .contains(&normalized_query)
                    })
                    .collect();

                results.sort_by_key(|language| {
                    language
                        .get_message()
                        .expect("all languages have names set")
                        .to_lowercase()
                });

                if results.is_empty() {
                    self.no_results_lock_height(false);
                    self.stack.set_visible_child_name("no-results");
                } else {
                    let search_list = self.search_list.get();

                    search_list.remove_all();

                    for result in results {
                        let row = language_row(result);
                        search_list.append(&row);
                    }

                    self.no_results_lock_height(true);
                    self.stack.set_visible_child_name("search-results");
                }
            }
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

        #[template_callback]
        pub(super) fn language_request_button_clicked(button: &gtk::Button) {
            let root = button
                .root()
                .map(|root| root.downcast::<gtk::Window>().unwrap());
            let launcher = gtk::UriLauncher::new(LANGUAGE_REQUEST_URL);

            launcher.launch(root.as_ref(), None::<gio::Cancellable>.as_ref(), |_| ());
        }

        #[template_callback]
        pub(super) fn load_language_illustration(picture: &gtk::Picture) {
            picture.set_resource(Some("/dev/bragefuglseth/Keypunch/assets/multilingual.svg"));
        }
    }
}

glib::wrapper! {
    pub struct KpTextLanguageDialog(ObjectSubclass<imp::KpTextLanguageDialog>)
        @extends gtk::Widget, adw::Dialog;
}

impl KpTextLanguageDialog {
    pub fn new(settings: &gio::Settings) -> Self {
        let obj: Self = glib::Object::builder()
            .property("settings", settings.clone())
            .build();

        let current =
            Language::from_str(&settings.string("text-language")).unwrap_or(Language::English);

        let recent: Vec<Language> = settings
            .value("recent-languages")
            .get::<Vec<String>>()
            .unwrap()
            .iter()
            .map(|s| Language::from_str(&s).unwrap())
            .collect();

        let imp = obj.imp();

        imp.populate_list(current, &recent);

        obj
    }
}

fn language_row(language: Language) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    row.set_title(&language.get_message().unwrap());

    let check_button = gtk::CheckButton::builder()
        .action_name("app.text-language")
        .action_target(&language.to_string().to_variant())
        .build();

    row.add_prefix(&check_button);
    row.set_activatable_widget(Some(&check_button));

    row
}
