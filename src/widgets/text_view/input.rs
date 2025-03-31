/* input.rs
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

use super::*;
use crate::text_utils::{end_alias, pop_grapheme_in_place, pop_word_in_place};

impl imp::KpTextView {
    pub(super) fn setup_input_handling(&self) {
        let obj = self.obj();
        let input_context = gtk::IMMulticontext::new();

        input_context.set_client_widget(Some(&*obj.upcast_ref::<gtk::Widget>()));

        input_context.connect_commit(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |_, text| {
                let obj = imp.obj();

                if obj.accepts_input() {
                    if !obj.running() {
                        obj.set_running(true);
                    }

                    imp.push_typed_text(text);
                }
            }
        ));

        input_context.connect_preedit_changed(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |ctx| {
                let obj = imp.obj();

                let (preedit, _, _) = ctx.preedit_string();
                let preedit = preedit.as_str();
                let preedit_has_changed = imp.previous_preedit.borrow().as_str() != preedit;

                if preedit_has_changed && obj.accepts_input() {
                    if !obj.running() {
                        obj.set_running(true);
                    }

                    *imp.previous_preedit.borrow_mut() = preedit.to_string();
                    imp.typed_text_changed(TextChange::Addition);
                }
            }
        ));

        input_context.connect_retrieve_surrounding(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            #[upgrade_or_default]
            move |ctx| {
                let current_typed = imp.typed_text.borrow();
                let typed_len = current_typed.len() as i32;
                ctx.set_surrounding_with_selection(&current_typed, typed_len, typed_len);
                true
            }
        ));

        input_context.connect_delete_surrounding(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            #[upgrade_or_default]
            move |_, offset, _| {
                // The cursor will always be at the end of the typed text,
                // so we can safely just pop the {offset} last characters
                imp.pop_typed_text(offset.abs() as usize);
                true
            }
        ));

        obj.connect_has_focus_notify(glib::clone!(
            #[weak]
            input_context,
            move |obj| {
                if obj.has_focus() {
                    input_context.focus_in();
                } else {
                    input_context.focus_out();
                }
            }
        ));

        input_context.set_input_hints(gtk::InputHints::NO_SPELLCHECK);

        let click_gesture = gtk::GestureClick::new();
        click_gesture.connect_released(glib::clone!(
            #[weak]
            input_context,
            move |controller, _, _, _| {
                input_context.activate_osk(controller.current_event());
            }
        ));
        self.obj().add_controller(click_gesture);

        let event_controller = gtk::EventControllerKey::new();
        event_controller.set_im_context(Some(&input_context));

        event_controller.connect_key_pressed(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            #[upgrade_or]
            glib::signal::Propagation::Proceed,
            move |controller, key, _, modifier| {
                if imp.typed_text.borrow().is_empty() { return glib::signal::Propagation::Proceed; }

                let obj = imp.obj();

                match (obj.accepts_input(), key) {
                    (true, gdk::Key::BackSpace)
                        if modifier.contains(gdk::ModifierType::CONTROL_MASK) =>
                    {
                        imp.pop_typed_text_word();
                        glib::signal::Propagation::Stop
                    }
                    (true, gdk::Key::BackSpace) => {
                        imp.pop_typed_text(1);
                        glib::signal::Propagation::Stop
                    }
                    (true, gdk::Key::Return) => {
                        controller
                            .im_context()
                            .expect("input controller has im context")
                            .emit_by_name_with_values("commit", &["\n".into()]);
                        glib::signal::Propagation::Stop
                    }
                    _ => glib::signal::Propagation::Proceed,
                }
            }
        ));

        self.obj().add_controller(event_controller);
        self.input_context.replace(Some(input_context));
    }

    fn push_typed_text(&self, s: &str) {
        self.typed_text.borrow_mut().push_str(s);

        let alias_opt = end_alias(&self.original_text.borrow(), &self.typed_text.borrow());
        if let Some((letter, potential_alias, true)) = alias_opt {
            for _ in 0..potential_alias.chars().count() {
                self.typed_text.borrow_mut().pop();
            }
            self.typed_text.borrow_mut().push_str(&letter);
        }

        self.typed_text_changed(TextChange::Addition);
    }

    fn pop_typed_text(&self, graphemes: usize) {
        pop_grapheme_in_place(&mut self.typed_text.borrow_mut(), graphemes);

        self.typed_text_changed(TextChange::Removal);
    }

    fn pop_typed_text_word(&self) {
        pop_word_in_place(
            &self.original_text.borrow(),
            &mut self.typed_text.borrow_mut(),
        );

        self.typed_text_changed(TextChange::Removal);
    }
}
