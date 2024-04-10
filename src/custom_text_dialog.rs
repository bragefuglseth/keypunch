use adw::subclass::prelude::*;
use gtk::glib;
use adw::prelude::*;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/bragefuglseth/Keypunch/custom_text_dialog.ui")]
    pub struct KpCustomTextDialog {
        #[template_child]
        header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        text_view: TemplateChild<gtk::TextView>,
        #[template_child]
        save_button: TemplateChild<gtk::Button>,
    }

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

    impl ObjectImpl for KpCustomTextDialog {
        fn constructed(&self) {
            self.parent_constructed();

            let header_bar = self.header_bar.get();
            self.scrolled_window.vadjustment().bind_property("value", &header_bar, "show-title")
                .transform_to(|_, scroll_position: f64| {
                    Some(scroll_position > 0.)
                })
                .sync_create()
                .build();

            let placeholder = gtk::Label::builder()
                .label("Insert custom text…")
                .css_classes(["dim-label"])
                .build();
            self.text_view.add_overlay(&placeholder, 0, 0);
            self.text_view.buffer().bind_property("text", &placeholder, "visible")
                .transform_to(|_, text: String| {
                    Some(text.is_empty())
                })
                .sync_create()
                .build();

            let save_button = self.save_button.get();
            self.text_view.buffer().bind_property("text", &save_button, "sensitive")
                .transform_to(|_, text: String| {
                    let has_content = text.chars().any(|c| !c.is_whitespace());
                    Some(has_content)
                })
                .sync_create()
                .build();
        }
    }
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
