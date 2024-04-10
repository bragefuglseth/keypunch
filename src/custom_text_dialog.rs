use adw::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/bragefuglseth/Keypunch/custom_text_dialog.ui")]
    pub struct KpCustomTextDialog {}

    #[glib::object_subclass]
    impl ObjectSubclass for KpCustomTextDialog {
        const NAME: &'static str = "KpCustomTextDialog";
        type Type = super::KpCustomTextDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for KpCustomTextDialog {}
    impl WidgetImpl for KpCustomTextDialog {}
    impl AdwDialogImpl for KpCustomTextDialog {}
}

glib::wrapper! {
    pub struct KpCustomTextDialog(ObjectSubclass<imp::KpCustomTextDialog>)
        @extends gtk::Widget, adw::Dialog;
}

impl KpCustomTextDialog {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
