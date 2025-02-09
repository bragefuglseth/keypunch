/* session.rs
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

use super::*;
use crate::session_enums::SessionSummary;
use crate::text_generation;
use crate::text_utils::{process_custom_text, GraphemeState};
use crate::widgets::{KpCustomTextDialog, KpTextLanguageDialog};
use gettextrs::gettext;
use glib::ControlFlow;
use i18n_format::i18n_fmt;
use std::iter::once;
use text_generation::CHUNK_GRAPHEME_COUNT;

// The lower this is, the more sensitive Keypunch is to "frustration" (random key mashing).
// If enough frustration is detected, the session will be cancelled, and a helpful
// message will be displayed.
const FRUSTRATION_THRESHOLD: usize = 3;

#[gtk::template_callbacks]
impl imp::KpWindow {
    pub(super) fn setup_session_config(&self) {
        let app = self.obj().kp_application();
        let settings = app.settings();

        let session_type_dropdown = self.session_type_dropdown.get();
        settings::bind_dropdown_selected(
            &settings,
            &session_type_dropdown,
            "session-type",
            settings::SESSION_TYPE_VALUES,
        );
        settings.connect_changed(
            Some("session-type"),
            glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _| {
                    imp.refresh_original_text();
                    imp.focus_text_view();
                }
            ),
        );

        settings.connect_changed(
            Some("text-language"),
            glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |_, _| {
                    imp.refresh_original_text();
                    imp.focus_text_view();
                }
            ),
        );

        settings.connect_changed(
            Some("custom-text"),
            glib::clone!(
                #[weak(rename_to=imp)]
                self,
                move |_, _| {
                    imp.refresh_original_text();
                    imp.focus_text_view();
                }
            ),
        );

        let duration_dropdown = self.duration_dropdown.get();

        settings::bind_dropdown_selected(
            &settings,
            &duration_dropdown,
            "session-duration",
            settings::SESSION_DURATION_VALUES,
        );

        duration_dropdown.connect_selected_item_notify(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |_| {
                imp.update_time();
                imp.focus_text_view();
            }
        ));

        self.custom_button.connect_clicked(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |_| {
                imp.show_custom_text_dialog(None);
            }
        ));
    }

    pub(super) fn setup_text_view(&self) {
        let text_view = self.text_view.get();

        text_view.connect_running_notify(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |text_view| {
                if text_view.running() {
                    imp.start();
                }
            }
        ));

        text_view.connect_local(
            "typed-text-changed",
            true,
            glib::clone!(
                #[weak(rename_to = imp)]
                self,
                #[upgrade_or_default]
                move |values| {
                    let Some(TypingSession { config, .. }) = imp.session.get() else {
                        return None;
                    };

                    let text_view = values.get(0).unwrap().get::<KpTextView>().unwrap();

                    let original_grapheme_count = text_view.original_grapheme_count();
                    let typed_grapheme_count = text_view.typed_grapheme_count();

                    if typed_grapheme_count >= original_grapheme_count {
                        if text_view.last_grapheme_state() != GraphemeState::Unfinished {
                            imp.finish();
                        }
                    }

                    if typed_grapheme_count
                        > original_grapheme_count
                            .checked_sub(CHUNK_GRAPHEME_COUNT / 2)
                            .unwrap_or(CHUNK_GRAPHEME_COUNT)
                    {
                        imp.extend_original_text(config);
                    }

                    if config == SessionConfig::Finite {
                        let (current_word, total_words) = text_view.progress();

                        // Translators: The `{}` blocks will be replaced with the current word count and the total word count.
                        // Do not translate them! The slash sign is a special unicode character, if your language doesn't
                        // use a completely different sign, you should probably copy and paste it from the original string.
                        imp.status_label.set_label(
                            &i18n_fmt! { i18n_fmt("{} ⁄ {}", current_word, total_words) },
                        );
                    }

                    let frustration_score = text_view
                        .keystrokes()
                        .iter()
                        .rev()
                        .take_while(|(timestamp, _)| {
                            timestamp.elapsed().as_secs() <= FRUSTRATION_THRESHOLD as u64
                        })
                        .filter(|(_, correct)| !*correct)
                        .count();

                    if frustration_score > FRUSTRATION_THRESHOLD * 10 {
                        imp.frustration_relief();
                    }

                    None
                }
            ),
        );
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
                    if imp.show_cursor.get() && imp.is_running() {
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

                    if imp.is_running() {
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

                    if imp.is_running() {
                        imp.obj().remove_css_class("hide-controls");
                    }
                }
            }
        ));
        obj.add_controller(click_gesture);
    }

    #[template_callback]
    pub(super) fn ready(&self) {
        self.session.set(None);
        self.text_view.set_running(false);
        self.text_view.set_accepts_input(true);
        self.main_stack.set_visible_child_name("session");
        self.status_stack.set_visible_child_name("ready");
        self.menu_button.set_visible(true);
        self.stop_button.set_visible(false);
        self.text_view.reset();
        self.focus_text_view();

        self.refresh_original_text();
        self.update_time();

        self.obj()
            .action_set_enabled("win.text-language-dialog", true);
        self.obj().action_set_enabled("win.cancel-session", false);
        self.obj().remove_css_class("hide-controls");

        let app = self.obj().kp_application();
        let settings = app.settings();

        // Discord IPC
        self.obj().kp_application().discord_rpc().set_activity(
            SessionConfig::from_settings(&settings),
            PresenceState::Ready,
        );

        self.end_existing_inhibit();
    }

    pub(super) fn start(&self) {
        let app = self.obj().kp_application();
        let settings = app.settings();

        let config = SessionConfig::from_settings(&settings);

        self.session.set(Some(TypingSession::new(config)));
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
                    if imp.is_running() {
                        imp.menu_button.set_visible(false);
                        imp.stop_button.set_opacity(1.);
                    }
                }
            ),
        );

        if matches!(config, SessionConfig::Generated { .. }) {
            self.start_timer(config);
        }

        self.obj()
            .action_set_enabled("win.text-language-dialog", false);
        self.obj().action_set_enabled("win.cancel-session", true);
        self.obj().add_css_class("hide-controls");

        // Discord IPC
        self.obj()
            .kp_application()
            .discord_rpc()
            .set_activity(config, PresenceState::Typing);

        // Translators: This is shown as a warning by GNOME Shell before logging out or shutting off the system in the middle of a typing session, alongside Keypunch's name and icon
        self.inhibit_session(&gettext("Ongoing typing session"))
    }

    pub(super) fn finish(&self) {
        let Some(session) = self.session.get() else {
            return;
        };

        self.end_session();
        self.show_results_view(session, Instant::now());

        let config = session.config;

        // Discord IPC
        self.obj()
            .kp_application()
            .discord_rpc()
            .set_activity(config, PresenceState::Results);
    }

    pub(super) fn frustration_relief(&self) {
        if !self.is_running() {
            return;
        }

        self.end_session();
        self.main_stack.set_visible_child_name("frustration-relief");

        // Avoid continue button being activated from a keypress immediately
        let continue_button = self.frustration_continue_button.get();
        self.obj().set_focus(None::<&gtk::Widget>);
        glib::timeout_add_local_once(
            Duration::from_millis(1000),
            glib::clone!(
                #[weak]
                continue_button,
                move || {
                    continue_button.grab_focus();
                }
            ),
        );
    }

    pub(super) fn end_session(&self) {
        self.session.set(None);
        self.text_view.set_running(false);
        self.text_view.set_accepts_input(false);

        self.obj()
            .action_set_enabled("win.text-language-dialog", false);
        self.obj().action_set_enabled("win.cancel-session", false);

        self.end_existing_inhibit();
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

    pub(super) fn refresh_original_text(&self) {
        if self.is_running() {
            return;
        }

        let app = self.obj().kp_application();
        let settings = app.settings();

        let config = SessionConfig::from_settings(&settings);

        let config_widget = match config {
            SessionConfig::Generated { .. } => self.duration_dropdown.get().upcast::<gtk::Widget>(),
            SessionConfig::Finite => self.custom_button.get().upcast::<gtk::Widget>(),
        };
        self.secondary_config_stack
            .set_visible_child(&config_widget);

        let new_original = match config {
            SessionConfig::Generated {
                language,
                difficulty,
                ..
            } => match difficulty {
                GeneratedSessionDifficulty::Simple => text_generation::simple(language),
                GeneratedSessionDifficulty::Advanced => text_generation::advanced(language),
            },
            SessionConfig::Finite => process_custom_text(&settings.string("custom-text")),
        };
        self.text_view.set_original_text(&new_original);

        // Discord IPC
        self.obj()
            .kp_application()
            .discord_rpc()
            .set_activity(config, PresenceState::Ready);
    }

    // TODO: is this needed?
    pub(super) fn update_time(&self) {
        let app = self.obj().kp_application();
        let settings = app.settings();

        let config = SessionConfig::from_settings(&settings);

        // Discord IPC
        self.obj()
            .kp_application()
            .discord_rpc()
            .set_activity(config, PresenceState::Ready);
    }

    pub(super) fn show_text_language_dialog(&self) {
        if self.is_running() || self.obj().visible_dialog().is_some() {
            return;
        }

        let app = self.obj().kp_application();
        let settings = app.settings();

        let dialog = KpTextLanguageDialog::new(&settings);

        dialog.connect_closed(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |_| {
                imp.focus_text_view();
            }
        ));

        dialog.present(Some(self.obj().upcast_ref::<gtk::Widget>()));
    }

    pub fn show_custom_text_dialog(&self, initial_override: Option<&str>) {
        if self.is_running() || self.obj().visible_dialog().is_some() {
            return;
        }

        let app = self.obj().kp_application();
        let settings = app.settings();

        let dialog = KpCustomTextDialog::new(&settings, initial_override);

        dialog.connect_local(
            "discard",
            true,
            glib::clone!(
                #[weak(rename_to = imp)]
                self,
                #[upgrade_or_default]
                move |values| {
                    let discarded_text: String = values
                        .get(1)
                        .expect("save signal contains text to be saved")
                        .get::<&str>()
                        .expect("value from save signal is string")
                        .into();

                    let toast = adw::Toast::builder()
                        .title(&gettext("Changes discarded"))
                        .button_label(&gettext("Restore"))
                        .build();

                    toast.connect_button_clicked(glib::clone!(
                        #[weak]
                        imp,
                        move |_| {
                            imp.show_custom_text_dialog(Some(&discarded_text));
                        }
                    ));

                    imp.toast_overlay.add_toast(toast);

                    None
                }
            ),
        );

        dialog.connect_closed(glib::clone!(
            #[weak(rename_to = imp)]
            self,
            move |_| {
                imp.focus_text_view();
            }
        ));

        dialog.present(Some(self.obj().upcast_ref::<gtk::Widget>()));
    }

    pub(super) fn extend_original_text(&self, config: SessionConfig) {
        let SessionConfig::Generated {
            language,
            difficulty,
            ..
        } = config
        else {
            return;
        };

        let new_chunk = match difficulty {
            GeneratedSessionDifficulty::Simple => text_generation::simple(language),
            GeneratedSessionDifficulty::Advanced => text_generation::advanced(language),
        };
        self.text_view.push_original_text(&new_chunk);
    }

    pub(super) fn start_timer(&self, config: SessionConfig) {
        let SessionConfig::Generated { duration, .. } = config else {
            return;
        };

        let duration = match duration {
            SessionDuration::Sec15 => Duration::from_secs(15),
            SessionDuration::Sec30 => Duration::from_secs(30),
            SessionDuration::Min1 => Duration::from_secs(60),
            SessionDuration::Min5 => Duration::from_secs(5 * 60),
            SessionDuration::Min10 => Duration::from_secs(10 * 60),
        };

        self.update_timer(duration.as_secs() + 1);

        glib::timeout_add_local(
            Duration::from_millis(100),
            glib::clone!(
                #[weak(rename_to = imp)]
                self,
                #[strong]
                duration,
                #[upgrade_or]
                ControlFlow::Break,
                move || {
                    let Some(TypingSession { start_instant, .. }) = imp.session.get() else {
                        return ControlFlow::Break;
                    };

                    if let Some(diff) = duration.checked_sub(start_instant.elapsed()) {
                        let seconds = diff.as_secs() + 1;

                        // add trailing zero for second values below 10
                        imp.update_timer(seconds);
                        ControlFlow::Continue
                    } else {
                        imp.finish();
                        ControlFlow::Break
                    }
                }
            ),
        );
    }

    fn update_timer(&self, seconds: u64) {
        // add trailing zero for second values below 10
        let text = if seconds >= 60 && seconds % 60 < 10 {
            // Translators: The format of the timer. The first `{}` block will be replaced
            // with the minutes passed, and the second one will be replaced with the seconds
            // passed. Do not translate the `{}` blocks. Note that the `∶` sign is a special
            // Unicode character; if your language doesn't use something completely different,
            // you should probably copy and paste it from the original string.
            i18n_fmt! { i18n_fmt("{}∶{}", seconds / 60, format!("0{}", seconds % 60)) }
        } else if seconds >= 60 {
            i18n_fmt! { i18n_fmt("{}∶{}", seconds / 60, seconds % 60) }
        } else {
            seconds.to_string()
        };

        self.status_label.set_label(&text);
    }

    pub(super) fn show_results_view(&self, session: TypingSession, finish_instant: Instant) {
        let continue_button = self.results_continue_button.get();

        let TypingSession {
            config,
            start_instant,
            start_system_time,
        } = session;

        let original_text = match config {
            SessionConfig::Generated { .. } => self.text_view.original_text(),
            SessionConfig::Finite => process_custom_text(&self.text_view.original_text()),
        };
        let typed_text = self.text_view.typed_text();

        let results_view = self.results_view.get();

        let keystrokes = self.text_view.keystrokes();

        let summary = SessionSummary::new(
            start_system_time,
            start_instant,
            finish_instant,
            config,
            &original_text,
            &typed_text,
            &keystrokes,
        );

        results_view.set_summary(summary);

        let app = self.obj().kp_application();
        let settings = app.settings();

        let personal_best_vec: Vec<(String, String, String, u32)> = settings
            .value("personal-best")
            .get()
            .unwrap_or_else(|| Vec::new());

        if let SessionConfig::Generated {
            language,
            difficulty,
            duration,
        } = config
        {
            let is_personal_best = summary.accuracy > 0.9
                && personal_best_vec
                    .iter()
                    .find(|(stored_difficulty, duration, lang_code, _)| {
                        *stored_difficulty == difficulty.to_string()
                            && *duration == duration.to_string()
                            && *lang_code == language.to_string()
                    })
                    .map(|(_, _, _, best_wpm)| summary.wpm.floor() as u32 > *best_wpm)
                    .unwrap_or(true);

            if is_personal_best {
                let new_personal_best_vec = add_personal_best(
                    personal_best_vec,
                    (
                        &difficulty.to_string(),
                        &duration.to_string(),
                        &language.to_string(),
                        summary.wpm.floor() as u32,
                    ),
                );

                settings
                    .set_value("personal-best", &new_personal_best_vec.to_variant())
                    .expect("can update stored personal best values");

                results_view.set_show_personal_best(true);
            }
        }

        self.main_stack.set_visible_child_name("results");

        self.obj().set_focus(None::<&gtk::Widget>);
        glib::timeout_add_local_once(
            Duration::from_millis(500),
            glib::clone!(
                #[weak]
                continue_button,
                move || {
                    continue_button.grab_focus();
                }
            ),
        );

        // Discord IPC
        self.obj()
            .kp_application()
            .discord_rpc()
            .set_stats(summary.wpm, summary.accuracy);
    }
}

pub(super) fn add_personal_best(
    old: Vec<(String, String, String, u32)>,
    new: (&str, &str, &str, u32),
) -> Vec<(String, String, String, u32)> {
    let (new_session_type, new_duration, new_language, new_wpm) = new;

    old.into_iter()
        .filter(
            |(stored_session_type, stored_duration, stored_lang_code, _)| {
                *stored_session_type != new_session_type
                    || *stored_duration != new_duration
                    || *stored_lang_code != new_language
            },
        )
        .chain(once((
            new_session_type.to_string(),
            new_duration.to_string(),
            new_language.to_string(),
            new_wpm,
        )))
        .collect()
}
