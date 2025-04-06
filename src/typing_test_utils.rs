/* typing_test_utils.rs
 *
 * SPDX-FileCopyrightText: © 2024–2025 Brage Fuglseth <bragefuglseth@gnome.org>
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

use crate::text_generation::Language;
use crate::text_utils::calculate_wpm;
use gettextrs::gettext;
use gtk::gio;
use gtk::prelude::*;
use std::str::FromStr;
use std::time::{Duration, Instant};
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};
use time::OffsetDateTime;

#[derive(Clone, Copy, PartialEq, EnumString, EnumDisplay)]
pub enum GeneratedTestDifficulty {
    Simple,
    Advanced,
}

impl GeneratedTestDifficulty {
    pub fn from_settings_string(s: &str) -> Option<Self> {
        match s {
            "simple" => Some(GeneratedTestDifficulty::Simple),
            "advanced" => Some(GeneratedTestDifficulty::Advanced),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum TestConfig {
    Finite,
    Generated {
        language: Language,
        difficulty: GeneratedTestDifficulty,
        duration: TestDuration,
    },
}

impl TestConfig {
    pub fn from_settings(settings: &gio::Settings) -> Self {
        match settings.string("session-type").as_str() {
            difficulty_string @ ("Simple" | "Advanced") => TestConfig::Generated {
                language: Language::from_str(&settings.string("text-language")).unwrap(),
                difficulty: GeneratedTestDifficulty::from_str(&difficulty_string).unwrap(),
                duration: TestDuration::from_str(&settings.string("session-duration")).unwrap(),
            },
            "Custom" => TestConfig::Finite,
            _ => panic!("invalid settings value for `session-type` key"),
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, EnumString, EnumDisplay, EnumIter)]
pub enum TestDuration {
    #[default]
    Sec15,
    Sec30,
    Min1,
    Min5,
    Min10,
}

impl TestDuration {
    pub fn ui_string(&self) -> String {
        match self {
            TestDuration::Sec15 => gettext("15 seconds"),
            TestDuration::Sec30 => gettext("30 seconds"),
            TestDuration::Min1 => gettext("1 minute"),
            TestDuration::Min5 => gettext("5 minutes"),
            TestDuration::Min10 => gettext("10 minutes"),
        }
    }

    pub fn english_string(&self) -> &str {
        match self {
            TestDuration::Sec15 => "15 seconds",
            TestDuration::Sec30 => "30 seconds",
            TestDuration::Min1 => "1 minute",
            TestDuration::Min5 => "5 minutes",
            TestDuration::Min10 => "10 minutes",
        }
    }
}

#[derive(Copy, Clone)]
pub enum PresenceState {
    Ready,
    Typing,
    Results,
}

impl PresenceState {
    pub fn english_string(&self) -> &str {
        match self {
            PresenceState::Ready => "Ready to start",
            PresenceState::Typing => "Typing",
            PresenceState::Results => "Viewing results",
        }
    }
}

#[derive(Clone, Copy)]
pub struct TypingTest {
    pub config: TestConfig,
    pub start_instant: Instant,
    pub start_system_time: OffsetDateTime,
}

impl TypingTest {
    pub fn new(config: TestConfig) -> Self {
        TypingTest {
            config,
            start_instant: Instant::now(),
            start_system_time: OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct TestSummary {
    pub config: TestConfig,
    pub real_duration: Duration,
    pub wpm: f64,
    pub start_timestamp: OffsetDateTime,
    pub accuracy: f64,
    pub finished: bool,
}

impl TestSummary {
    pub fn new(
        start_timestamp: OffsetDateTime,
        start_instant: Instant,
        end_instant: Instant,
        config: TestConfig,
        original: &str,
        typed: &str,
        keystrokes: &Vec<(Instant, bool)>,
        finished: bool,
    ) -> Self {
        let real_duration = end_instant.duration_since(start_instant);
        let correct_keystrokes = keystrokes.iter().filter(|(_, correct)| *correct).count();
        let total_keystrokes = keystrokes.len();

        TestSummary {
            config,
            real_duration,
            wpm: calculate_wpm(real_duration, &original, &typed),
            start_timestamp,
            accuracy: correct_keystrokes as f64 / total_keystrokes as f64,
            finished,
        }
    }
}
