use crate::typing_test_utils::*;
use anyhow::Result;
use gettextrs::gettext;
use i18n_format::i18n_fmt;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use time::{Date, Duration, Month, OffsetDateTime, Time};

pub struct ChartItem {
    pub title: String,
    pub time_index: usize,
    pub wpm: f64,
    pub accuracy: f64,
}

pub struct PeriodSummary {
    pub wpm: f64,
    pub accuracy: f64,
    pub finish_rate: f64,
    pub practice_time: String, // TODO: store this as a time::Duration instead
}

pub const DATABASE: LazyLock<TypingStatsDb> = LazyLock::new(|| {
    let path = gtk::glib::user_data_dir().join("keypunch");

    TypingStatsDb::setup(path).unwrap()
});

refinery::embed_migrations!("./src/migrations");

pub struct TypingStatsDb(Connection);

impl TypingStatsDb {
    pub fn setup(location: PathBuf) -> Result<TypingStatsDb> {
        fs::create_dir_all(&location)?;

        let mut conn = Connection::open(&location.join("statistics.sqlite")).unwrap();
        migrations::runner().run(&mut conn).unwrap();

        Ok(TypingStatsDb(conn))
    }

    pub fn push_summary(&self, summary: &TestSummary) -> Result<()> {
        let (test_type, language, duration) = match summary.config {
            TestConfig::Finite => ("Custom", None, None),
            TestConfig::Generated {
                difficulty,
                language,
                duration,
                ..
            } => {
                let difficulty = match difficulty {
                    GeneratedTestDifficulty::Simple => "Simple",
                    GeneratedTestDifficulty::Advanced => "Advanced",
                };
                (
                    difficulty,
                    Some(language.to_string()),
                    Some(duration.to_string()),
                )
            }
        };

        self.0.execute(
            "
            INSERT INTO tests (
                timestamp,
                finished,
                test_type,
                language,
                duration,
                real_duration,
                wpm,
                accuracy
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            (
                summary.start_timestamp,
                summary.finished,
                test_type,
                language,
                duration,
                summary.real_duration.as_secs(),
                summary.wpm,
                summary.accuracy,
            ),
        )?;
        Ok(())
    }

    // Returns the average WPM and accuracy at a given date
    pub fn average_from_period(
        &self,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> rusqlite::Result<(f64, f64)> {
        self.0.query_row(
            "SELECT     AVG(wpm), AVG(accuracy)
            FROM        tests
            WHERE       finished = TRUE
            AND         UNIXEPOCH(timestamp) BETWEEN ? AND ?
            AND         test_type IN ('Simple', 'Advanced')",
            (start.unix_timestamp(), end.unix_timestamp()),
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
    }

    pub fn get_past_month(&self) -> Option<Vec<ChartItem>> {
        let now = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());
        let today_start = now.replace_time(Time::MIDNIGHT);
        let today_end = now.replace_time(Time::MAX);

        let mut month_data: Vec<ChartItem> = (0..30)
            .filter_map(|n| {
                let start = today_start - Duration::days(29 - n);
                let end = today_end - Duration::days(29 - n);

                if let Ok((wpm, accuracy)) = DATABASE.average_from_period(start, end) {
                    Some(ChartItem {
                        title: formatted_date(start.date()),
                        time_index: n as usize,
                        wpm,
                        accuracy,
                    })
                } else {
                    None
                }
            })
            .collect();

        let &ChartItem {
            time_index: time_offset,
            ..
        } = month_data.get(0)?;

        for item in month_data.iter_mut() {
            item.time_index -= time_offset;
        }

        Some(month_data)
    }

    pub fn get_past_year(&self) -> Option<Vec<ChartItem>> {
        let now = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());

        let mut month_data: Vec<ChartItem> = (0..12)
            .filter_map(|n| {
                let mut start = now.replace_day(1).unwrap().replace_time(Time::MIDNIGHT);

                for _ in 0..(11 - n) {
                    let prev_month_length = start.month().previous().length(start.year());
                    start -= Duration::days(prev_month_length as i64)
                }

                let month_length = start.month().length(start.year());
                let end = start
                    .replace_day(month_length)
                    .unwrap()
                    .replace_time(Time::MAX);

                if let Ok((wpm, accuracy)) = DATABASE.average_from_period(start, end) {
                    Some(ChartItem {
                        title: formatted_month(start.date()),
                        time_index: n as usize,
                        wpm,
                        accuracy,
                    })
                } else {
                    None
                }
            })
            .collect();

        let &ChartItem {
            time_index: time_offset,
            ..
        } = month_data.get(0)?;

        for item in month_data.iter_mut() {
            item.time_index -= time_offset;
        }

        Some(month_data)
    }

    pub fn last_month_summary(&self) -> Option<PeriodSummary> {
        let now = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());

        let start = now.replace_time(Time::MIDNIGHT) - Duration::days(27);
        let (wpm, accuracy) = self.average_from_period(start, now).ok()?;

        let finish_rate = self
            .0
            .query_row(
                "SELECT     SUM(finished), COUNT(*)
            FROM        tests
            WHERE       UNIXEPOCH(timestamp) BETWEEN ? AND ?
            AND         test_type IN ('Simple', 'Advanced')",
                (start.unix_timestamp(), now.unix_timestamp()),
                |row| Ok(row.get::<_, f64>(0)? / row.get::<_, f64>(1)?),
            )
            .ok()?;

        Some(PeriodSummary {
            wpm,
            accuracy,
            finish_rate,
            practice_time: String::from("todo"),
        })
    }
}

