use super::*;

const UNFOCUSED_TIMEOUT_MILLIS: u64 = 1500;

impl imp::KpWindow {
    pub(super) fn setup_focus(&self) {
        self.focus_button
            .connect_clicked(glib::clone!(@weak self as imp => move |_| {
                imp.focus_text_view();
            }));

        self.text_view.connect_has_focus_notify(
            glib::clone!(@weak self as imp => move |text_view| {
                let bottom_stack_empty = imp.bottom_stack_empty.get();
                let just_start_typing = imp.just_start_typing.get();
                let focus_button = imp.focus_button.get();
                let bottom_stack = imp.bottom_stack.get();

                match (text_view.has_focus(), imp.running.get()) {
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
                            glib::clone!(@weak text_view,
                                @weak bottom_stack,
                                @weak focus_button
                                => move || {
                                    if !text_view.has_focus() {
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

    pub(super) fn focus_text_view(&self) {
        self.obj().set_focus_widget(Some(&self.text_view.get()));
    }
}
