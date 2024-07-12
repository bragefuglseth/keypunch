use super::*;
use std::str::FromStr;

impl imp::KpWindow {
    pub(super) fn settings(&self) -> &gio::Settings {
        self.settings
            .get_or_init(|| gio::Settings::new("dev.bragefuglseth.Keypunch"))
    }

    pub(super) fn load_settings(&self) {
        let settings = self.settings();
        let width = settings.int("window-width");
        let height = settings.int("window-height");
        let maximized = settings.boolean("window-maximized");
        let session_type = settings.string("session-type");
        let duration = settings.string("session-duration");
        let language = settings.string("text-language");
        let recent_languages = settings.value("recent-languages");
        let custom_text = settings.string("custom-text");

        let obj = self.obj();
        obj.set_default_size(width, height);

        self.session_type
            .set(SessionType::from_str(&session_type).unwrap_or(SessionType::Simple));

        self.duration
            .set(SessionDuration::from_str(&duration).unwrap_or(SessionDuration::Sec30));

        self.language
            .set(Language::from_str(language.as_str()).unwrap_or(Language::English));

        let recent_languages_vec: Vec<Language> = recent_languages
            .get::<Vec<String>>()
            .expect("recent languages is a list of type `String`")
            .iter()
            .filter_map(|s| Language::from_str(&s).ok())
            .collect();
        self.recent_languages
            .borrow_mut()
            .extend(&recent_languages_vec);

        if maximized {
            obj.maximize();
        }

        *self.custom_text.borrow_mut() = custom_text.into();
    }

    pub(super) fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let obj = self.obj();
        let width = obj.default_width();
        let height = obj.default_height();
        let maximized = obj.is_maximized();

        let settings = self.settings();
        settings.set_int("window-width", width)?;
        settings.set_int("window-height", height)?;
        settings.set_boolean("window-maximized", maximized)?;

        Ok(())
    }
}
