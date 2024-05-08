mod caret;
mod colors;
mod input;
mod scrolling;

use crate::text_utils::{insert_whsp_markers, validate_with_whsp_markers};
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::{gdk, gsk};
use std::cell::{Cell, OnceCell, RefCell};

const LINE_HEIGHT: i32 = 45;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/widgets/text_view.blp")]
    #[properties(wrapper_type = super::KpTextView)]
    pub struct KpTextView {
        #[template_child]
        pub(super) text_view: TemplateChild<gtk::TextView>,

        #[property(get, set=Self::set_caret_x)]
        pub(super) caret_x: Cell<f64>,
        #[property(get, set=Self::set_caret_y)]
        pub(super) caret_y: Cell<f64>,
        #[property(get, set)]
        pub(super) caret_height: Cell<f64>,

        pub(super) original_text: RefCell<String>,
        #[property(get, set)]
        pub(super) typed_text: RefCell<String>,
        #[property(get, set)]
        pub(super) running: Cell<bool>,
        #[property(get, set)]
        pub(super) accepts_input: Cell<bool>,

        pub(super) input_context: RefCell<Option<gtk::IMMulticontext>>,
        pub(super) scroll_animation: OnceCell<adw::TimedAnimation>,
        pub(super) caret_x_animation: OnceCell<adw::TimedAnimation>,
        pub(super) caret_y_animation: OnceCell<adw::TimedAnimation>,
        pub(super) caret_rgb: Cell<(f32, f32, f32)>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpTextView {
        const NAME: &'static str = "KpTextView";
        type Type = super::KpTextView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("KpTextView");

            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for KpTextView {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let text_view = self.text_view.get();
            text_view.set_bottom_margin(LINE_HEIGHT);
            text_view.set_can_target(false);

            obj.connect_typed_text_notify(|obj| {
                let imp = obj.imp();
                imp.update_colors();
                imp.update_caret_position(false);
                imp.update_scroll_position(false);
            });

            self.setup_input_handling();
            self.setup_color_scheme();
            self.update_colors();
            self.update_scroll_position(true);
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for KpTextView {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Vertical => (LINE_HEIGHT * 3, LINE_HEIGHT * 3, -1, -1),
                gtk::Orientation::Horizontal => self.text_view.measure(orientation, for_size),
                _ => panic!("orientation type not accounted for"),
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.text_view.allocate(width, height, baseline, None);
            self.update_scroll_position(true);
            self.update_caret_position(true);
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.obj().snapshot_child(&self.text_view.get(), snapshot);

            let (caret_path, caret_stroke, caret_color) = self.caret_stroke_data();
            snapshot.append_stroke(&caret_path, &caret_stroke, &caret_color);
        }
    }

    impl KpTextView {
        pub fn set_original_text(&self, text: &str) {
            *self.original_text.borrow_mut() = text.to_string();
            self.text_view
                .buffer()
                .set_text(&insert_whsp_markers(&text));
            self.update_colors();
            self.update_caret_position(true);
            self.update_scroll_position(true);
        }

        pub fn push_original_text(&self, text: &str) {
            self.original_text.borrow_mut().push_str(text);

            let buffer = self.text_view.buffer();
            buffer.insert(&mut buffer.end_iter(), &insert_whsp_markers(&text));
            self.update_colors();
        }
    }
}

glib::wrapper! {
    pub struct KpTextView(ObjectSubclass<imp::KpTextView>)
        @extends gtk::Widget;
}

impl KpTextView {
    pub fn original_text(&self) -> String {
        self.imp().original_text.borrow().to_string()
    }

    pub fn set_original_text(&self, text: &str) {
        self.imp().set_original_text(text);
    }

    pub fn push_original_text(&self, text: &str) {
        self.imp().push_original_text(text);
    }

    pub fn reset(&self, animate: bool) {
        self.set_original_text("");
        self.set_typed_text("");
        self.set_running(false);

        if !animate {
            let imp = self.imp();
            imp.scroll_animation().skip();
            imp.caret_x_animation().skip();
            imp.caret_y_animation().skip();
        }
    }
}