fn formatted_date(date: Date) -> String {
    let day = date.day();

    match date.month() {
        // Translators: This is a date. The {} is replaced with a number.
        Month::January => i18n_fmt! { i18n_fmt("January {}", day) },
        Month::February => i18n_fmt! { i18n_fmt("February {}", day) },
        Month::March => i18n_fmt! { i18n_fmt("March {}", day) },
        Month::April => i18n_fmt! { i18n_fmt("April {}", day) },
        Month::May => i18n_fmt! { i18n_fmt("May {}", day) },
        Month::June => i18n_fmt! { i18n_fmt("June {}", day) },
        Month::July => i18n_fmt! { i18n_fmt("July {}", day) },
        Month::August => i18n_fmt! { i18n_fmt("August {}", day) },
        Month::September => i18n_fmt! { i18n_fmt("September {}", day) },
        Month::October => i18n_fmt! { i18n_fmt("October {}", day) },
        Month::November => i18n_fmt! { i18n_fmt("November {}", day) },
        Month::December => i18n_fmt! { i18n_fmt("December {}", day) },
    }
}

fn formatted_month(date: Date) -> String {
    let year = date.year();

    match date.month() {
        // Translators: This is a month label for the "monthly" view in the statistics dialog.
        // The {} is replaced with a year.
        Month::January => i18n_fmt! { i18n_fmt("January {}", year) },
        Month::February => i18n_fmt! { i18n_fmt("February {}", year) },
        Month::March => i18n_fmt! { i18n_fmt("March {}", year) },
        Month::April => i18n_fmt! { i18n_fmt("April {}", year) },
        Month::May => i18n_fmt! { i18n_fmt("May {}", year) },
        Month::June => i18n_fmt! { i18n_fmt("June {}", year) },
        Month::July => i18n_fmt! { i18n_fmt("July {}", year) },
        Month::August => i18n_fmt! { i18n_fmt("August {}", year) },
        Month::September => i18n_fmt! { i18n_fmt("September {}", year) },
        Month::October => i18n_fmt! { i18n_fmt("October {}", year) },
        Month::November => i18n_fmt! { i18n_fmt("November {}", year) },
        Month::December => i18n_fmt! { i18n_fmt("December {}", year) },
    }
}
