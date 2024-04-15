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
mod session_config;
mod settings;
mod ui_state;

use crate::results_view::KpResultsView;
use crate::text_view::KpTextView;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use std::cell::OnceCell;
use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Default)]
pub enum SessionType {
    #[default]
    Simple,
    Advanced,
    Custom,
}

#[derive(Clone, Copy, Default)]
pub enum SessionDuration {
    #[default]
    Sec15,
    Sec30,
    Min1,
    Min5,
    Min10,
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/bragefuglseth/Keypunch/ui/window.ui")]
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
        pub custom_text: RefCell<String>,
        pub duration: Cell<SessionDuration>,
        pub start_time: Cell<Option<Instant>>,
        pub finish_time: Cell<Option<Instant>>,
        pub running: Cell<bool>,
        pub show_cursor: Cell<bool>,
        pub cursor_hidden_timestamp: Cell<u32>,
        pub last_unfocus_timestamp: Cell<Option<Instant>>,
        pub last_unfocus_event: RefCell<Option<glib::SourceId>>,
        pub open_dialog: Cell<bool>,
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
            self.setup_settings();
            self.setup_session_config();

            self.setup_text_view();
            self.setup_focus();
            self.setup_stop_button();
            self.setup_continue_button();
            self.setup_ui_hiding();
            self.show_cursor();

            self.ready(false);
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
