mod caret;
mod input;
mod scrolling;
mod styling;

use crate::text_view::styling::TextViewColorScheme;
use crate::typing_session::RcwTypingSession;
use crate::util::insert_whsp_markers;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::{gdk, graphene, gsk, pango};
use std::cell::{Cell, OnceCell, RefCell};

const LINE_HEIGHT: i32 = 45;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/dev/bragefuglseth/Raceway/text_view.ui")]
    #[properties(wrapper_type = super::RcwTextView)]
    pub struct RcwTextView {
        #[template_child]
        pub(super) label: TemplateChild<gtk::Label>,

        #[property(get, set=Self::set_scroll_position)]
        pub(super) scroll_position: Cell<f64>,
        #[property(get, set=Self::set_caret_x)]
        pub(super) caret_x: Cell<f64>,
        #[property(get, set=Self::set_caret_y)]
        pub(super) caret_y: Cell<f64>,
        #[property(get, set=Self::set_typing_session)]
        pub(super) typing_session: RefCell<RcwTypingSession>,

        pub(super) color_scheme: Cell<TextViewColorScheme>,
        pub(super) line: Cell<i32>,
        pub(super) input_context: RefCell<Option<gtk::IMMulticontext>>,

        pub(super) scroll_animation: OnceCell<adw::TimedAnimation>,
        pub(super) caret_x_animation: OnceCell<adw::TimedAnimation>,
        pub(super) caret_y_animation: OnceCell<adw::TimedAnimation>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RcwTextView {
        const NAME: &'static str = "RcwTextView";
        type Type = super::RcwTextView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("RcwTextView");

            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RcwTextView {
        fn constructed(&self) {
            self.parent_constructed();

            self.setup_input_handling();
            self.setup_color_scheme();
        }

        fn dispose(&self) {
            let obj = self.obj();

            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }

    impl WidgetImpl for RcwTextView {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Vertical => (LINE_HEIGHT * 3, LINE_HEIGHT * 3, -1, -1),
                gtk::Orientation::Horizontal => self.label.measure(orientation, for_size),

                _ => self.label.measure(orientation, for_size), // because we have to…
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let transform = gsk::Transform::new().translate(&graphene::Point::new(
                0.,
                self.obj().scroll_position() as f32 * -1.,
            ));

            self.label
                .allocate(width, height, baseline, Some(transform));

            self.update_scroll_position();
            self.update_caret_position();
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let obj = self.obj();

            obj.snapshot_child(&self.label.get(), snapshot);

            // Draw caret
            let (caret_path, caret_stroke, caret_color) = self.caret_stroke_data();
            snapshot.append_stroke(&caret_path, &caret_stroke, &caret_color);
        }
    }

    impl RcwTextView {
        pub(super) fn set_typing_session(&self, session: RcwTypingSession) {
            self.typing_session.replace(session);

            let obj = self.obj();

            obj.typing_session()
                .bind_property("original-text", &self.label.get(), "label")
                .transform_to(|_, text| Some(insert_whsp_markers(text)))
                .sync_create()
                .build();

            self.update_text_styling();
            self.update_scroll_position();
        }
    }
}

glib::wrapper! {
    pub struct RcwTextView(ObjectSubclass<imp::RcwTextView>)
        @extends gtk::Widget;
}
