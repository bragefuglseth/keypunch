use gettextrs::gettext;
use strum_macros::{Display as EnumDisplay, EnumIter, EnumString};

#[derive(Clone, Copy, Default, PartialEq, EnumString, EnumDisplay, EnumIter)]
pub enum SessionType {
    #[default]
    Simple,
    Advanced,
    Custom,
}

impl SessionType {
    pub fn ui_string(&self) -> String {
        match self {
            SessionType::Simple => gettext("Simple"),
            SessionType::Advanced => gettext("Advanced"),
            SessionType::Custom => gettext("Custom"),
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
            SessionDuration::Sec15 => gettext("15 seconds"),
            SessionDuration::Sec30 => gettext("30 seconds"),
            SessionDuration::Min1 => gettext("1 minute"),
            SessionDuration::Min5 => gettext("5 minutes"),
            SessionDuration::Min10 => gettext("10 minutes"),
        }
    }

    pub fn english_string(&self) -> &str {
        match self {
            SessionDuration::Sec15 => "15 seconds",
            SessionDuration::Sec30 => "30 seconds",
            SessionDuration::Min1 => "1 minute",
            SessionDuration::Min5 => "5 minutes",
            SessionDuration::Min10 => "10 minutes",
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
