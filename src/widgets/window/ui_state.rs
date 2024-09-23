use super::*;

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
                        imp.header_bar_running.add_css_class("hide-controls");
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
                        imp.header_bar_running.remove_css_class("hide-controls");
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
                        imp.header_bar_running.remove_css_class("hide-controls");
                    }
                }
            }
        ));
        obj.add_controller(click_gesture);
    }

    pub(super) fn ready(&self) {
        self.running.set(false);
        self.header_bar_running.add_css_class("hide-controls");
        self.text_view.set_running(false);
        self.text_view.set_accepts_input(true);
        self.main_stack.set_visible_child_name("session");
        self.header_stack.set_visible_child_name("ready");
        self.text_view.reset();
        self.focus_text_view();

        self.update_original_text();
        self.update_time();

        self.obj()
            .action_set_enabled("win.text-language-dialog", true);
        self.obj().action_set_enabled("win.cancel-session", false);

        self.end_existing_inhibit();
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

        self.obj()
            .action_set_enabled("win.text-language-dialog", false);
        self.obj().action_set_enabled("win.cancel-session", true);

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
}
