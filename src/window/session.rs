use super::*;
use glib::ControlFlow;
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
