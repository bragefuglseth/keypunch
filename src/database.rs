use crate::typing_test_utils::TestSummary;
use crate::text_generation::Language;
use anyhow::Result;
use gtk::glib;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DB_FILENAME: &'static str = "statistics.sqlite";

pub const DATABASE: LazyLock<TypingStatsDb> = LazyLock::new(|| {
    let mut path = glib::user_data_dir();
    path.push(DB_FILENAME);

    TypingStatsDb::setup(path)
});

refinery::embed_migrations!("./src/migrations");

pub struct TypingStatsDb(Connection);

impl TypingStatsDb {
    pub fn setup(location: PathBuf) -> TypingStatsDb {
        let mut conn = Connection::open(location).unwrap();
        migrations::runner().run(&mut conn).unwrap();

        TypingStatsDb(conn)
    }

    // TODO
    // pub fn push_test(&self, data: &TestSummary) -> Result<()> {
    //     self.0.execute(
    //         "
    //         INSERT INTO tests (
    //             timestamp,
    //             finished,
    //             test_type,
    //             language,
    //             duration,
    //             wpm,
    //             accuracy
    //         )
    //         VALUES (?, ?, ?, ?, ?, ?, ?)",
    //         (
    //             data.timestamp
    //                 .duration_since(UNIX_EPOCH)
    //                 .expect("System time is always post-UNIX epoch")
    //                 .as_secs(),
    //             data.finished,
    //             data.test_type.to_string(),
    //             data.language.map(|l| l.to_string()),
    //             data.duration.as_secs(),
    //             data.wpm,
    //             data.accuracy,
    //         ),
    //     )?;
    //     Ok(())
    // }
}
