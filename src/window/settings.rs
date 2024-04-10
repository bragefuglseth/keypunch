use super::*;

impl imp::KpWindow {
    pub(super) fn settings(&self) -> &gio::Settings {
        self.settings.get_or_init(|| {
            gio::Settings::new("dev.bragefuglseth.Keypunch")
        })
    }

    pub(super) fn setup_settings(&self) {
        let settings = self.settings();
        let width = settings.int("window-width");
        let height = settings.int("window-height");
        let maximized = settings.boolean("window-maximized");

        let obj = self.obj();
        obj.set_default_size(width, height);

        if maximized {
            obj.maximize();
        }
    }

    pub(super) fn save_settings(&self) -> Result<(), glib::BoolError> {
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
