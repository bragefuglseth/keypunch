/* ui_state.rs
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
use crate::application::KpApplication;

impl imp::KpWindow {
    pub(super) fn setup_stop_button(&self) {
        self.stop_button.connect_clicked(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |_| {
                imp.ready();
            }
        ));
    }

    pub(super) fn setup_continue_button(&self) {
        self.continue_button.connect_clicked(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |_| {
                imp.ready();
            }
        ));
    }

    pub(super) fn setup_ui_hiding(&self) {
        let obj = self.obj();

        self.show_cursor.set(true);

        let device = obj
            .display()
            .default_seat()
            .expect("display always has a default seat")
            .pointer()
            .expect("default seat has device");

        self.text_view.connect_local(
            "typed-text-changed",
            true,
            glib::clone!(
                #[weak(rename_to = imp)]
                self,
                #[upgrade_or_default]
                move |_| {
                    if imp.show_cursor.get() && imp.running.get() {
                        imp.obj().add_css_class("hide-controls");
                        imp.hide_cursor();
                    }

                    None
                }
            ),
        );

        let motion_ctrl = gtk::EventControllerMotion::new();
        motion_ctrl.connect_motion(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            #[strong]
            device,
            move |_, _, _| {
                if !imp.show_cursor.get() && device.timestamp() > imp.cursor_hidden_timestamp.get()
                {
                    imp.show_cursor();

                    if imp.running.get() {
                        imp.obj().remove_css_class("hide-controls");
                    }
                }
            }
        ));
        obj.add_controller(motion_ctrl);

        let click_gesture = gtk::GestureClick::new();
        click_gesture.connect_released(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |_, _, _, _| {
                if !imp.show_cursor.get() {
                    imp.show_cursor();

                    if imp.running.get() {
                        imp.obj().remove_css_class("hide-controls");
                    }
                }
            }
        ));
        obj.add_controller(click_gesture);
    }

    pub(super) fn ready(&self) {
        self.running.set(false);
        self.text_view.set_running(false);
        self.text_view.set_accepts_input(true);
        self.main_stack.set_visible_child_name("session");
        self.status_stack.set_visible_child_name("ready");
        self.menu_button.set_visible(true);
        self.stop_button.set_visible(false);
        self.text_view.reset();
        self.focus_text_view();

        self.update_original_text();
        self.update_time();

        self.obj()
            .action_set_enabled("win.text-language-dialog", true);
        self.obj().action_set_enabled("win.cancel-session", false);
        self.obj().remove_css_class("hide-controls");

        // Discord IPC
        self.obj()
            .application()
            .expect("ready() isn't called before window has been paired with application")
            .downcast_ref::<KpApplication>()
            .unwrap()
            .discord_rpc()
            .set_activity(
                self.session_type.get(),
                self.duration.get(),
                PresenceState::Ready,
            );

        self.end_existing_inhibit();
    }

    pub(super) fn start(&self) {
        self.running.set(true);
        self.start_time.set(Some(Instant::now()));
        self.main_stack.set_visible_child_name("session");
        self.status_stack.set_visible_child_name("running");
        self.hide_cursor();
        self.bottom_stack
            .set_visible_child(&self.bottom_stack_empty.get());

        // Ugly hack to stop the stop button from "flashing" when starting a session:
        // Make it visible with 0 opacity, and set the opacity to 1 after the 200ms
        // crossfade effect has finished
        self.stop_button.set_opacity(0.);
        self.stop_button.set_visible(true);

        glib::timeout_add_local_once(
            Duration::from_millis(200),
            glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move || {
                    if imp.running.get() {
                        imp.menu_button.set_visible(false);
                        imp.stop_button.set_opacity(1.);
                    }
                }
            ),
        );

        match self.session_type.get() {
            SessionType::Simple | SessionType::Advanced => self.start_timer(),
            SessionType::Custom => (),
        }

        self.obj()
            .action_set_enabled("win.text-language-dialog", false);
        self.obj().action_set_enabled("win.cancel-session", true);
        self.obj().add_css_class("hide-controls");

        // Discord IPC
        self.obj()
            .application()
            .expect("ready() isn't called before window has been paired with application")
            .downcast_ref::<KpApplication>()
            .unwrap()
            .discord_rpc()
            .set_activity(
                self.session_type.get(),
                self.duration.get(),
                PresenceState::Typing,
            );

        // Translators: This is shown as a warning by GNOME Shell before logging out or shutting off the system in the middle of a typing session, alongside Keypunch's name and icon
        self.inhibit_session(&gettext("Ongoing typing session"))
    }

    pub(super) fn finish(&self) {
        if !self.running.get() {
            return;
        }

        self.running.set(false);
        self.text_view.set_running(false);
        self.text_view.set_accepts_input(false);
        self.finish_time.set(Some(Instant::now()));
        self.show_results_view();
        self.stop_button.set_visible(false);

        self.obj()
            .action_set_enabled("win.text-language-dialog", false);
        self.obj().action_set_enabled("win.cancel-session", false);

        self.end_existing_inhibit();

        // Discord IPC
        self.obj()
            .application()
            .expect("ready() isn't called before window has been paired with application")
            .downcast_ref::<KpApplication>()
            .unwrap()
            .discord_rpc()
            .set_activity(
                self.session_type.get(),
                self.duration.get(),
                PresenceState::Results,
            );
    }

    pub(super) fn hide_cursor(&self) {
        let device = self
            .obj()
            .display()
            .default_seat()
            .expect("display always has a default seat")
            .pointer()
            .expect("default seat has device");

        self.show_cursor.set(false);
        self.cursor_hidden_timestamp.set(device.timestamp());
        self.obj().set_cursor_from_name(Some("none"));
    }

    pub(super) fn show_cursor(&self) {
        self.show_cursor.set(true);
        self.obj().set_cursor_from_name(Some("default"));
    }
}
