use rand::seq::SliceRandom;
use rand::thread_rng;
use include_dir::{include_dir, Dir};
use unicode_segmentation::UnicodeSegmentation;

static EMBEDDED_WORD_LIST_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data/word_lists");

const EMBEDDED_WORD_LIST_LEN: usize = 200;

pub const CHUNK_GRAPHEME_COUNT: usize = 400;

fn words_from_lang_code(lang_code: &str) -> [&'static str; EMBEDDED_WORD_LIST_LEN] {
    let s = EMBEDDED_WORD_LIST_DIR.get_file(&format!("{lang_code}.txt"))
        .expect(&format!("word list for \"{}\" exists", lang_code))
        .contents_utf8()
        .expect("file has valid utf8 contents");

    s.lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .try_into()
        .expect("word list vec can be parsed into [&str; 200]")
}

pub mod basic_latin {
    use super::*;
    pub fn simple(lang_code: &str) -> String {
        let word_list = words_from_lang_code(lang_code);

        let mut rng = thread_rng();

        let mut s = String::new();

        while s.graphemes(true).count() < CHUNK_GRAPHEME_COUNT {
            s.push_str(word_list.choose(&mut rng).expect("word list contains at least 1 word"));
            s.push(' ');
        }

        s
    }
}
