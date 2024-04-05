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

use crate::text_view::RcwTextView;
use crate::typing_session::TypingSession;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/bragefuglseth/Raceway/window.ui")]
    pub struct RcwWindow {
        // Template widgets
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub text_view: TemplateChild<RcwTextView>,
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

            let text_view = self.text_view.get();

            text_view.set_typing_session(TypingSession::new("Lorem ipsum dolor sit amet.".to_string()));
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
