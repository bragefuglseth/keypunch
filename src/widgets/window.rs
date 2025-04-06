/* window.rs
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

mod focus;
mod inhibit;
mod typing_test;

use crate::application::KpApplication;
use crate::config::APP_ID;
use crate::settings;
use crate::typing_test_utils::*;
use crate::widgets::{KpResultsView, KpTextView};
use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{gio, glib};
use std::cell::{Cell, OnceCell, RefCell};
use std::time::{Duration, Instant};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "src/widgets/window.blp")]
    pub struct KpWindow {
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub test_type_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub secondary_config_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub duration_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub custom_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub header_bar_start: TemplateChild<gtk::Stack>,
        #[template_child]
        pub statistics_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub stop_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub status_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub status_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub text_view: TemplateChild<KpTextView>,
        #[template_child]
        pub bottom_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub bottom_stack_empty: TemplateChild<gtk::Box>,
        #[template_child]
        pub just_start_typing: TemplateChild<gtk::Label>,
        #[template_child]
        pub focus_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub results_view: TemplateChild<KpResultsView>,
        #[template_child]
        pub results_continue_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub frustration_continue_button: TemplateChild<gtk::Button>,

        pub settings: OnceCell<gio::Settings>,

        pub current_test: Cell<Option<TypingTest>>,
        pub show_cursor: Cell<bool>,
        pub cursor_hidden_timestamp: Cell<u32>,
        pub last_unfocus_timestamp: Cell<Option<Instant>>,
        pub last_unfocus_event: RefCell<Option<glib::SourceId>>,

        pub inhibit_cookie: Cell<Option<u32>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpWindow {
        const NAME: &'static str = "KpWindow";
        type Type = super::KpWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action("win.about", None, move |window, _, _| {
                window.imp().show_about_dialog();
            });

            klass.install_action("win.text-language-dialog", None, move |window, _, _| {
                window.imp().show_text_language_dialog();
            });

            klass.install_action("win.cancel-test", None, move |window, _, _| {
                window.imp().cancel_test();
            });

            klass.install_action("win.statistics-dialog", None, move |window, _, _| {
                window.imp().show_statistics_dialog();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for KpWindow {
        fn constructed(&self) {
            self.parent_constructed();

            if APP_ID.ends_with(".Devel") {
                self.obj().add_css_class("devel");
            }

            // Workaround until dropdowns gain proper flat styling in libadwaita 2.0
            self.test_type_dropdown
                .first_child()
                .unwrap()
                .add_css_class("flat");
            self.duration_dropdown
                .first_child()
                .unwrap()
                .add_css_class("flat");
        }
    }
    impl WidgetImpl for KpWindow {}
    impl WindowImpl for KpWindow {
        fn close_request(&self) -> glib::Propagation {
            // Save settings
            self.save_window_size().unwrap();

            // Don't inhibit the default handler
            glib::Propagation::Proceed
        }
    }
    impl ApplicationWindowImpl for KpWindow {}
    impl AdwApplicationWindowImpl for KpWindow {}

    impl KpWindow {
        pub(super) fn load_window_size(&self) {
            let app = self.obj().kp_application();
            let settings = app.settings();

            let width = settings.int("window-width");
            let height = settings.int("window-height");
            let maximized = settings.boolean("window-maximized");

            let obj = self.obj();
            obj.set_default_size(width, height);

            if maximized {
                obj.maximize();
            }
        }

        pub(super) fn save_window_size(&self) -> Result<(), glib::BoolError> {
            let obj = self.obj();
            let width = obj.default_width();
            let height = obj.default_height();
            let maximized = obj.is_maximized();

            let app = self.obj().kp_application();
            let settings = app.settings();

            settings.set_int("window-width", width)?;
            settings.set_int("window-height", height)?;
            settings.set_boolean("window-maximized", maximized)?;

            Ok(())
        }

        fn show_about_dialog(&self) {
            if self.is_running() || self.obj().visible_dialog().is_some() {
                return;
            }

            let about = adw::AboutDialog::from_appdata(
                "/dev/bragefuglseth/Keypunch/dev.bragefuglseth.Keypunch.metainfo.xml",
                Some("5.0"),
            );

            about.set_developers(&["Brage Fuglseth https://bragefuglseth.dev"]);

            about.add_credit_section(
                Some(&gettext("Orthography by")),
                &[
                    "Angelo Verlain Shema https://www.vixalien.com/",
                    "Arnob Goswami",
                    "Daniel Uhrinyi",
                    "Fabio Lovato https://loviuz.it/",
                    "Gregor Niehl https://gregorni.gitlab.io/",
                    "Hadi Azarnasab https://hadi7546.ir/",
                    "Ibrahim Muhammad",
                    "Kim Jimin https://developomp.com",
                    "Shellheim",
                    "Tamazight teachers of Tizi Ouzou",
                    "Urtsi Santsi",
                    "Yevhen Popok",
                ],
            );

            // Translators: Replace "translator-credits" with your names, one name per line
            about.set_translator_credits(&gettext("translator-credits"));

            about.set_copyright("© 2024–2025 Brage Fuglseth");

            about.add_acknowledgement_section(
                Some(&gettext("Special thanks to")),
                &["Sophie Herold https://www.patreon.com/sophieh"],
            );

            about.add_other_app(
                "dev.bragefuglseth.Fretboard",
                // Translators: Metainfo for the app Fretboard. <https://github.com/bragefuglseth/fretboard>
                &gettext("Fretboard"),
                // Translators: Metainfo for the app Fretboard. <https://github.com/bragefuglseth/fretboard>
                &gettext("Look up guitar chords"),
            );

            about.connect_closed(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.focus_text_view();
                }
            ));

            about.present(Some(self.obj().upcast_ref::<gtk::Widget>()));
        }

        pub fn is_running(&self) -> bool {
            self.current_test.get().is_some()
        }
    }
}

glib::wrapper! {
    pub struct KpWindow(ObjectSubclass<imp::KpWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl KpWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        let obj: KpWindow = glib::Object::builder()
            .property("application", application)
            .build();

        let imp = obj.imp();
        imp.load_window_size();
        imp.setup_text_view();
        imp.setup_focus();
        imp.setup_ui_hiding();
        imp.show_cursor();
        imp.setup_test_config();
        imp.ready();

        obj
    }

    pub fn kp_application(&self) -> KpApplication {
        self.application()
            .unwrap()
            .downcast::<KpApplication>()
            .unwrap()
    }
}
