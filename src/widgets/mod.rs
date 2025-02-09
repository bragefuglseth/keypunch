/* mod.rs
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

mod custom_text_dialog;
mod results_view;
mod text_language_dialog;
mod text_view;
mod window;

pub use custom_text_dialog::KpCustomTextDialog;
pub use results_view::KpResultsView;
pub use text_language_dialog::KpTextLanguageDialog;
pub use text_view::KpTextView;
pub use window::KpWindow;
