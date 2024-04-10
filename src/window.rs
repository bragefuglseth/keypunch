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

mod session_config;
mod session;
mod ui_state;
mod settings;

use crate::text_view::KpTextView;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};
use std::cell::OnceCell;

#[derive(Clone, Copy, Default)]
pub enum TextType {
    #[default]
    Simple,
    Advanced,
    Custom,
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/bragefuglseth/Keypunch/window.ui")]
    pub struct KpWindow {
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub header_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub header_bar_ready: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub mode_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub secondary_config_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub time_dropdown: TemplateChild<gtk::DropDown>,
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
        pub ready_message: TemplateChild<gtk::Revealer>,

        pub settings: OnceCell<gio::Settings>,

        pub text_type: Cell<TextType>,
        pub duration: Cell<Duration>,
        pub start_time: Cell<Option<Instant>>,
        pub running: Cell<bool>,
        pub show_cursor: Cell<bool>,
        pub cursor_hidden_timestamp: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpWindow {
        const NAME: &'static str = "KpWindow";
        type Type = super::KpWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for KpWindow {
        fn constructed(&self) {
            self.parent_constructed();

            self.setup_session_config();
            self.setup_text_view();
            self.setup_stop_button();
            self.setup_ui_hiding();
            self.setup_settings();

            self.ready(false);
        }
    }
    impl WidgetImpl for KpWindow {}
    impl WindowImpl for KpWindow {
        fn close_request(&self) -> glib::Propagation {
            // Save settings
            self.save_settings()
                .expect("able to save settings");

            // Don't inhibit the default handler
            glib::Propagation::Proceed
        }
    }
    impl ApplicationWindowImpl for KpWindow {}
    impl AdwApplicationWindowImpl for KpWindow {}
}

glib::wrapper! {
    pub struct KpWindow(ObjectSubclass<imp::KpWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,        @implements gio::ActionGroup, gio::ActionMap;
}

impl KpWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
}
