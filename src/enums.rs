use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Clone, Copy, Default)]
pub enum SessionType {
    #[default]
    Simple,
    Advanced,
    Custom,
}

impl SessionType {
    pub fn from_i32(i: i32) -> Option<Self> {
        match i {
            0 => Some(SessionType::Simple),
            1 => Some(SessionType::Advanced),
            2 => Some(SessionType::Custom),
            _ => None,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            SessionType::Simple => String::from("Simple"),
            SessionType::Advanced => String::from("Advanced"),
            SessionType::Custom => String::from("Custom"),
        }
    }

    pub fn string_list() -> gtk::StringList {
        gtk::StringList::new(&[
            &SessionType::Simple.as_string(),
            &SessionType::Advanced.as_string(),
            &SessionType::Custom.as_string(),
        ])
    }
}

#[derive(Copy, Clone, Default, PartialEq)]
pub enum SessionDuration {
    #[default]
    Sec15,
    Sec30,
    Min1,
    Min5,
    Min10,
}

impl SessionDuration {
    pub fn from_i32(i: i32) -> Option<Self> {
        match i {
            0 => Some(SessionDuration::Sec15),
            1 => Some(SessionDuration::Sec30),
            2 => Some(SessionDuration::Min1),
            3 => Some(SessionDuration::Min5),
            4 => Some(SessionDuration::Min10),
            _ => None,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            SessionDuration::Sec15 => String::from("15 seconds"),
            SessionDuration::Sec30 => String::from("30 seconds"),
            SessionDuration::Min1 => String::from("1 minute"),
            SessionDuration::Min5 => String::from("5 minutes"),
            SessionDuration::Min10 => String::from("10 minutes"),
        }
    }

    pub fn string_list() -> gtk::StringList {
        gtk::StringList::new(&[
            &SessionDuration::Sec15.as_string(),
            &SessionDuration::Sec30.as_string(),
            &SessionDuration::Min1.as_string(),
            &SessionDuration::Min5.as_string(),
            &SessionDuration::Min10.as_string(),
        ])
    }
}

#[derive(Clone, Copy, Default, EnumDisplay, EnumString, EnumIter, PartialEq)]
pub enum Language {
    #[default]
    #[strum(to_string = "en_US")]
    EnglishUS,
    #[strum(to_string = "nb_NO")]
    NorwegianBokmaal,
    #[strum(to_string = "nn_NO")]
    NorwegianNynorsk,
    #[strum(to_string = "es_ES")]
    Spanish,
    #[strum(to_string = "se_SE")]
    Swedish,
}

impl Language {
    pub fn pretty_name(&self) -> &'static str {
        match self {
            Language::EnglishUS => "English (US)",
            Language::NorwegianBokmaal => "Norsk bokmål",
            Language::NorwegianNynorsk => "Norsk nynorsk",
            Language::Spanish => "Español",
            Language::Swedish => "Svenska",
        }
    }
}
