/* prompt_dialog.rs
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

 use adw::prelude::*;
 use adw::subclass::prelude::*;
 use glib::subclass::Signal;
 use gtk::{gio, glib};
 use std::cell::{Cell, RefCell};
 use std::sync::OnceLock;

mod imp {
    use super::*;
    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/widgets/prompt_dialog.blp")]
    #[properties(wrapper_type = super::KpPromptDialog)]
    pub struct KpPromptDialog {
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub placeholder: TemplateChild<gtk::Label>,
        #[template_child]
        pub text_view: TemplateChild<gtk::TextView>,
        #[template_child]
        pub generate_button: TemplateChild<gtk::Button>,

        pub current_text: RefCell<String>,

        pub apply_changes: Cell<bool>,

        #[property(get, construct_only, nullable)]
        pub settings: RefCell<Option<gio::Settings>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpPromptDialog {
        const NAME: &'static str = "KpPromptDialog";
        type Type = super::KpPromptDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for KpPromptDialog {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("generate")
                        .param_types([str::static_type()])
                        .build(),
                    Signal::builder("discard")
                        .param_types([str::static_type()])
                        .build(),
                ]
            })
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

            self.text_view
                .buffer()
                .bind_property("text", &self.placeholder.get(), "visible")
                .transform_to(|_, text: String| Some(text.is_empty()))
                .sync_create()
                .build();

            let generate_button = self.generate_button.get();
            self.text_view
                .buffer()
                .bind_property("text", &generate_button, "sensitive")
                .transform_to(|_, text: String| {
                    let has_content = text.chars().any(|c| !c.is_whitespace());
                    Some(has_content)
                })
                .sync_create()
                .build();

                generate_button.connect_clicked(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.apply_changes.set(true);
                    imp.obj()
                        .emit_by_name_with_values("generate", &[imp.text().into()]);
                    imp.obj().close();
                }
            ));
        }
    }
    impl WidgetImpl for KpPromptDialog {}
    impl AdwDialogImpl for KpPromptDialog {
        fn closed(&self) {
            if self.changed() && !self.apply_changes.get() {
                self.obj()
                    .emit_by_name_with_values("discard", &[self.text().into()]);
            }
        }
    }

    impl KpPromptDialog {
        fn changed(&self) -> bool {
            self.current_text.borrow().as_str() != self.text()
        }

        fn text(&self) -> String {
            let buf = self.text_view.buffer();
            buf.text(&buf.start_iter(), &buf.end_iter(), false)
                .to_string()
        }
    }
}

glib::wrapper! {
    pub struct KpPromptDialog(ObjectSubclass<imp::KpPromptDialog>)
        @extends gtk::Widget, adw::Dialog;
}

impl KpPromptDialog {
    pub fn new(settings: &gio::Settings) -> Self {
        let obj: Self = glib::Object::builder().build();

        let imp = obj.imp();

        let current_prompt = settings.string("prompt");

        *imp.current_text.borrow_mut() = current_prompt.to_string();
        imp.text_view.buffer().set_text(&current_prompt.to_string());
        imp.text_view
            .emit_by_name_with_values("select-all", &[true.into()]);
        obj
    }
}
