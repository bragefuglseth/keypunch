/* text_view.rs
 *
 * SPDX-FileCopyrightText: © 2024–2025 Brage Fuglseth <bragefuglseth@gnome.org>
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

mod accessibility;
mod caret;
mod colors;
mod input;
mod scrolling;

use crate::text_utils::{
    current_word, insert_replacements, validate_with_replacements, GraphemeState,
};
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::glib;
use gtk::{gdk, gsk};
use std::cell::{Cell, OnceCell, Ref, RefCell};
use std::sync::OnceLock;
use std::time::Instant;
use unicode_segmentation::UnicodeSegmentation;

const LINE_HEIGHT: i32 = 50;

#[derive(PartialEq)]
enum TextChange {
    Addition,
    Removal,
    Replacement,
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/widgets/text_view.blp")]
    #[properties(wrapper_type = super::KpTextView)]
    pub struct KpTextView {
        #[template_child]
        pub(super) text_view: TemplateChild<gtk::TextView>,

        #[property(get, set=Self::set_caret_x)]
        pub(super) caret_x: Cell<f64>,
        #[property(get, set=Self::set_caret_y)]
        pub(super) caret_y: Cell<f64>,
        #[property(get, set)]
        pub(super) caret_height: Cell<f64>,
        #[property(get, set)]
        pub(super) running: Cell<bool>,
        #[property(get, set)]
        pub(super) accepts_input: Cell<bool>,

        pub(super) original_text: RefCell<String>,
        pub(super) typed_text: RefCell<String>,
        pub(super) previous_preedit: RefCell<String>,
        pub(super) keystrokes: RefCell<Vec<(Instant, bool)>>,
        pub(super) input_context: RefCell<Option<gtk::IMMulticontext>>,
        pub(super) scroll_animation: OnceCell<adw::TimedAnimation>,
        pub(super) caret_x_animation: OnceCell<adw::TimedAnimation>,
        pub(super) caret_y_animation: OnceCell<adw::TimedAnimation>,
        pub(super) caret_rgb: Cell<(f32, f32, f32)>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpTextView {
        const NAME: &'static str = "KpTextView";
        type Type = super::KpTextView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("KpTextView");

            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for KpTextView {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("typed-text-changed").build()])
        }

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

            let text_view = self.text_view.get();
            text_view.set_bottom_margin(LINE_HEIGHT);
            text_view.set_can_target(false);

            obj.connect_has_focus_notify(|obj| {
                let imp = obj.imp();
                imp.update_scroll_position(true);
            });

            self.setup_input_handling();
            self.setup_color_scheme();
            self.compare_and_update_colors();
            self.update_scroll_position(true);
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for KpTextView {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Vertical => (LINE_HEIGHT * 3, LINE_HEIGHT * 3, -1, -1),
                gtk::Orientation::Horizontal => self.text_view.measure(orientation, for_size),
                _ => panic!("orientation type not accounted for"),
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.text_view.allocate(width, height, baseline, None);
            self.update_scroll_position(true);
            self.update_caret_position(true);
            self.compare_and_update_colors();
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.obj().snapshot_child(&self.text_view.get(), snapshot);

            let (caret_path, caret_stroke, caret_color) = self.caret_stroke_data();
            snapshot.append_stroke(&caret_path, &caret_stroke, &caret_color);
        }
    }

    impl KpTextView {
        // TODO: Check if this can use `typed_text_changed` instead to streamline
        pub fn set_original_text(&self, text: &str) {
            *self.original_text.borrow_mut() = text.to_string();
            self.text_view
                .buffer()
                .set_text(&insert_replacements(&text));
            self.compare_and_update_colors();
            self.update_caret_position(true);
            self.update_scroll_position(true);
            self.update_accessible_state();
        }

        pub fn push_original_text(&self, text: &str) {
            self.original_text.borrow_mut().push_str(text);

            let buffer = self.text_view.buffer();
            buffer.insert(&mut buffer.end_iter(), &insert_replacements(&text));
            self.compare_and_update_colors();
        }

        pub(super) fn typed_text_changed(&self, change: TextChange) {
            let input_context = self.input_context.borrow();
            let (preedit, _, _) = input_context.as_ref().unwrap().preedit_string();

            let comparison = validate_with_replacements(
                &self.original_text.borrow(),
                &self.typed_text.borrow(),
                preedit.as_str().graphemes(true).count(),
            );

            if change == TextChange::Addition {
                let last_grapheme_state = comparison
                    .iter()
                    .last()
                    .map(|(state, _, _, _)| *state)
                    .unwrap_or(GraphemeState::Unfinished);

                let correct = last_grapheme_state != GraphemeState::Mistake;
                let keystroke = (Instant::now(), correct);
                self.keystrokes.borrow_mut().push(keystroke);
            } else if change == TextChange::Removal {
                // If text is removed, it's always a "correct" stroke
                let keystroke = (Instant::now(), true);
                self.keystrokes.borrow_mut().push(keystroke);
            }

            self.update_colors(&comparison);
            self.update_caret_position(!self.running.get());
            self.update_scroll_position(!self.running.get());
            self.update_accessible_state();

            self.obj().emit_by_name::<()>("typed-text-changed", &[]);
        }
    }
}

glib::wrapper! {
    pub struct KpTextView(ObjectSubclass<imp::KpTextView>)
        @extends gtk::Widget, @implements gtk::Accessible;
}

impl KpTextView {
    pub fn original_text(&self) -> String {
        self.imp().original_text.borrow().to_string()
    }

    pub fn typed_text(&self) -> String {
        self.imp().typed_text.borrow().to_string()
    }

    pub fn set_original_text(&self, text: &str) {
        self.imp().set_original_text(text);
    }

    pub fn push_original_text(&self, text: &str) {
        self.imp().push_original_text(text);
    }

    pub fn set_typed_text(&self, text: &str) {
        *self.imp().typed_text.borrow_mut() = text.to_string();
        self.imp().typed_text_changed(TextChange::Replacement);
        self.imp().input_context.borrow().as_ref().unwrap().reset();
    }

    pub fn original_grapheme_count(&self) -> usize {
        self.imp()
            .original_text
            .borrow()
            .as_str()
            .graphemes(true)
            .count()
    }

    pub fn typed_grapheme_count(&self) -> usize {
        self.imp()
            .typed_text
            .borrow()
            .as_str()
            .graphemes(true)
            .count()
    }

    pub fn last_grapheme_state(&self) -> GraphemeState {
        let imp = self.imp();

        validate_with_replacements(
            &imp.original_text.borrow().as_str(),
            &imp.typed_text.borrow().as_str(),
            0,
        )
        .last()
        .map(|(state, _, _, _)| *state)
        .unwrap_or(GraphemeState::Unfinished)
    }

    // Returns a tuple with (current word, total word count)
    pub fn progress(&self) -> (usize, usize) {
        let imp = self.imp();

        let current_word = current_word(
            imp.original_text.borrow().as_str(),
            imp.typed_text.borrow().as_str().graphemes(true).count(),
        );
        let total_words = imp.original_text.borrow().as_str().unicode_words().count();

        (current_word, total_words)
    }

    // Returns a tuple with the amount of correct keystrokes and the total amount of keystrokes
    pub fn keystrokes(&self) -> Ref<Vec<(Instant, bool)>> {
        self.imp().keystrokes.borrow()
    }

    pub fn reset(&self) {
        self.set_original_text("");
        self.set_typed_text("");
        self.set_running(false);

        let imp = self.imp();
        imp.scroll_animation().skip();
        imp.caret_x_animation().skip();
        imp.caret_y_animation().skip();
        imp.keystrokes.borrow_mut().clear();
    }
}
