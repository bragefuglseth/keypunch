/* application.rs
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

use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::widgets::KpWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct KpApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for KpApplication {
        const NAME: &'static str = "KpApplication";
        type Type = super::KpApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for KpApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.set_resource_base_path(Some("/dev/bragefuglseth/Keypunch/"));

            obj.setup_gactions();

            obj.set_accels_for_action("win.text-language-dialog", &["<primary>comma"]);
            obj.set_accels_for_action("win.cancel-session", &["Escape"]);
            obj.set_accels_for_action("win.close", &["<primary>w"]);
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
        }
    }

    impl ApplicationImpl for KpApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let application = self.obj();
            // Get the current window or create one if necessary
            let window = application.active_window().unwrap_or_else(|| {
                let window = KpWindow::new(&*application);
                window.upcast()
            });

            // Ask the window manager/compositor to present the window
            window.present();
        }
    }

    impl GtkApplicationImpl for KpApplication {}
    impl AdwApplicationImpl for KpApplication {}
}

glib::wrapper! {
    pub struct KpApplication(ObjectSubclass<imp::KpApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl KpApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .build()
    }

    fn setup_gactions(&self) {
        let actions = [gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build()];

        self.add_action_entries(actions);
    }
}
