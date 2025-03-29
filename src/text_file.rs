use gtk::gio::{
    self,
    prelude::{FileExt, FileMonitorExt},
};

enum State {
    NotLoaded,
    Loaded,
    Renamed,
    Deleted,
    ExistedLastTime,
}

pub struct TextFile {
    path: String,
    state: State,
}

impl TextFile {
    pub fn new(path: String) -> Self {
        let textfile = TextFile {
            path: path.clone(),
            state: State::NotLoaded,
        };

        let file = gio::File::for_path(path);
        let monitor = file
            .monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE)
            .unwrap_or_else(|err| panic!("Failed to create file monitor: {err}"));

        monitor.connect_changed(
            move |_monitor: &gio::FileMonitor,
                  _file: &gio::File,
                  _other_file: Option<&gio::File>,
                  event: gio::FileMonitorEvent| {
                if event == gio::FileMonitorEvent::ChangesDoneHint {}
            },
        );

        textfile
    }
}
