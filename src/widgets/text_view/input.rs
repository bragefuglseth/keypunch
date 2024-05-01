use super::*;
use crate::util::pop_grapheme;

impl imp::KpTextView {
    pub(super) fn setup_input_handling(&self) {
        let buffer = self.text_view.buffer();
        buffer.connect_has_selection_notify(glib::clone!(@weak self as imp => move |buffer| {
            buffer.select_range(&buffer.start_iter(), &buffer.start_iter());
            imp.update_scroll_position();
        }));

        let obj = self.obj();
        let input_context = gtk::IMMulticontext::new();

        input_context.set_client_widget(Some(&*obj.upcast_ref::<gtk::Widget>()));

        input_context.connect_commit(glib::clone!(@weak self as imp => move |_, text| {
            let obj = imp.obj();

            if obj.accepts_input() {
                if !obj.running() {
                    obj.set_running(true);
                }

                imp.push_to_typed_text(text);
            }
        }));

        obj.connect_has_focus_notify(glib::clone!(@weak input_context =>  move |obj| {
            if obj.has_focus() {
                input_context.focus_in();
            } else {
                input_context.focus_out();
            }
        }));

        let event_controller = gtk::EventControllerKey::new();
        event_controller.set_im_context(Some(&input_context));

        event_controller.connect_key_pressed(glib::clone!(@strong obj => move |controller, key, _, _| {
                match (obj.accepts_input(), key) {
                    (true, gdk::Key::BackSpace) => {
                        let current_typed = obj.typed_text();
                        obj.set_typed_text(pop_grapheme(&current_typed));
                        glib::signal::Propagation::Stop
                    },
                    (true, gdk::Key::Return) => {
                        controller.im_context().expect("input controller has im context").emit_by_name_with_values("commit", &["\n".into()]);
                        glib::signal::Propagation::Stop
                    }
                    _ => glib::signal::Propagation::Proceed
                }
            }));

        self.obj().add_controller(event_controller);
        self.input_context.replace(Some(input_context));
    }

    fn push_to_typed_text(&self, s: &str) {
        self.typed_text.borrow_mut().push_str(s);
        self.obj().notify_typed_text();
    }
}
