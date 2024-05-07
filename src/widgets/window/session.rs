use super::*;
use crate::text_generation;
use glib::ControlFlow;
use text_generation::CHUNK_GRAPHEME_COUNT;
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

            let original_grapheme_count = original_text.graphemes(true).count();
            let typed_grapheme_count = typed_text.graphemes(true).count();

            if typed_grapheme_count >= original_grapheme_count {
                imp.finish();
            }

            if typed_grapheme_count > original_grapheme_count.checked_sub(CHUNK_GRAPHEME_COUNT / 2).unwrap_or(CHUNK_GRAPHEME_COUNT) {
                let session_type = imp.session_type.get();

                let new_chunk = match session_type {
                    SessionType::Simple => text_generation::simple("en_US").unwrap(),
                    SessionType::Advanced => text_generation::advanced("en_US").unwrap(),
                    SessionType::Custom => String::new(),
                };

                tw.push_original_text(&new_chunk);
            }

            match imp.session_type.get() {
                SessionType::Simple | SessionType::Advanced => (),
                SessionType::Custom => {
                    let current_word = original_text
                        .graphemes(true)
                        .take(typed_grapheme_count)
                        .collect::<String>()
                        .unicode_words()
                        .count();

                    let total_words = original_text.unicode_words().count();

                    imp.running_title.set_title(&format!("{current_word} ⁄ {total_words}"));
                }
            }
        }));
    }

    pub(super) fn start_timer(&self) {
        let duration = match self.duration.get() {
            SessionDuration::Sec15 => Duration::from_secs(15),
            SessionDuration::Sec30 => Duration::from_secs(30),
            SessionDuration::Min1 => Duration::from_secs(60),
            SessionDuration::Min5 => Duration::from_secs(5 * 60),
            SessionDuration::Min10 => Duration::from_secs(10 * 60),
        };

        self.update_timer(duration.as_secs() + 1);

        glib::timeout_add_local(
            Duration::from_millis(100),
            glib::clone!(@weak self as imp, @strong duration => @default-return ControlFlow::Break, move || {
                let start_time = imp.start_time.get().expect("start time is set when session is running");

                if !imp.running.get() { return ControlFlow::Break; };

                if let Some(diff) = duration.checked_sub(start_time.elapsed()) {
                    let seconds = diff.as_secs() + 1;

                    // add trailing zero for second values below 10
                    imp.update_timer(seconds);
                    ControlFlow::Continue
                } else {
                    imp.finish();
                    ControlFlow::Break
                }
            }),
        );
    }

    fn update_timer(&self, seconds: u64) {
        // add trailing zero for second values below 10
        let text = if seconds >= 60 && seconds % 60 < 10 {
            format!("{}∶0{}", seconds / 60, seconds % 60) // trailing zero
        } else if seconds >= 60 {
            format!("{}∶{}", seconds / 60, seconds % 60)
        } else {
            seconds.to_string()
        };

        self.running_title.set_title(&text);
    }
}
