mod animation;
mod caret;
mod color;
mod input;

use crate::text_view::color::TextViewColorScheme;
use crate::typing_session::TypingSession;
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
        scroll_position: Cell<f64>,
        #[property(get, set=Self::set_caret_x)]
        caret_x: Cell<f64>,
        #[property(get, set=Self::set_caret_y)]
        caret_y: Cell<f64>,

        line: Cell<i32>,

        pub(super) typing_session: RefCell<TypingSession>,
        pub(super) color_scheme: Cell<TextViewColorScheme>,
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

            let caret_x = obj.caret_x() as f32;
            let caret_y = obj.caret_y() as f32;

            let caret_path = gsk::PathBuilder::new();
            caret_path.move_to(caret_x, caret_y);
            caret_path.line_to(
                caret_x,
                caret_y + (self.label.layout().baseline() / pango::SCALE) as f32 + 2.,
            );

            let clr = self.color_scheme.get();

            let caret_alpha = if self.typing_session.borrow().typed_text_len() == 0 {
                0.
            } else {
                1.
            };

            let caret_color = gdk::RGBA::new(clr.caret.0, clr.caret.1, clr.caret.2, caret_alpha);
            let caret_stroke = gsk::Stroke::new(1.);
            let caret_path = caret_path.to_path();

            snapshot.append_stroke(&caret_path, &caret_stroke, &caret_color);
        }
    }

    impl RcwTextView {
        fn set_scroll_position(&self, line_number: f64) {
            self.scroll_position.set(line_number);
            self.obj().queue_allocate();
        }

        fn set_caret_x(&self, caret_x: f64) {
            self.caret_x.set(caret_x);
            self.obj().queue_draw();
        }

        fn set_caret_y(&self, caret_y: f64) {
            self.caret_y.set(caret_y);
            self.obj().queue_draw();
        }

        pub(super) fn set_typing_session(&self, session: TypingSession) {
            let display_text = insert_whsp_markers(&session.original_text());

            self.label.set_label(&display_text);
            self.typing_session.replace(session);
            self.typed_text_changed();
        }

        pub(super) fn typed_text_changed(&self) {
            self.update_visuals();
            self.update_scroll_position();
            self.update_caret_position();
        }

        fn update_scroll_position(&self) {
            let session = self.typing_session.borrow();

            let (line, _) = self
                .label
                .layout()
                .index_to_line_x(session.validate_with_whsp_markers().len() as i32, false);

            if line != self.line.get() {
                self.line.set(line);
                self.animate_to_line(match line {
                    0 | 1 => 0,
                    num => num - 1,
                });
            }
        }
    }
}

glib::wrapper! {
    pub struct RcwTextView(ObjectSubclass<imp::RcwTextView>)
        @extends gtk::Widget;
}

impl RcwTextView {
    pub fn set_typing_session(&self, session: TypingSession) {
        self.imp().set_typing_session(session);
    }
}
