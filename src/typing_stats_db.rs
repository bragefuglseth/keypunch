use gtk::glib;
use rusqlite::Connection;
use std::path::PathBuf;

const DB_FILENAME: &'static str = "typing_stats.sqlite";

pub struct TypingStatsDb(Connection);

impl Default for TypingStatsDb {
    fn default() -> Self {
        let mut path = glib::user_data_dir();
        path.push(DB_FILENAME);

        TypingStatsDb::setup(path).unwrap()
    }
}

impl TypingStatsDb {
    pub fn setup(location: PathBuf) -> rusqlite::Result<TypingStatsDb> {
        let connection = Connection::open(location)?;

        Ok(TypingStatsDb(connection))
    }
}
