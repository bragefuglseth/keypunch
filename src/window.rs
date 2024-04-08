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

mod dropdowns;
mod session;

use crate::text_view::RcwTextView;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use std::time::{Duration, Instant};
use std::cell::Cell;

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
    #[template(resource = "/dev/bragefuglseth/Raceway/window.ui")]
    pub struct RcwWindow {
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub header_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub header_bar_ready: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub mode_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub time_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub header_bar_running: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub stop_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub running_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub text_view: TemplateChild<RcwTextView>,
        #[template_child]
        pub ready_message: TemplateChild<gtk::Revealer>,

        pub text_type: Cell<TextType>,
        pub duration: Cell<Duration>,
        pub start_time: Cell<Option<Instant>>,
        pub running: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RcwWindow {
        const NAME: &'static str = "RcwWindow";
        type Type = super::RcwWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RcwWindow {
        fn constructed(&self) {
            self.parent_constructed();

            self.setup_dropdowns();
            self.setup_text_view();
            self.setup_stop_button();

            self.ready(false);
        }
    }
    impl WidgetImpl for RcwWindow {}
    impl WindowImpl for RcwWindow {}
    impl ApplicationWindowImpl for RcwWindow {}
    impl AdwApplicationWindowImpl for RcwWindow {}
}

glib::wrapper! {
    pub struct RcwWindow(ObjectSubclass<imp::RcwWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,        @implements gio::ActionGroup, gio::ActionMap;
}

impl RcwWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
}
