use crate::typing_session::TypingSession;
use crate::util::{insert_whsp_markers, pop_grapheme};
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::{gdk, graphene, gsk, pango};
use std::cell::{Cell, OnceCell, RefCell};

const LINE_HEIGHT: i32 = 45;

#[derive(Default, Clone, Copy)]
struct TextViewColorScheme {
    pub untyped: (u16, u16, u16),
    pub typed: (u16, u16, u16),
    pub mistake: (u16, u16, u16),
    pub caret: (f32, f32, f32),
}

const COLOR_SCHEME_LIGHT: TextViewColorScheme = TextViewColorScheme {
    untyped: (41472, 41472, 41472),
    typed: (12800, 12800, 12800),
    mistake: (49152, 7168, 10240),
    caret: (0.2, 0.2, 0.2),
};

const COLOR_SCHEME_DARK: TextViewColorScheme = TextViewColorScheme {
    untyped: (33792, 33792, 33792),
    typed: (65280, 65280, 65280),
    mistake: (65280, 31488, 25344),
    caret: (1., 1., 1.),
};

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
        color_scheme: Cell<TextViewColorScheme>,

        scroll_animation: OnceCell<adw::TimedAnimation>,
        caret_x_animation: OnceCell<adw::TimedAnimation>,
        caret_y_animation: OnceCell<adw::TimedAnimation>,
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

            let caret_alpha = if self.typing_session.borrow().typed_text_len() == 0 { 0. } else { 1. };

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

        fn scroll_animation(&self) -> adw::TimedAnimation {
            self.scroll_animation
                .get_or_init(|| {
                    let obj = self.obj().to_owned();

                    adw::TimedAnimation::builder()
                        .duration(300)
                        .widget(&obj)
                        .target(&adw::PropertyAnimationTarget::new(&obj, "scroll-position"))
                        .build()
                })
                .clone()
        }

        fn caret_x_animation(&self) -> adw::TimedAnimation {
            self.caret_x_animation
                .get_or_init(|| {
                    let obj = self.obj().to_owned();

                    adw::TimedAnimation::builder()
                        .duration(200)
                        .widget(&obj)
                        .target(&adw::PropertyAnimationTarget::new(&obj, "caret-x"))
                        .build()
                })
                .clone()
        }

        fn caret_y_animation(&self) -> adw::TimedAnimation {
            self.caret_y_animation
                .get_or_init(|| {
                    let obj = self.obj().to_owned();

                    adw::TimedAnimation::builder()
                        .duration(200)
                        .widget(&obj)
                        .target(&adw::PropertyAnimationTarget::new(&obj, "caret-y"))
                        .build()
                })
                .clone()
        }

        fn setup_input_handling(&self) {
            let input_method = gtk::IMMulticontext::new();

            input_method.connect_commit(glib::clone!(@weak self as imp => move |_, text| {
                imp.typing_session.borrow().typed_text().borrow_mut().push_str(text);
                imp.typed_text_changed();
            }));

            let event_controller = gtk::EventControllerKey::new();
            event_controller.set_im_context(Some(&input_method));

            let obj = self.obj();
            event_controller.connect_key_pressed(glib::clone!(@strong obj => move |controller, key, _, _| {
                match key {
                    gdk::Key::BackSpace => {
                        pop_grapheme(&mut obj.imp().typing_session.borrow().typed_text().borrow_mut());
                        obj.imp().typed_text_changed();
                        glib::signal::Propagation::Stop
                    },
                    gdk::Key::Return => {
                        controller.im_context().expect("input controller has im context").emit_by_name_with_values("commit", &["\n".into()]);
                        glib::signal::Propagation::Stop
                    }
                    _ => glib::signal::Propagation::Proceed
                }
            }));

            self.obj().add_controller(event_controller);
        }

        fn setup_color_scheme(&self) {
            let obj = self.obj();
            let style = adw::StyleManager::default();
            style.connect_dark_notify(glib::clone!(@weak obj => move |_| {
                obj.imp().update_color_scheme();
            }));

            self.update_color_scheme();
        }

        fn update_color_scheme(&self) {
            let style = adw::StyleManager::default();

            self.color_scheme.set(if style.is_dark() {
                COLOR_SCHEME_DARK
            } else {
                COLOR_SCHEME_LIGHT
            });

            self.update_visuals();
        }

        pub(super) fn set_typing_session(&self, session: TypingSession) {
            let display_text = insert_whsp_markers(&session.original_text);

            self.label.set_label(&display_text);
            self.typing_session.replace(session);
            self.typed_text_changed();
        }

        fn typed_text_changed(&self) {
            self.update_visuals();
            self.update_scroll_position();
            self.update_caret_position();
        }

        fn update_visuals(&self) {
            let clr = self.color_scheme.get();

            let comparison = self.typing_session.borrow().validate_with_whsp_markers();

            let attr_list = pango::AttrList::new();

            let untyped_attr =
                pango::AttrColor::new_foreground(clr.untyped.0, clr.untyped.1, clr.untyped.2);
            attr_list.insert(untyped_attr);

            let mut typed_attr =
                pango::AttrColor::new_foreground(clr.typed.0, clr.typed.1, clr.typed.2);
            typed_attr.set_start_index(0);
            typed_attr.set_end_index(comparison.len() as u32);
            attr_list.insert(typed_attr);

            comparison
                .iter()
                .enumerate()
                .filter(|(_, &correctly_typed)| !correctly_typed)
                .map(|(n, _)| {
                    let mut mistake_fg_attr = pango::AttrColor::new_foreground(
                        clr.mistake.0,
                        clr.mistake.1,
                        clr.mistake.2,
                    );
                    mistake_fg_attr.set_start_index(n as u32);
                    mistake_fg_attr.set_end_index(n as u32 + 1);

                    mistake_fg_attr
                })
                .for_each(|attr| attr_list.insert(attr));

            self.label.set_attributes(Some(&attr_list));
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

        fn update_caret_position(&self) {
            let session = self.typing_session.borrow();
            let current_index = session.validate_with_whsp_markers().len();

            let layout = self.label.get().layout();
            let layout_width = layout.width() / pango::SCALE;

            let (line_index, ltr_x) = layout.index_to_line_x(current_index as i32, false);
            let ltr_x = ltr_x / pango::SCALE;

            let line_width = layout.line(line_index).expect("line exists at index").extents().1.width() / pango::SCALE;

            let line_direction = layout.line(line_index).expect("line exists at index").resolved_direction();

            let x = if line_direction == pango::Direction::Rtl {
                layout_width - line_width + ltr_x
            } else {
                ltr_x
            };

            let x = if x == 0 { 1 } else if x == layout_width { layout_width - 1} else { x };

            let reference_line = if line_index == 0 { 0 } else { 1 };
            let line_start_index = layout
                .line(reference_line)
                .map(|l| l.start_index())
                .unwrap_or(0);
            let y = layout.index_to_pos(line_start_index).y() / pango::SCALE;

            let obj = self.obj();

            let caret_x_animation = self.caret_x_animation();
            caret_x_animation.set_value_from(obj.caret_x());
            caret_x_animation.set_value_to(x as f64);
            caret_x_animation.play();

            let caret_y_animation = self.caret_y_animation();
            caret_y_animation.set_value_from(obj.caret_y());
            caret_y_animation.set_value_to(y as f64);
            caret_y_animation.play();
        }

        fn animate_to_line(&self, line: i32) {
            let scroll_animation = self.scroll_animation();

            let current_position = self.obj().scroll_position();

            scroll_animation.set_value_from(current_position);
            scroll_animation.set_value_to((line * LINE_HEIGHT) as f64);

            scroll_animation.play();
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
