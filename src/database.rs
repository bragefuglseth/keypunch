use crate::typing_test_utils::*;
use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::UNIX_EPOCH;
use std::fs;

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
                summary.start_timestamp
                    .duration_since(UNIX_EPOCH)
                    .expect("System time is always post-UNIX epoch")
                    .as_secs(),
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
}
