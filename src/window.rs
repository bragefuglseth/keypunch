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
use crate::typing_session::SessionType;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use std::time::Duration;

enum SessionMode {
    Simple,
    Advanced,
    Custom,
}

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
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

            let text_view = self.text_view.get();

            text_view.typing_session().bind_property("progress-text", &self.running_title.get(), "title").sync_create().build();

            text_view.typing_session().connect_local("started", true, glib::clone!(@weak self as imp => @default-return None, move |_| {
                imp.header_stack.get().set_visible_child_name("running");
                imp.ready_message.set_reveal_child(false);
                None
            }));

            self.stop_button.connect_clicked(glib::clone!(@weak self as imp => move |_| {
                imp.header_stack.get().set_visible_child_name("ready");
                imp.ready_message.set_reveal_child(true);
                imp.text_view.get().typing_session().stop();
                imp.update_session_config();
                imp.focus_text_view();
            }));

            text_view.typing_session().connect_local("finished", true, glib::clone!(@weak self as imp => @default-return None, move |_| {
                imp.main_stack.set_visible_child_name("results");
                None
            }));

            self.update_session_config();
        }
    }
    impl WidgetImpl for RcwWindow {}
    impl WindowImpl for RcwWindow {}
    impl ApplicationWindowImpl for RcwWindow {}
    impl AdwApplicationWindowImpl for RcwWindow {}

    impl RcwWindow {
        fn setup_dropdowns(&self) {
            let mode_dropdown = self.mode_dropdown.get();
            let mode_model = gtk::StringList::new(&["Simple", "Advanced", "Custom"]);
            mode_dropdown.set_model(Some(&mode_model));
            mode_dropdown.connect_selected_item_notify(glib::clone!(@weak self as imp => move |_| {
                imp.update_session_config();
                imp.focus_text_view();
            }));

            let time_dropdown = self.time_dropdown.get();
            let time_model = gtk::StringList::new(&["15 seconds", "30 seconds", "1 minute", "5 minutes", "10 minutes"]);
            time_dropdown.set_model(Some(&time_model));
            time_dropdown.connect_selected_item_notify(glib::clone!(@weak self as imp => move |_| {
                imp.update_session_config();
                imp.focus_text_view();
            }));
        }

        fn update_session_config(&self) {
            let session = self.text_view.get().typing_session();

            let mode_string = self.mode_dropdown.selected_item().unwrap().downcast_ref::<gtk::StringObject>().unwrap().string();

            let mode = match mode_string.as_str() {
                "Simple" => SessionMode::Simple,
                "Advanced" => SessionMode::Advanced,
                "Custom" => SessionMode::Custom,
                _ => panic!("non-existent session mode set"),
            };

            let text = match mode {
                SessionMode::Simple => "lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore et dolore magnam aliquam quaerat voluptatem Ut enim aeque doleamus animo cum corpore dolemus fieri tamen permagna accessio potest si aliquod aeternum et infinitum impendere malum nobis opinemur Quod idem licet transferre in voluptatem ut",
                SessionMode::Advanced => "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magnam aliquam quaerat voluptatem. Ut enim aeque doleamus animo, cum corpore dolemus, fieri tamen permagna accessio potest, si aliquod aeternum et infinitum impendere malum nobis opinemur. Quod idem licet transferre in voluptatem, ut.",
                SessionMode::Custom => "The quick, brown fox jumped over the lazy dog.",
            };

            session.set_original_text(text);

            session.set_type(match mode {
                SessionMode::Simple | SessionMode::Advanced => {
                    let time_string = self.time_dropdown.selected_item().unwrap().downcast_ref::<gtk::StringObject>().unwrap().string();
                    let duration = match time_string.as_str() {
                        "15 seconds" => Duration::from_secs(15),
                        "30 seconds" => Duration::from_secs(30),
                        "1 minute" => Duration::from_secs(60),
                        "5 minutes" => Duration::from_secs(5 * 60),
                        "10 minutes" => Duration::from_secs(10 * 60),
                        _ => panic!("non-existent duration set"),
                    };
                    SessionType::TimeBased(duration)
                }
                SessionMode::Custom => SessionType::LengthBased,
            })
        }

        fn focus_text_view(&self) {
            self.obj().set_focus_widget(Some(&self.text_view.get()));
        }
    }
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
