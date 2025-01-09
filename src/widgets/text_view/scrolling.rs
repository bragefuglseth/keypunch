/* scrolling.rs
 *
 * SPDX-FileCopyrightText: © 2024 Brage Fuglseth
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
use crate::text_utils::line_offset_with_replacements;
use unicode_segmentation::UnicodeSegmentation;

impl imp::KpTextView {
    pub(super) fn scroll_animation(&self) -> adw::TimedAnimation {
        self.scroll_animation
            .get_or_init(|| {
                let text_view = self.text_view.get();
                let vadjustment = self
                    .text_view
                    .vadjustment()
                    .expect("text view has vadjustment");

                adw::TimedAnimation::builder()
                    .duration(300)
                    .widget(&text_view)
                    .target(&adw::PropertyAnimationTarget::new(&vadjustment, "value"))
                    .build()
            })
            .clone()
    }

    // Updates the scroll position according to the text view and the length of the typed text so far.
    // If `force` is true, the change will be made unconditionally and without an animation.
    pub(super) fn update_scroll_position(&self, force: bool) {
        let obj = self.obj();

        let original = self.original_text.borrow();
        let typed = self.typed_text.borrow();

        let input_context = self.input_context.borrow();
        let (preedit, _, _) = input_context.as_ref().unwrap().preedit_string();

        // Validation is performed on typed text with one added character, to get the start index
        // of the next character.
        let (caret_line, caret_idx) =
            line_offset_with_replacements(&original, &typed, preedit.graphemes(true).count());

        let text_view = self.text_view.get();
        let buf = text_view.buffer();

        let iter = buf
            .iter_at_line_index(caret_line as i32, caret_idx as i32)
            .unwrap_or(buf.end_iter());
        let location = text_view.iter_location(&iter);

        let snap_to_top = {
            let mut control_iter = iter;

            // If we can't move the iter backwards two lines, that must mean that we're
            // at line 1 or 2. To prevent any form of slight scrolling when moving from line 1 to 2,
            // we snap the text view to the top in that case. This is necessary because some scripts
            // can have very tall letters.
            text_view.backward_display_line(&mut control_iter);

            !text_view.backward_display_line(&mut control_iter) || typed.is_empty()
        };

        let y = if snap_to_top {
            0.
        } else {
            (location.y() + location.height() / 2)
                .checked_sub(obj.height() / 2)
                .unwrap_or(0) as f64
        };

        let current_position = self
            .text_view
            .vadjustment()
            .expect("text view always has vadjustment")
            .value();

        let scroll_animation = self.scroll_animation();
        if force {
            self.text_view
                .vadjustment()
                .expect("text view should have vadjustment")
                .set_value(y);
        } else {
            let line_has_changed = (scroll_animation.value_to() - y).abs() > 10.;

            if line_has_changed {
                scroll_animation.set_value_from(current_position);
                scroll_animation.set_value_to(y);
                scroll_animation.play();
            }
        }
    }
}
