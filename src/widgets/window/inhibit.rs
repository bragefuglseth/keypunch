/* inhibit.rs
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

impl imp::KpWindow {
    pub(super) fn inhibit_session(&self, reason: &str) {
        self.end_existing_inhibit();

        let obj = self.obj();
        let app = obj
            .application()
            .expect("the window always has an associated app");

        let cookie = app.inhibit(
            Some(self.obj().upcast_ref::<gtk::Window>()),
            gtk::ApplicationInhibitFlags::LOGOUT,
            Some(&reason),
        );

        self.inhibit_cookie.set(Some(cookie));
    }

    pub(super) fn end_existing_inhibit(&self) {
        if let Some(cookie) = self.inhibit_cookie.take() {
            self.obj()
                .application()
                .expect("the window always has an associated app")
                .uninhibit(cookie);
        }
    }
}
