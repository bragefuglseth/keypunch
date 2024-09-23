use super::*;

impl imp::KpWindow {
    pub(super) fn inhibit_session(&self, reason: &str) {
        self.end_existing_inhibit();

        let obj = self.obj();
        let app = obj
            .application()
            .expect("the window always has an associated app");

        let cookie = app.inhibit(
            Some(self.obj().upcast_ref::<gtk::Window>()),
            gtk::ApplicationInhibitFlags::LOGOUT,
            Some(&reason),
        );

        self.inhibit_cookie.set(Some(cookie));
    }

    pub(super) fn end_existing_inhibit(&self) {
        if let Some(cookie) = self.inhibit_cookie.take() {
            self.obj()
                .application()
                .expect("the window always has an associated app")
                .uninhibit(cookie);
        }
    }
}
