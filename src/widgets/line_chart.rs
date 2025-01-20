/* line_chart.rs
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

use crate::widgets::KpInteractiveGraph;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::{Cell, OnceCell, RefCell};

struct ChartItem {
    pub title: String,
    pub time_index: usize,
    pub wpm: f64,
    pub accuracy: f64,
}

// The Y bound (y boundary) is the tallest logical y-height shown on the diagram
// (WPM units, not pixels).
// It grows in multiples of Y_BOUND_GROW_STEPS at a time.
pub const Y_BOUND_GROW_STEPS: usize = 50;
pub const RULER_COUNT: usize = 6;
pub const CHART_HEIGHT: usize = 200;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct KpLineChart {
        data: RefCell<Vec<ChartItem>>,
        main_box: OnceCell<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpLineChart {
        const NAME: &'static str = "KpLineChart";
        type Type = super::KpLineChart;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }
    }

    impl ObjectImpl for KpLineChart {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let main_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
            main_box.set_parent(&*obj);
            self.main_box
                .set(main_box)
                .expect("main box hasn't been initialized");

            let accuracy_legend = adw::Bin::builder()
                .css_classes(["accuracy-legend"])
                .width_request(34)
                .halign(gtk::Align::Start)
                .build();
            let accuracy_header = gtk::Label::builder()
                .label("Accuracy")
                .xalign(0.)
                .hexpand(true)
                .css_classes(["caption", "dimmed"])
                .build();
            let accuracy_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
            accuracy_box.append(&accuracy_legend);
            accuracy_box.append(&accuracy_header);

            let wpm_legend = adw::Bin::builder()
                .css_classes(["wpm-legend"])
                .width_request(34)
                .halign(gtk::Align::End)
                .build();
            let wpm_header = gtk::Label::builder()
                .label("Words per Minute")
                .xalign(1.)
                .hexpand(true)
                .css_classes(["caption", "accent"])
                .build();
            let wpm_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
            wpm_box.append(&wpm_legend);
            wpm_box.append(&wpm_header);

            let header_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(6)
                .margin_start(12)
                .margin_end(12)
                .margin_top(12)
                .margin_bottom(12)
                .build();

            header_box.append(&accuracy_box);
            header_box.append(&wpm_box);

            self.main_box().append(&header_box);

            *self.data.borrow_mut() = vec![
                ChartItem {
                    title: String::from("January"),
                    time_index: 0,
                    wpm: 98.,
                    accuracy: 0.97,
                },
                ChartItem {
                    title: String::from("January"),
                    time_index: 1,
                    wpm: 100.,
                    accuracy: 0.98,
                },
                ChartItem {
                    title: String::from("January"),
                    time_index: 2,
                    wpm: 102.,
                    accuracy: 0.95,
                },
                ChartItem {
                    title: String::from("January"),
                    time_index: 3,
                    wpm: 100.,
                    accuracy: 0.96,
                },
                ChartItem {
                    title: String::from("January"),
                    time_index: 5,
                    wpm: 105.,
                    accuracy: 0.99,
                },
                ChartItem {
                    title: String::from("January"),
                    time_index: 6,
                    wpm: 103.,
                    accuracy: 0.93,
                },
                ChartItem {
                    title: String::from("January"),
                    time_index: 8,
                    wpm: 98.,
                    accuracy: 0.94,
                },
                ChartItem {
                    title: String::from("January"),
                    time_index: 9,
                    wpm: 99.,
                    accuracy: 0.98,
                },
            ];

            let highest_y_val = self
                .data
                .borrow()
                .iter()
                .max_by(|ChartItem { wpm: a, .. }, ChartItem { wpm: b, .. }| {
                    a.partial_cmp(&b).expect("values are comparable")
                })
                .map(|item| item.wpm)
                .unwrap_or(0.)
                .floor() as usize;

            let y_bound = ((highest_y_val / Y_BOUND_GROW_STEPS) + 1) * Y_BOUND_GROW_STEPS;

            let ruler_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

            let ruler_height = (CHART_HEIGHT / (RULER_COUNT - 1)) as i32;

            for i in 0..RULER_COUNT {
                let percentage = (i as f64 / (RULER_COUNT - 1) as f64 * 100.).floor() as usize;

                let acc_label = gtk::Label::builder()
                    // TODO: Make translatable
                    .label(&format!("{percentage}%"))
                    .xalign(0.)
                    .margin_start(12)
                    .halign(gtk::Align::Fill)
                    .hexpand(true)
                    .css_classes(["dimmed", "caption"])
                    .build();

                let wpm_label = gtk::Label::builder()
                    .label(
                        (((y_bound as f64 / (RULER_COUNT - 1) as f64) * i as f64).floor() as usize)
                            .to_string(),
                    )
                    .xalign(1.)
                    .margin_end(12)
                    .css_classes(["accent", "caption"])
                    .build();

                let ylabel_box = gtk::Box::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .halign(gtk::Align::Fill)
                    .margin_bottom(2)
                    .build();
                ylabel_box.append(&acc_label);
                ylabel_box.append(&wpm_label);

                let separator = gtk::Separator::builder()
                    .orientation(gtk::Orientation::Vertical)
                    .build();

                let ruler = gtk::Box::builder()
                    .orientation(gtk::Orientation::Vertical)
                    .valign(gtk::Align::End)
                    .build();

                ruler.append(&ylabel_box);
                ruler.append(&separator);

                let bin = adw::Bin::builder().child(&ruler).build();

                if i != (RULER_COUNT - 1) {
                    bin.set_height_request(ruler_height);
                }

                ruler_box.prepend(&bin);
            }

            let interactive_graph = KpInteractiveGraph::new();
            interactive_graph.set_margin_start(54);
            interactive_graph.set_margin_end(54);

            for item in self.data.borrow().iter() {
                let dot = adw::Bin::builder()
                    .width_request(6)
                    .height_request(6)
                    .valign(gtk::Align::Center)
                    .halign(gtk::Align::Center)
                    .css_classes(["line-chart-dot"])
                    .build();

                let popover_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
                popover_box.append(
                    &gtk::Label::builder()
                        .label(&item.title)
                        .css_classes(["heading"])
                        .build(),
                );

                popover_box.append(
                    &gtk::Label::builder()
                        .label(&format!(
                            "<b>{:.0}</b> words per minute\n<b>{}%</b> accuracy",
                            &item.wpm.floor(),
                            (item.accuracy * 100.).floor()
                        ))
                        .justify(gtk::Justification::Center)
                        .use_markup(true)
                        .build(),
                );

                let popover = gtk::Popover::builder().child(&popover_box).build();

                let btn = gtk::MenuButton::builder()
                    .css_classes(["line-chart-button"])
                    .direction(gtk::ArrowType::Up)
                    .child(&dot)
                    .popover(&popover)
                    .build();

                interactive_graph.insert_with_coordinates(
                    &btn,
                    item.time_index as i32,
                    item.wpm.floor() as i32,
                );

                interactive_graph.insert_accuracy_datapoint(item.time_index, item.accuracy);
            }

            let overlay = gtk::Overlay::new();
            overlay.set_child(Some(&ruler_box));
            overlay.add_overlay(&interactive_graph);

            self.main_box().append(&overlay);
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for KpLineChart {}

    impl KpLineChart {
        fn main_box(&self) -> &gtk::Box {
            self.main_box
                .get()
                .expect("main box initialized during construction")
        }
    }
}

glib::wrapper! {
    pub struct KpLineChart(ObjectSubclass<imp::KpLineChart>)
        @extends gtk::Widget;
}
