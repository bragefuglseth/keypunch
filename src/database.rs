use crate::typing_test_utils::*;
use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::fs;
use time::{OffsetDateTime, Duration, Time};

pub struct ChartItem {
    pub title: String,
    pub time_index: usize,
    pub wpm: f64,
    pub accuracy: f64,
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
            TestConfig::Generated { difficulty, language, duration, .. } => {
                let difficulty = match difficulty {
                    GeneratedTestDifficulty::Simple => "Simple",
                    GeneratedTestDifficulty::Advanced => "Advanced",
                };
                (difficulty, Some(language.to_string()), Some(duration.to_string()))
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
    pub fn average_from_period(&self, start: OffsetDateTime, end: OffsetDateTime) -> rusqlite::Result<(f64, f64)> {
        self.0.query_row(
            "SELECT     AVG(wpm), AVG(accuracy)
            FROM        tests
            WHERE       finished = TRUE
            AND         UNIXEPOCH(timestamp) BETWEEN ? AND ?
            AND         test_type IN ('Simple', 'Advanced')",
            (start.unix_timestamp(), end.unix_timestamp()),
            |row| Ok((row.get(0)?, row.get(1)?))
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
                        title: "Test".to_string(),
                        time_index: n as usize,
                        wpm,
                        accuracy,
                    })
                } else {
                    None
                }
            })
            .collect();

        let &ChartItem { time_index: time_offset, .. } = month_data.get(0)?;

        for item in month_data.iter_mut() {
            item.time_index -= time_offset;
        }

        Some(month_data)
    }
}
