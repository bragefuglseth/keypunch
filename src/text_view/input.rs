use super::*;

impl imp::RcwTextView {
    pub(super) fn setup_input_handling(&self) {
        let obj = self.obj();
        let input_context = gtk::IMMulticontext::new();

        input_context
            .set_client_widget(Some(&*obj.upcast_ref::<gtk::Widget>()));

        input_context
            .connect_commit(glib::clone!(@weak self as imp => move |_, text| {
                imp.typing_session.borrow().typed_text().borrow_mut().push_str(text);
                imp.typed_text_changed();
            }));

        let event_controller = gtk::EventControllerKey::new();
        event_controller.set_im_context(Some(
            &input_context,
        ));

        event_controller.connect_key_pressed(glib::clone!(@strong obj => move |controller, key, _, _| {
                match key {
                    gdk::Key::BackSpace => {
                        pop_grapheme(&mut obj.imp().typing_session.borrow().typed_text().borrow_mut());
                        obj.imp().typed_text_changed();
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
