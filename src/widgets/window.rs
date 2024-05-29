/* window.rs
 *
 * Copyright 2024 Brage Fuglseth
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
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

mod focus;
mod session;
mod settings;
mod ui_state;

use crate::config::APP_ID;
use crate::text_generation::Language;
use crate::widgets::{KpResultsView, KpTextView};
use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{gio, glib};
use std::cell::{Cell, OnceCell, RefCell};
use std::time::{Duration, Instant};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Clone, Copy, Default, PartialEq, EnumString, EnumDisplay, EnumIter)]
pub enum SessionType {
    #[default]
    Simple,
    Advanced,
    Custom,
}

impl SessionType {
    pub fn ui_string(&self) -> String {
        match self {
            SessionType::Simple => gettext("Simple"),
            SessionType::Advanced => gettext("Advanced"),
            SessionType::Custom => gettext("Custom"),
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, EnumString, EnumDisplay, EnumIter)]
pub enum SessionDuration {
    #[default]
    Sec15,
    Sec30,
    Min1,
    Min5,
    Min10,
}

impl SessionDuration {
    pub fn ui_string(&self) -> String {
        match self {
            SessionDuration::Sec15 => gettext("15 seconds"),
            SessionDuration::Sec30 => gettext("30 seconds"),
            SessionDuration::Min1 => gettext("1 minute"),
            SessionDuration::Min5 => gettext("5 minutes"),
            SessionDuration::Min10 => gettext("10 minutes"),
        }
    }
}

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
        pub header_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub header_bar_ready: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub session_type_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub secondary_config_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub duration_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub custom_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub header_bar_running: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub stop_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub running_title: TemplateChild<adw::WindowTitle>,
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
        pub continue_button: TemplateChild<gtk::Button>,

        pub settings: OnceCell<gio::Settings>,

        pub session_type: Cell<SessionType>,
        pub language: Cell<Language>,
        pub recent_languages: RefCell<Vec<Language>>,
        pub custom_text: RefCell<String>,
        pub duration: Cell<SessionDuration>,
        pub start_time: Cell<Option<Instant>>,
        pub finish_time: Cell<Option<Instant>>,
        pub running: Cell<bool>,
        pub show_cursor: Cell<bool>,
        pub cursor_hidden_timestamp: Cell<u32>,
        pub last_unfocus_timestamp: Cell<Option<Instant>>,
        pub last_unfocus_event: RefCell<Option<glib::SourceId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpWindow {
        const NAME: &'static str = "KpWindow";
        type Type = super::KpWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("win.about", None, move |window, _, _| {
                window.imp().show_about_dialog();
            });

            klass.install_action("win.text-language-dialog", None, move |window, _, _| {
                window.imp().show_text_language_dialog();
            });

            klass.install_action("win.cancel-session", None, move |window, _, _| {
                window.imp().ready();
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

            self.load_settings();
            self.setup_session_config();

            self.setup_text_view();
            self.setup_focus();
            self.setup_stop_button();
            self.setup_continue_button();
            self.setup_ui_hiding();
            self.show_cursor();

            self.ready();
        }
    }
    impl WidgetImpl for KpWindow {}
    impl WindowImpl for KpWindow {
        fn close_request(&self) -> glib::Propagation {
            // Save settings
            self.save_settings().expect("able to save settings");

            // Don't inhibit the default handler
            glib::Propagation::Proceed
        }
    }
    impl ApplicationWindowImpl for KpWindow {}
    impl AdwApplicationWindowImpl for KpWindow {}

    impl KpWindow {
        fn show_about_dialog(&self) {
            let about = adw::AboutDialog::from_appdata(
                "/dev/bragefuglseth/Keypunch/dev.bragefuglseth.Keypunch.metainfo.xml",
                Some("1.0"),
            );

            about.set_developers(&["Brage Fuglseth https://bragefuglseth.dev"]);

            about.add_credit_section(
                Some(&gettext("Ortography by")),
                &[
                    "Angelo Verlain Shema https://www.vixalien.com/",
                    "Gregor Niehl https://gregorni.gitlab.io/",
                    "Ibrahim Muhammad",
                ],
            );

            // Translators: Replace "translator-credits" with your names, one name per line
            about.set_translator_credits(&gettext("translator-credits"));

            about.set_copyright("© 2024 Brage Fuglseth");

            about.add_acknowledgement_section(
                Some(&gettext("Special thanks to")),
                &["Sophie Herold https://www.patreon.com/sophieh"],
            );

            about.connect_closed(glib::clone!(@weak self as imp => move |_| {
                imp.focus_text_view();
            }));

            about.present(self.obj().upcast_ref::<gtk::Widget>());
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
        glib::Object::builder()
            .property("application", application)
            .build()
    }
}
