use crate::util::WHSP_MARKERS;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::{Cell, RefCell};
use std::time::{Instant, Duration};
use unicode_segmentation::UnicodeSegmentation;
use glib::{subclass::Signal, ControlFlow};
use std::sync::OnceLock;

#[derive(Default, PartialEq, Clone, Copy)]
pub enum SessionType {
    #[default]
    LengthBased,
    TimeBased(Duration),
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
enum SessionState {
    #[default]
    Ready,
    Running,
    Finished(Instant),
}

mod imp {
    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type=super::RcwTypingSession)]
    pub struct RcwTypingSession {
        pub(super) session_type: Cell<SessionType>,
        pub(super) state: Cell<SessionState>,
        pub(super) start_time: Cell<Option<Instant>>,
        #[property(get, set)]
        pub(super) original_text: RefCell<String>,
        #[property(get)]
        pub(super) typed_text: RefCell<String>,
        #[property(get, set)]
        pub(super) progress_text: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RcwTypingSession {
        const NAME: &'static str = "RcwTypingSession";
        type Type = super::RcwTypingSession;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for RcwTypingSession {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![Signal::builder("started").build(), Signal::builder("finished").build(), Signal::builder("stopped").build()]
            })
        }
    }
}

glib::wrapper! {
    pub struct RcwTypingSession(ObjectSubclass<imp::RcwTypingSession>);
}

impl Default for RcwTypingSession {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl RcwTypingSession {
    pub fn set_type(&self, session_type: SessionType) {
        self.imp().session_type.set(session_type);
    }

    fn start(&self) {
        let imp = self.imp();
        let state = &imp.state;

        if state.get() != SessionState::Ready { return; }

        imp.start_time.set(Some(Instant::now()));
        state.set(SessionState::Running);
        self.emit_by_name::<()>("started", &[]);

        if let SessionType::TimeBased(duration) = imp.session_type.get() {
            glib::timeout_add_local(
                Duration::from_millis(100),
                glib::clone!(@weak self as session, @strong duration => @default-return ControlFlow::Break, move || {
                    let imp = session.imp();
                    let start_time = imp.start_time.get().expect("start time is set when session is running");

                    if imp.state.get() != SessionState::Running { return ControlFlow::Break; };

                    if let Some(diff) = duration.checked_sub(start_time.elapsed()) {
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

                        session.set_progress_text(text);
                        ControlFlow::Continue
                    } else {
                        session.finish();
                        ControlFlow::Break
                    }
                }),
            );
        }
    }

    fn update_word_count(&self) {
        let imp = self.imp();

        if imp.session_type.get() != SessionType::LengthBased { return; };

        let total_words = imp.original_text.borrow().unicode_words().count();

        let typed_words_len = imp.typed_text.borrow().graphemes(true).count();
        let current_word = imp.original_text.borrow()
            .unicode_word_indices()
            .filter(|&(i,_)| i <= typed_words_len)
            .count();

        self.set_progress_text(format!("{current_word} ⁄ {total_words}"));
    }

    pub fn stop(&self) {
        let imp = self.imp();
        let state = &imp.state;

        if state.get() != SessionState::Running { return; }

        state.set(SessionState::Ready);
        *imp.typed_text.borrow_mut() = String::new();
        self.emit_by_name::<()>("stopped", &[]);
    }

    fn finish(&self) {
        let state = &self.imp().state;

        if state.get() != SessionState::Running { return; }

        state.set(SessionState::Finished(Instant::now()));
        self.emit_by_name::<()>("finished", &[]);
    }

    pub fn push_to_typed_text(&self, s: &str) {
        let imp = self.imp();

        match (imp.state.get(), self.text_left()) {
            (SessionState::Ready, true) => {
                self.start();
                self.imp().typed_text.borrow_mut().push_str(s);
                self.update_word_count();
            }
            (SessionState::Running, true) => {
                self.imp().typed_text.borrow_mut().push_str(s);
                self.update_word_count();
                if !self.text_left() {
                    self.finish();
                }
            }
            (SessionState::Running, false) => {
                self.finish();
            }
            _ => (),
        }
    }

    fn text_left(&self) -> bool {
        self.imp().typed_text.borrow().graphemes(true).count() < self.original_text().graphemes(true).count()
    }

    pub fn pop_typed_text(&self) {
        let imp = self.imp();
        let state = imp.state.get();

        if state == SessionState::Running {
            {
                let mut typed_text = imp.typed_text.borrow_mut();
                let mut v = typed_text.graphemes(true).collect::<Vec<_>>();
                v.pop();
                *typed_text = v.into_iter().collect();
            }

            self.update_word_count();
        }
    }

    pub fn validate_with_whsp_markers(&self) -> Vec<bool> {
        let original = &self.original_text();
        let typed = self.imp().typed_text.borrow();

        original
            .graphemes(true)
            .zip(typed.graphemes(true))
            .map(|(og, tg)| {
                let matches = og == tg;
                // check if the typed grapheme exists in the whitespace char database
                // used by the text view. if that's the case, match the length of the
                // indicator used.
                if let Some((_, val)) = WHSP_MARKERS.iter().find(|(key, _)| *key == og) {
                    vec![matches; val.len()]
                } else {
                    vec![matches; og.len()]
                }
            })
            .flatten()
            .collect()
    }
}
