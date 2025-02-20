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
use gtk::glib;
use std::sync::OnceLock;
use url::Url;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "src/widgets/ollama_config_dialog.blp")]
    pub struct KpOllamaConfigDialog {
        #[template_child]
        pub ollama_url: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub model_name: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub save_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpOllamaConfigDialog {
        const NAME: &'static str = "KpOllamaConfigDialog";
        type Type = super::KpOllamaConfigDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for KpOllamaConfigDialog {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("save").build()])
        }

        fn constructed(&self) {
            self.parent_constructed();

            let save_button = self.save_button.get();
            save_button.connect_clicked(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.obj().emit_by_name::<()>("save", &[]);
                    imp.obj().close();
                }
            ));
        }
    }
    impl WidgetImpl for KpOllamaConfigDialog {}
    impl AdwDialogImpl for KpOllamaConfigDialog {}
    impl KpOllamaConfigDialog {}
}

glib::wrapper! {
    pub struct KpOllamaConfigDialog(ObjectSubclass<imp::KpOllamaConfigDialog>)
        @extends gtk::Widget, adw::Dialog;
}

impl KpOllamaConfigDialog {
    pub fn new(ollama_url: &Url, model_name: &str) -> Self {
        let obj = glib::Object::new::<Self>();
        let imp = obj.imp();

        imp.ollama_url.set_text(&ollama_url.to_string());
        imp.model_name.set_text(model_name);
        obj
    }

    pub fn get_ollama_url(&self) -> Url {
        //TODO input validation. string should be url type
        let url_string = self.imp().ollama_url.text().to_string();
        Url::parse(&url_string).expect("Error parsing URL")
    }

    pub fn get_model_name(&self) -> String {
        self.imp().model_name.text().to_string()
    }
}
