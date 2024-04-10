use super::*;
use glib::ControlFlow;
use gtk::gdk;
use unicode_segmentation::UnicodeSegmentation;

impl imp::KpWindow {
    pub(super) fn setup_text_view(&self) {
        let text_view = self.text_view.get();

        text_view.connect_running_notify(glib::clone!(@weak self as imp => move |tw| {
            if tw.running() {
                imp.start();
            }
        }));

        text_view.connect_typed_text_notify(glib::clone!(@weak self as imp => move |tw| {
            if !imp.running.get() { return; }

            let original_text = tw.original_text();
            let typed_text = tw.typed_text();

            match imp.text_type.get() {
                TextType::Simple | TextType::Advanced => (),
                TextType::Custom => {
                    let total_words = original_text.unicode_words().count();

                    let typed_words_len = typed_text.graphemes(true).count();
                    let current_word = original_text
                        .unicode_word_indices()
                        .filter(|&(i, _)| i <= typed_words_len)
                        .count();

                    imp.running_title.set_title(&format!("{current_word} ⁄ {total_words}"));
                }
            }

            if typed_text.len() >= original_text.len() {
                imp.finish();
            }
        }));
    }

    pub(super) fn focus_text_view(&self) {
        self.obj().set_focus_widget(Some(&self.text_view.get()));
    }

    pub(super) fn setup_stop_button(&self) {
        self.stop_button
            .connect_clicked(glib::clone!(@weak self as imp => move |_| {
                imp.ready(true);
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
                imp.header_bar_running.remove_css_class("hide-controls");

                imp.show_cursor();
            }
        }));

        self.obj().add_controller(motion_ctrl);
    }

    pub(super) fn device(&self) -> gdk::Device {
        self
            .obj()
            .display()
            .default_seat()
            .expect("display always has a default seat")
            .pointer()
            .expect("default seat has device")
    }

    pub(super) fn hide_cursor(&self) {
        self.show_cursor.set(false);
        self.cursor_hidden_timestamp.set(self.device().timestamp());
        self.obj().set_cursor_from_name(Some("none"));
    }

    pub(super) fn show_cursor(&self) {
        self.show_cursor.set(true);
        self.obj().set_cursor_from_name(Some("default"));
    }


    pub(super) fn ready(&self, animate: bool) {
        self.running.set(false);
        self.header_bar_running.add_css_class("hide-controls");
        self.text_view.set_running(false);
        self.text_view.set_accepts_input(true);
        self.main_stack.set_visible_child_name("session");
        self.header_stack.set_visible_child_name("ready");
        self.ready_message.set_reveal_child(true);
        self.text_view.reset(animate);
        self.show_cursor();
        self.focus_text_view();

        self.update_original_text();
        self.update_time();
    }

    pub(super) fn start(&self) {
        self.running.set(true);
        self.start_time.set(Some(Instant::now()));
        self.main_stack.set_visible_child_name("session");
        self.header_stack.set_visible_child_name("running");
        self.ready_message.set_reveal_child(false);
        self.hide_cursor();
        self.header_bar_running.add_css_class("hide-controls");

        match self.text_type.get() {
            TextType::Simple | TextType::Advanced => self.start_timer(),
            TextType::Custom => (),
        }
    }

    pub(super) fn finish(&self) {
        self.running.set(false);
        self.text_view.set_running(false);
        self.text_view.set_accepts_input(false);
        self.main_stack.set_visible_child_name("results");
        self.show_cursor();
    }

    pub(super) fn start_timer(&self) {
        glib::timeout_add_local(
            Duration::from_millis(100),
            glib::clone!(@weak self as imp => @default-return ControlFlow::Break, move || {
                let start_time = imp.start_time.get().expect("start time is set when session is running");

                if !imp.running.get() { return ControlFlow::Break; };

                if let Some(diff) = imp.duration.get().checked_sub(start_time.elapsed()) {
                    let seconds = diff.as_secs() + 1;

                    // add trailing zero for second values below 10
                    let text = if seconds >= 60 && seconds % 60 < 10 {
                        let with_trailing_zero = format!("0{}", seconds % 60);
                        format!("{}∶{}", seconds / 60, with_trailing_zero)
                    } else if seconds >= 60 {
                        format!("{}∶{}", seconds / 60, seconds % 60)
                    }else {
                        seconds.to_string()
                    };

                    imp.running_title.set_title(&text);
                    ControlFlow::Continue
                } else {
                    imp.finish();
                    ControlFlow::Break
                }
            }),
        );
    }
}
