use super::*;

const UNFOCUSED_TIMEOUT_MILLIS: u64 = 2000;

impl imp::KpWindow {
    pub(super) fn setup_focus(&self) {
        self.focus_button
            .connect_clicked(glib::clone!(@weak self as imp => move |_| {
                imp.focus_text_view();
            }));

        self.obj().connect_focus_widget_notify(
            glib::clone!(@weak self as imp => move |_| {
                let text_view = imp.text_view.get();
                let bottom_stack_empty = imp.bottom_stack_empty.get();
                let just_start_typing = imp.just_start_typing.get();
                let focus_button = imp.focus_button.get();
                let bottom_stack = imp.bottom_stack.get();

                match (imp.text_view_focused(), imp.running.get()) {
                    (true, true) => {
                        bottom_stack.set_visible_child(&bottom_stack_empty);
                        text_view.remove_css_class("unfocused");
                    }
                    (true, false) => {
                        bottom_stack.set_visible_child(&just_start_typing);
                        text_view.remove_css_class("unfocused");
                    }
                    (false, _) => {
                        let timeout = glib::timeout_add_local_once(
                            Duration::from_millis(UNFOCUSED_TIMEOUT_MILLIS),
                            glib::clone!(@weak bottom_stack,
                                @weak focus_button,
                                @weak imp
                                => move || {
                                    if !imp.text_view_focused() && !imp.open_dialog.get() {
                                        bottom_stack.set_visible_child(&focus_button);
                                        text_view.add_css_class("unfocused");
                                    }
                            }
                        ));

                        let Some(previous_event) = imp.last_unfocus_event
                            .replace(Some(timeout)) else { return; };

                        let Some(previous_timestamp) = imp.last_unfocus_timestamp
                            .replace(Some(Instant::now())) else { return; };

                        if (Instant::now() - previous_timestamp).as_millis() < UNFOCUSED_TIMEOUT_MILLIS.into() {
                            previous_event.remove();
                        }
                    }
                };
            }),
        );
    }

    pub(super) fn text_view_focused(&self) -> bool {
        if let Some(focus) = self.obj().focus_widget() {
            focus == self.text_view.get()
        } else {
            false
        }
    }

    pub(super) fn focus_text_view(&self) {
        self.obj().set_focus_widget(Some(&self.text_view.get()));
    }
}
