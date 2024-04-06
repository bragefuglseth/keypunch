use super::*;

impl imp::RcwTextView {
    pub(super) fn setup_input_handling(&self) {
        let obj = self.obj();
        let input_context = gtk::IMMulticontext::new();

        input_context.set_client_widget(Some(&*obj.upcast_ref::<gtk::Widget>()));

        input_context.connect_commit(glib::clone!(@weak self as imp => move |_, text| {
            imp.typing_session.borrow().push_to_typed_text(text);
            imp.update_text_styling();
            imp.update_scroll_position();
        }));

        let event_controller = gtk::EventControllerKey::new();
        event_controller.set_im_context(Some(&input_context));

        event_controller.connect_key_pressed(glib::clone!(@strong obj => move |controller, key, _, _| {
                match key {
                    gdk::Key::BackSpace => {
                        obj.imp().typing_session.borrow().pop_typed_text();
                        obj.imp().update_text_styling();
                        obj.imp().update_scroll_position();
                        glib::signal::Propagation::Stop
                    },
                    gdk::Key::Return => {
                        controller.im_context().expect("input controller has im context").emit_by_name_with_values("commit", &["\n".into()]);
                        glib::signal::Propagation::Stop
                    }
                    _ => glib::signal::Propagation::Proceed
                }
            }));

        self.obj().add_controller(event_controller);
        self.input_context.replace(Some(input_context));
    }
}
