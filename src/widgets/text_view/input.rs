use super::*;
use crate::text_utils::{pop_word, pop_grapheme};

impl imp::KpTextView {
    pub(super) fn setup_input_handling(&self) {
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

        input_context.connect_retrieve_surrounding(glib::clone!(@weak obj => @default-return false, move |ctx| {
            let current_typed = obj.typed_text();
            let typed_len = current_typed.len() as i32;
            ctx.set_surrounding_with_selection(&current_typed, typed_len, typed_len);
            true
        }));

        input_context.connect_delete_surrounding(glib::clone!(@weak obj => @default-return false, move |_, offset, _| {
            let mut current_typed = obj.typed_text();

            // The cursor will always be at the end of the typed text,
            // so we can safely just pop the {offset} last characters
            for _ in 0..offset.abs() {
                current_typed = pop_grapheme(&current_typed);
            }

            obj.set_typed_text(current_typed);
            true
        }));

        obj.connect_has_focus_notify(glib::clone!(@weak input_context =>  move |obj| {
            if obj.has_focus() {
                input_context.focus_in();
            } else {
                input_context.focus_out();
            }
        }));

        input_context.set_input_hints(gtk::InputHints::NO_SPELLCHECK);

        let click_gesture = gtk::GestureClick::new();
        click_gesture.connect_released(glib::clone!(@weak input_context => move |controller, _, _, _| {
            input_context.activate_osk(controller.current_event());
        }));
        self.obj().add_controller(click_gesture);

        let event_controller = gtk::EventControllerKey::new();
        event_controller.set_im_context(Some(&input_context));

        event_controller.connect_key_pressed(glib::clone!(@strong obj => move |controller, key, _, modifier| {
                match (obj.accepts_input(), key) {
                   (true, gdk::Key::BackSpace) if modifier.contains(gdk::ModifierType::CONTROL_MASK) => {
                        let original = obj.original_text();
                        let current_typed = obj.typed_text();
                        obj.set_typed_text(pop_word(&original, &current_typed));
                        glib::signal::Propagation::Stop
                    },
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
