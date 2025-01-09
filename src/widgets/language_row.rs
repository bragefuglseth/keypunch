/* language_row.rs
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

use crate::text_generation::Language;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::{Cell, OnceCell};
use strum::EnumMessage;

mod imp {
    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::KpLanguageRow)]
    pub struct KpLanguageRow {
        pub check_button: OnceCell<gtk::CheckButton>,
        pub language: Cell<Language>,

        #[property(get, set)]
        pub checked: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpLanguageRow {
        const NAME: &'static str = "KpLanguageRow";
        type Type = super::KpLanguageRow;
        type ParentType = adw::ActionRow;
    }

    impl ObjectImpl for KpLanguageRow {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            let check_button = self.check_button();
            obj.add_prefix(&check_button);
            obj.set_activatable_widget(Some(&check_button));

            check_button
                .bind_property("active", obj.upcast_ref::<glib::Object>(), "checked")
                .bidirectional()
                .sync_create()
                .build();
        }
    }
    impl WidgetImpl for KpLanguageRow {}
    impl ListBoxRowImpl for KpLanguageRow {}
    impl PreferencesRowImpl for KpLanguageRow {}
    impl ActionRowImpl for KpLanguageRow {}

    impl KpLanguageRow {
        pub(super) fn check_button(&self) -> gtk::CheckButton {
            self.check_button
                .get_or_init(|| {
                    gtk::CheckButton::builder()
                        .valign(gtk::Align::Center)
                        .build()
                })
                .to_owned()
        }

        pub(super) fn set_language(&self, language: Language) {
            self.language.set(language);
            self.obj().set_title(
                language
                    .get_message()
                    .expect("all languages have names set"),
            );
        }
    }
}

glib::wrapper! {
    pub struct KpLanguageRow(ObjectSubclass<imp::KpLanguageRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow;
}

impl KpLanguageRow {
    pub fn new(language: Language) -> Self {
        let obj = glib::Object::new::<Self>();
        obj.imp().set_language(language);
        obj
    }

    pub fn language(&self) -> Language {
        self.imp().language.get()
    }

    pub fn check_button(&self) -> gtk::CheckButton {
        self.imp().check_button()
    }
}
