use gtk::gio;
use rand::seq::SliceRandom;
use rand::thread_rng;

fn words_from_lang_code(lang_code: &str) -> Vec<String> {
    let resource_bytes = gio::resources_lookup_data(
        &format!("/dev/bragefuglseth/Keypunch/word_lists/{lang_code}.txt"),
        gio::ResourceLookupFlags::NONE,
    )
    .expect(&format!("word list for \"{}\" exists", lang_code));

    let s = std::str::from_utf8(&resource_bytes).expect("resource byte stream is valid");

    s.lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect()
}

pub mod basic_latin {
    use super::*;
    pub fn simple(lang_code: &str) -> String {
        let word_list = words_from_lang_code(lang_code);

        let mut rng = thread_rng();

        let mut s = String::with_capacity(5000);
        for _ in 0..1000 {
            s.push_str(
                word_list
                    .choose(&mut rng)
                    .expect("word list contains at least 1 word"),
            );
            s.push_str(" ");
        }

        s
    }
}
