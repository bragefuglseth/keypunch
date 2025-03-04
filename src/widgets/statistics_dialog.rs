/* statistics_dialog.rs
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

use crate::widgets::KpLineChart;
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::glib;
use std::sync::OnceLock;
use crate::database::DATABASE;
use crate::database::ChartItem;
use time::{Time, OffsetDateTime, Duration};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "src/widgets/statistics_dialog.blp")]
    pub struct KpStatisticsDialog {
        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        month_bin: TemplateChild<adw::Bin>,
        #[template_child]
        year_bin: TemplateChild<adw::Bin>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KpStatisticsDialog {
        const NAME: &'static str = "KpStatisticsDialog";
        type Type = super::KpStatisticsDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for KpStatisticsDialog {
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

            self.stack.set_visible_child_name("statistics");

            let header_bar = self.header_bar.get();
            self.scrolled_window
                .vadjustment()
                .bind_property("value", &header_bar, "show-title")
                .transform_to(|_, scroll_position: f64| Some(scroll_position > 0.))
                .sync_create()
                .build();

            let month_data = DATABASE.get_past_month().unwrap(); // TODO: Handle the no data case

            let month_stats_chart = KpLineChart::new(&month_data);

            self.month_bin.set_child(Some(&month_stats_chart));
        }
    }
    impl WidgetImpl for KpStatisticsDialog {}
    impl AdwDialogImpl for KpStatisticsDialog {}
    impl KpStatisticsDialog {}
}

glib::wrapper! {
    pub struct KpStatisticsDialog(ObjectSubclass<imp::KpStatisticsDialog>)
        @extends gtk::Widget, adw::Dialog;
}

impl KpStatisticsDialog {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
