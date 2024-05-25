use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use glib::subclass::Signal;
use gtk::glib;
use std::cell::{Cell, RefCell};
use std::sync::OnceLock;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "src/widgets/custom_text_dialog.blp")]
    pub struct KpCustomTextDialog {
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub placeholder: TemplateChild<gtk::Label>,
        #[template_child]
        pub text_view: TemplateChild<gtk::TextView>,
        #[template_child]
        pub save_button: TemplateChild<gtk::Button>,

        pub current_text: RefCell<String>,

        pub apply_changes: Cell<bool>,
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
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("save")
                        .param_types([str::static_type()])
                        .build(),
                    Signal::builder("discard")
                        .param_types([str::static_type()])
                        .build(),
                ]
            })
        }

        fn constructed(&self) {
            self.parent_constructed();

            let header_bar = self.header_bar.get();
            self.scrolled_window
                .vadjustment()
                .bind_property("value", &header_bar, "show-title")
                .transform_to(|_, scroll_position: f64| Some(scroll_position > 0.))
                .sync_create()
                .build();

            self.text_view
                .buffer()
                .bind_property("text", &self.placeholder.get(), "visible")
                .transform_to(|_, text: String| Some(text.is_empty()))
                .sync_create()
                .build();

            let save_button = self.save_button.get();
            self.text_view
                .buffer()
                .bind_property("text", &save_button, "sensitive")
                .transform_to(|_, text: String| {
                    let has_content = text.chars().any(|c| !c.is_whitespace());
                    Some(has_content)
                })
                .sync_create()
                .build();

            save_button.connect_clicked(glib::clone!(@weak self as imp => move |_| {
                imp.apply_changes.set(true);
                imp.obj().emit_by_name_with_values("save", &[imp.text().into()]);
                imp.obj().close();
            }));
        }
    }
    impl WidgetImpl for KpCustomTextDialog {}
    impl AdwDialogImpl for KpCustomTextDialog {
        fn closed(&self) {
            if self.changed() && !self.apply_changes.get() {
                self.obj()
                    .emit_by_name_with_values("discard", &[self.text().into()]);
            }
        }
    }

    impl KpCustomTextDialog {
        fn changed(&self) -> bool {
            self.current_text.borrow().as_str() != self.text()
        }

        fn text(&self) -> String {
            let buf = self.text_view.buffer();
            buf.text(&buf.start_iter(), &buf.end_iter(), false)
                .to_string()
        }
    }
}

glib::wrapper! {
    pub struct KpCustomTextDialog(ObjectSubclass<imp::KpCustomTextDialog>)
        @extends gtk::Widget, adw::Dialog;
}

impl KpCustomTextDialog {
    pub fn new(current_text: &str, initial_text: &str) -> Self {
        let obj = glib::Object::new::<Self>();
        let imp = obj.imp();
        *imp.current_text.borrow_mut() = current_text.to_string();
        imp.text_view.buffer().set_text(initial_text);
        imp.text_view
            .emit_by_name_with_values("select-all", &[true.into()]);
        obj
    }
}
