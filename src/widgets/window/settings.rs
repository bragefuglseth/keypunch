use super::*;

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
        let session_type = settings.enum_("session-type");
        let duration = settings.enum_("session-duration");
        let custom_text = settings.string("custom-text");

        let obj = self.obj();
        obj.set_default_size(width, height);

        self.session_type.set(SessionType::from_i32(session_type).expect("settings contain valid SessionType value"));

        self.duration.set(SessionDuration::from_i32(duration).expect("settings contain valid SessionDuration value"));

        if maximized {
            obj.maximize();
        }

        *self.custom_text.borrow_mut() = custom_text.into();
    }

    pub(super) fn save_settings(&self) -> Result<(), glib::BoolError> {
        let obj = self.obj();
        let width = obj.default_width();
        let height = obj.default_height();
        let maximized = obj.is_maximized();
        let session_type = self.session_type.get();
        let duration = self.duration.get();
        let custom_text = self.custom_text.borrow();

        let settings = self.settings();
        settings.set_int("window-width", width)?;
        settings.set_int("window-height", height)?;
        settings.set_boolean("window-maximized", maximized)?;
        settings.set_enum("session-type", session_type as i32)?;
        settings.set_enum("session-duration", duration as i32)?;
        settings.set_string("custom-text", &custom_text)?;
        Ok(())
    }
}
