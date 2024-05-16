use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type=super::KpResultsView)]
    #[template(file = "src/widgets/results_view.blp")]
    pub struct KpResultsView {
        #[template_child]
        wpm_accuracy_box: TemplateChild<gtk::Box>,
        #[template_child]
        wpm_label: TemplateChild<gtk::Label>,
        #[template_child]
        accuracy_label: TemplateChild<gtk::Label>,
        #[template_child]
        session_info_box: TemplateChild<gtk::Box>,
        #[template_child]
        session_type_label: TemplateChild<gtk::Label>,
        #[template_child]
        duration_label: TemplateChild<gtk::Label>,
        #[template_child]
        language_box: TemplateChild<gtk::Box>,
        #[template_child]
        language_label: TemplateChild<gtk::Label>,

        #[property(get, set)]
        wpm: Cell<f64>,
        #[property(get, set)]
        accuracy: Cell<f64>,
        #[property(get, set)]
        session_type: RefCell<String>,
        #[property(get, set)]
        duration: Cell<u64>,
        #[property(get, set)]
        language: RefCell<String>,
        #[property(get, set)]
        show_language: Cell<bool>,
        #[property(get, set, builder(gtk::Orientation::Vertical))]
        orientation: RefCell<gtk::Orientation>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpResultsView {
        const NAME: &'static str = "KpResultsView";
        type Type = super::KpResultsView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("KpResultsView");

            klass.set_layout_manager_type::<gtk::BinLayout>();

            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            KpResultsView {
                wpm_accuracy_box: Default::default(),
                wpm_label: Default::default(),
                accuracy_label: Default::default(),
                session_info_box: Default::default(),
                session_type_label: Default::default(),
                duration_label: Default::default(),
                language_box: Default::default(),
                language_label: Default::default(),

                wpm: Default::default(),
                accuracy: Default::default(),
                session_type: Default::default(),
                duration: Default::default(),
                language: Default::default(),
                show_language: Default::default(),
                orientation: RefCell::new(gtk::Orientation::Horizontal),
            }
        }
    }

    impl ObjectImpl for KpResultsView {
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

            let wpm_accuracy_box = self.wpm_accuracy_box.get();
            let wpm_label = self.wpm_label.get();
            let accuracy_label = self.accuracy_label.get();
            let session_type_label = self.session_type_label.get();
            let duration_label = self.duration_label.get();
            let language_box = self.language_box.get();
            let language_label = self.language_label.get();
            let session_info_box = self.session_info_box.get();

            let obj = self.obj();
            obj.bind_property("wpm", &wpm_label, "label")
                .transform_to(|_, wpm: f64| {
                    let formatted = format!("{:.0}", wpm.floor());
                    Some(formatted)
                })
                .build();

            obj.bind_property("accuracy", &accuracy_label, "label")
                .transform_to(|_, accuracy: f64| {
                    let display_accuracy = (accuracy * 100.).floor();
                    let formatted = format!("{:.0}%", display_accuracy);
                    Some(formatted)
                })
                .build();

            obj.bind_property("session-type", &session_type_label, "label")
                .build();

            obj.bind_property("duration", &duration_label, "label")
                .transform_to(|_, duration: u64| {
                    let formatted = human_readable_duration(duration);
                    Some(formatted)
                })
                .build();

            obj.bind_property("show-language", &language_box, "visible")
                .build();

            obj.bind_property("language", &language_label, "label")
                .build();

            obj.bind_property("orientation", &wpm_accuracy_box, "orientation")
                .build();

            obj.bind_property("orientation", &session_info_box, "orientation")
                .build();

            obj.bind_property("orientation", &session_info_box, "spacing")
                .transform_to(|_, orientation| match orientation {
                    gtk::Orientation::Horizontal => Some(36),
                    gtk::Orientation::Vertical => Some(18),
                    _ => None,
                })
                .build();
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }
    impl WidgetImpl for KpResultsView {}
    impl OrientableImpl for KpResultsView {}
}

glib::wrapper! {
    pub struct KpResultsView(ObjectSubclass<imp::KpResultsView>)
        @extends gtk::Widget, @implements gtk::Orientable;
}

pub fn human_readable_duration(total_secs: u64) -> String {
    let minutes = total_secs / 60;
    let secs = total_secs % 60;

    if minutes > 0 && secs > 0 {
        format!("{minutes}m {secs}s")
    } else if minutes == 1 {
        format!("{minutes} minute")
    } else if minutes > 0 {
        format!("{minutes} minutes")
    } else if secs == 1 {
        format!("{secs} second")
    } else {
        format!("{secs} seconds")
    }
}
