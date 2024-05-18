use super::*;
use crate::text_utils::{calculate_accuracy, calculate_wpm};
use strum::EnumMessage;

impl imp::KpWindow {
    pub(super) fn setup_stop_button(&self) {
        self.stop_button
            .connect_clicked(glib::clone!(@weak self as imp => move |_| {
                imp.ready(true);
            }));
    }

    pub(super) fn setup_continue_button(&self) {
        self.continue_button
            .connect_clicked(glib::clone!(@weak self as imp => move |_| {
                imp.ready(false);
            }));
    }

    pub(super) fn setup_ui_hiding(&self) {
        self.show_cursor.set(true);

        let device = self
            .obj()
            .display()
            .default_seat()
            .expect("display always has a default seat")
            .pointer()
            .expect("default seat has device");

        self.text_view.connect_typed_text_notify(
            glib::clone!(@weak self as imp, @strong device => move |_| {
                if imp.show_cursor.get() && imp.running.get() {
                    imp.header_bar_running.add_css_class("hide-controls");

                    imp.hide_cursor();
                }
            }),
        );

        let motion_ctrl = gtk::EventControllerMotion::new();
        motion_ctrl.connect_motion(glib::clone!(@weak self as imp, @strong device => move |_,_,_| {
            if !imp.show_cursor.get() && device.timestamp() > imp.cursor_hidden_timestamp.get() {
                imp.show_cursor();

                if imp.running.get() {
                    imp.header_bar_running.remove_css_class("hide-controls");
                }
            }
        }));

        self.obj().add_controller(motion_ctrl);
    }

    pub(super) fn ready(&self, animate: bool) {
        self.running.set(false);
        self.header_bar_running.add_css_class("hide-controls");
        self.text_view.set_running(false);
        self.text_view.set_accepts_input(true);
        self.main_stack.set_visible_child_name("session");
        self.header_stack.set_visible_child_name("ready");
        self.text_view.reset(animate);
        self.focus_text_view();

        self.update_original_text();
        self.update_time();
    }

    pub(super) fn start(&self) {
        self.running.set(true);
        self.start_time.set(Some(Instant::now()));
        self.main_stack.set_visible_child_name("session");
        self.header_stack.set_visible_child_name("running");
        self.hide_cursor();
        self.bottom_stack
            .set_visible_child(&self.bottom_stack_empty.get());
        self.header_bar_running.add_css_class("hide-controls");

        match self.session_type.get() {
            SessionType::Simple | SessionType::Advanced => self.start_timer(),
            SessionType::Custom => (),
        }
    }

    pub(super) fn finish(&self) {
        self.running.set(false);
        self.text_view.set_running(false);
        self.text_view.set_accepts_input(false);
        self.finish_time.set(Some(Instant::now()));
        self.show_results_view();
    }

    fn show_results_view(&self) {
        let continue_button = self.continue_button.get();
        let original_text = self.text_view.original_text();
        let typed_text = self.text_view.typed_text();
        let Some(start_time) = self.start_time.get() else {
            return;
        };
        let Some(finish_time) = self.finish_time.get() else {
            return;
        };

        let results_view = self.results_view.get();

        let duration = finish_time.duration_since(start_time);
        results_view.set_duration(duration.as_secs());

        let wpm = calculate_wpm(duration, &typed_text);
        results_view.set_wpm(wpm);

        let accuracy = calculate_accuracy(&original_text, &typed_text);
        results_view.set_accuracy(accuracy);

        let session_type = self.session_type.get();
        results_view.set_session_type(session_type.ui_string());

        let language = self
            .language
            .get()
            .get_message()
            .expect("all languages have names set");
        results_view.set_language(language);

        let show_language = matches!(session_type, SessionType::Simple | SessionType::Advanced);
        results_view.set_show_language(show_language);

        self.main_stack.set_visible_child_name("results");

        self.obj().set_focus_widget(None::<&gtk::Widget>);
        glib::timeout_add_local_once(
            Duration::from_millis(500),
            glib::clone!(@weak continue_button => move || {
                    continue_button.grab_focus();
                }
            ),
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
