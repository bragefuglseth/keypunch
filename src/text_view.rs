use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::{graphene, gsk, gdk, pango};
use std::cell::{Cell, OnceCell, RefCell};
use crate::typing_session::TypingSession;
use crate::util::pop_grapheme;

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

                _ => self.label.measure(orientation, for_size),     // because we have to…
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
            caret_path.line_to(caret_x, caret_y + (self.label.layout().baseline() / pango::SCALE) as f32 + 2.);
            let caret_path = caret_path.to_path();

            let caret_stroke = gsk::Stroke::new(2.);
            let caret_color = gdk::RGBA::new(0.5, 0.5, 0.5, 1.);

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
            self.obj().queue_allocate();
        }

        fn set_caret_y(&self, caret_y: f64) {
            self.caret_y.set(caret_y);
            self.obj().queue_allocate();
        }

        fn scroll_animation(&self) -> adw::TimedAnimation {
            self.scroll_animation
                .get_or_init(|| {
                    let obj = self.obj().to_owned();

                    adw::TimedAnimation::builder()
                        .duration(200)
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
                        .duration(100)
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
                        .duration(100)
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
                    },
                    gdk::Key::Return => {
                        controller.im_context().expect("input controller has im context").emit_by_name_with_values("commit", &["\n".into()]);
                    }
                    _ => ()
                }

                glib::signal::Propagation::Stop
            }));

            self.obj().add_controller(event_controller);
        }

        pub(super) fn set_typing_session(&self, session: TypingSession) {
            let display_text = session.original_text.clone().replace("\n", "↲\n");

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
            let comparison = self.typing_session.borrow().validate_with_whsp_markers();

            let attr_list = pango::AttrList::new();

            let untyped_attr = pango::AttrColor::new_foreground(40000, 40000, 40000);
            attr_list.insert(untyped_attr);

            let mut typed_attr = pango::AttrColor::new_foreground(10000, 10000, 10000);
            typed_attr.set_start_index(0);
            typed_attr.set_end_index(comparison.len() as u32);
            attr_list.insert(typed_attr);

            comparison.iter().enumerate()
                .filter(|(_, &correctly_typed)| !correctly_typed)
                .map(|(n, _)| {
                    let mut mistake_attr = pango::AttrColor::new_foreground(50000, 10000, 10000);
                    mistake_attr.set_start_index(n as u32);
                    mistake_attr.set_end_index(n as u32 + 1);
                    mistake_attr
                })
                .for_each(|attr| attr_list.insert(attr));

            self.label.set_attributes(Some(&attr_list));
        }

        fn update_scroll_position(&self) {
            let session = self.typing_session.borrow();

            let (line, _) = self.label.layout().index_to_line_x(session.typed_text_len_whsp_markers() as i32, false);

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
            let current_index = session.typed_text_len_whsp_markers();
            
            let layout = self.label.get().layout();
            
            let (line, x) = layout.index_to_line_x(current_index as i32, false);
            let x = if line == 0 && x == 0 { -2 } else { x / pango::SCALE };
            
            let reference_line = if line == 0 { 0 } else { 1 };
            let start_index = layout.line(reference_line).map(|l| l.start_index()).unwrap_or(0);
            let y = layout.index_to_pos(start_index).y() / pango::SCALE;
            
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
