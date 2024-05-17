use strum_macros::{Display as EnumDisplay, EnumIter, EnumString, EnumMessage};

#[derive(Clone, Copy, Default, EnumString, EnumDisplay, EnumIter)]
pub enum SessionType {
    #[default]
    Simple,
    Advanced,
    Custom,
}

impl SessionType {
    pub fn ui_string(&self) -> String {
        match self {
            SessionType::Simple => String::from("Simple"),
            SessionType::Advanced => String::from("Advanced"),
            SessionType::Custom => String::from("Custom"),
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, EnumString, EnumDisplay, EnumIter)]
pub enum SessionDuration {
    #[default]
    Sec15,
    Sec30,
    Min1,
    Min5,
    Min10,
}

impl SessionDuration {
    pub fn ui_string(&self) -> String {
        match self {
            SessionDuration::Sec15 => String::from("15 seconds"),
            SessionDuration::Sec30 => String::from("30 seconds"),
            SessionDuration::Min1 => String::from("1 minute"),
            SessionDuration::Min5 => String::from("5 minutes"),
            SessionDuration::Min10 => String::from("10 minutes"),
        }
    }
}

#[derive(Clone, Copy, Default, EnumDisplay, EnumString, EnumIter, EnumMessage, PartialEq)]
pub enum Language {
    #[default]
    #[strum(message = "English (US)", to_string = "en_US")]
    EnglishUS,
    #[strum(message = "Norsk bokmål", to_string = "nb_NO")]
    NorwegianBokmaal,
    #[strum(message = "Norsk nynorsk", to_string = "nn_NO")]
    NorwegianNynorsk,
    #[strum(message = "Español", to_string = "es_ES")]
    Spanish,
    #[strum(message = "Svenska", to_string = "se_SE")]
    Swedish,
}
