/* interactive_graph.rs
 *
 * SPDX-FileCopyrightText: © 2025 Brage Fuglseth <bragefuglseth@gnome.org>
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::widgets::line_chart::{CHART_HEIGHT, Y_BOUND_GROW_STEPS};
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gdk, glib, gsk};
use layout::KpInteractiveGraphLayout;
use layout_child::KpInteractiveGraphLayoutChild;
use std::cell::{Cell, RefCell};

mod layout_child {
    use super::*;

    mod imp {
        use super::*;

        #[derive(Debug, Default, glib::Properties)]
        #[properties(wrapper_type = super::KpInteractiveGraphLayoutChild)]
        pub struct KpInteractiveGraphLayoutChild {
            #[property(get, set=Self::set_x_origin)]
            pub x_origin: Cell<i32>,
            #[property(get, set=Self::set_y_origin)]
            pub y_origin: Cell<i32>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for KpInteractiveGraphLayoutChild {
            const NAME: &'static str = "KpInteractiveGraphLayoutChild";
            type Type = super::KpInteractiveGraphLayoutChild;
            type ParentType = gtk::LayoutChild;
        }

        #[glib::derived_properties]
        impl ObjectImpl for KpInteractiveGraphLayoutChild {}

        impl LayoutChildImpl for KpInteractiveGraphLayoutChild {}

        impl KpInteractiveGraphLayoutChild {
            pub fn set_x_origin(&self, x_origin: i32) {
                self.x_origin.set(x_origin);

                self.obj().layout_manager().layout_changed();
            }

            pub fn set_y_origin(&self, y_origin: i32) {
                self.y_origin.set(y_origin);

                self.obj().layout_manager().layout_changed();
            }
        }
    }

    glib::wrapper! {
        pub struct KpInteractiveGraphLayoutChild(ObjectSubclass<imp::KpInteractiveGraphLayoutChild>)
            @extends gtk::LayoutChild,
            @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
    }

    impl KpInteractiveGraphLayoutChild {
        pub fn new(layout_manager: &gtk::LayoutManager, child: &gtk::Widget) -> Self {
            glib::Object::builder()
                .property("layout-manager", &*layout_manager)
                .property("child-widget", &*child)
                .build()
        }
    }
}

mod layout {
    use super::*;

    mod imp {
        use super::*;

        #[derive(Debug, Default, glib::Properties)]
        #[properties(wrapper_type = super::KpInteractiveGraphLayout)]
        pub struct KpInteractiveGraphLayout {
            #[property(get, set)]
            x_bound: Cell<i32>,
            #[property(get, set)]
            y_bound: Cell<i32>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for KpInteractiveGraphLayout {
            const NAME: &'static str = "KpInteractiveGraphLayout";
            type Type = super::KpInteractiveGraphLayout;
            type ParentType = gtk::LayoutManager;
        }

        #[glib::derived_properties]
        impl ObjectImpl for KpInteractiveGraphLayout {}
        impl LayoutManagerImpl for KpInteractiveGraphLayout {
            fn measure(
                &self,
                _widget: &gtk::Widget,
                orientation: gtk::Orientation,
                _for_size: i32,
            ) -> (i32, i32, i32, i32) {
                match orientation {
                    gtk::Orientation::Vertical => {
                        (CHART_HEIGHT as i32, CHART_HEIGHT as i32, -1, -1)
                    }
                    gtk::Orientation::Horizontal => (100, 100, -1, -1),
                    _ => unreachable!(),
                }
            }

            fn allocate(&self, widget: &gtk::Widget, width: i32, height: i32, _baseline: i32) {
                let mut child = widget
                    .first_child()
                    .expect("graph has at least one datapoint");
                loop {
                    let (req, _) = child.preferred_size();

                    let layout_child: KpInteractiveGraphLayoutChild =
                        self.obj().layout_child(&child).downcast().unwrap();

                    let (mut x, mut y) = graph_to_widget_coords(
                        layout_child.x_origin(),
                        layout_child.y_origin(),
                        self.x_bound.get(),
                        self.y_bound.get(),
                        width,
                        height,
                    );

                    x -= req.width() / 2;
                    y -= req.height() / 2;

                    child.size_allocate(&gtk::Allocation::new(x, y, req.width(), req.height()), -1);

                    if let Some(next_child) = child.next_sibling() {
                        child = next_child;
                    } else {
                        break;
                    }
                }
            }

            fn create_layout_child(
                &self,
                _container: &gtk::Widget,
                child: &gtk::Widget,
            ) -> gtk::LayoutChild {
                KpInteractiveGraphLayoutChild::new(&*self.obj().upcast_ref(), &child).upcast()
            }
        }
    }

    glib::wrapper! {
        pub struct KpInteractiveGraphLayout(ObjectSubclass<imp::KpInteractiveGraphLayout>)
            @extends gtk::LayoutManager;
    }
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct KpInteractiveGraph {
        pub accuracy_datapoints: RefCell<Vec<(usize, f64)>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpInteractiveGraph {
        const NAME: &'static str = "KpInteractiveGraph";
        type Type = super::KpInteractiveGraph;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<layout::KpInteractiveGraphLayout>();
        }
    }

    impl ObjectImpl for KpInteractiveGraph {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().set_valign(gtk::Align::End);
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for KpInteractiveGraph {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let width = self.obj().width();
            let height = self.obj().height();

            let style_manager = adw::StyleManager::default();
            let accent = style_manager.accent_color_rgba();
            let dimmed = match (style_manager.is_dark(), style_manager.is_high_contrast()) {
                (false, false) => gdk::RGBA::new(0.6, 0.6, 0.6, 1.),
                (false, true) => gdk::RGBA::new(0.2, 0.2, 0.2, 1.),
                (true, false) => gdk::RGBA::new(0.4, 0.4, 0.4, 1.),
                (true, true) => gdk::RGBA::new(0.8, 0.8, 0.8, 1.),
            };

            let mut child = self
                .obj()
                .first_child()
                .expect("graph has at least one datapoint");

            loop {
                if let Some(next_child) = child.next_sibling() {
                    let p1 = child
                        .compute_bounds(&*self.obj())
                        .expect("child is allocated")
                        .center();
                    let p2 = next_child
                        .compute_bounds(&*self.obj())
                        .expect("child is allocated")
                        .center();

                    let path = gsk::PathBuilder::new();
                    path.move_to(p1.x(), p1.y());
                    path.line_to(p2.x(), p2.y());

                    let path = path.to_path();

                    let stroke = gsk::Stroke::new(2.);

                    snapshot.append_stroke(&path, &stroke, &accent);

                    self.obj().snapshot_child(&child, &*snapshot);
                    child = next_child
                } else {
                    self.obj().snapshot_child(&child, &*snapshot);
                    break;
                }
            }

            let layout_manager = self
                .obj()
                .layout_manager()
                .unwrap()
                .downcast::<KpInteractiveGraphLayout>()
                .unwrap();

            let path = gsk::PathBuilder::new();

            let (start_time, start_accuracy) =
                self.accuracy_datapoints.borrow().get(0).unwrap().clone();
            let (start_x, start_y) = graph_to_widget_coords(
                start_time as i32,
                (start_accuracy * 100.).floor() as i32,
                layout_manager.x_bound(),
                100,
                width,
                height,
            );
            path.move_to(start_x as f32, start_y as f32);

            for (time_index, accuracy) in self.accuracy_datapoints.borrow().iter().skip(1) {
                let (x, y) = graph_to_widget_coords(
                    *time_index as i32,
                    (*accuracy * 100.).floor() as i32,
                    layout_manager.x_bound(),
                    100,
                    width,
                    height,
                );

                path.line_to(x as f32, y as f32);
            }

            let path = path.to_path();

            let stroke = gsk::Stroke::new(2.);
            stroke.set_dash(&[4., 2.]);

            snapshot.append_stroke(&path, &stroke, &dimmed);
        }
    }
}

glib::wrapper! {
    pub struct KpInteractiveGraph(ObjectSubclass<imp::KpInteractiveGraph>)
        @extends gtk::Widget;
}

impl KpInteractiveGraph {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn insert_with_coordinates(&self, widget: &impl IsA<gtk::Widget>, x: i32, y: i32) {
        widget.set_parent(&*self);

        let layout_manager = self
            .layout_manager()
            .expect("layout manager was set at class init")
            .downcast::<KpInteractiveGraphLayout>()
            .unwrap();

        let layout_child = layout_manager
            .layout_child(&*widget)
            .downcast::<KpInteractiveGraphLayoutChild>()
            .unwrap();

        layout_child.set_x_origin(x);
        layout_child.set_y_origin(y);

        layout_manager.set_x_bound(layout_manager.x_bound().max(x));
        layout_manager.set_y_bound(
            layout_manager
                .y_bound()
                .max(((y / Y_BOUND_GROW_STEPS as i32) + 1) * Y_BOUND_GROW_STEPS as i32),
        );
    }

    pub fn insert_accuracy_datapoint(&self, time_index: usize, accuracy: f64) {
        self.imp()
            .accuracy_datapoints
            .borrow_mut()
            .push((time_index, accuracy));
    }
}

fn graph_to_widget_coords(
    x: i32,
    y: i32,
    x_bound: i32,
    y_bound: i32,
    width: i32,
    height: i32,
) -> (i32, i32) {
    let trans_x = x as f64 * (width as f64 / x_bound as f64);
    let trans_y = height as f64 - (y as f64 * (height as f64 / y_bound as f64));

    (trans_x.floor() as i32, trans_y.floor() as i32)
}
