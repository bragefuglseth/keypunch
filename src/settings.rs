use gtk::gio;
use gtk::prelude::*;

// Must match their corresponding gschema enumss
pub const SESSION_TYPE_VALUES: &'static [&str] = &["Simple", "Advanced", "Custom"];
pub const SESSION_DURATION_VALUES: &'static [&str] = &["Sec15", "Sec30", "Min1", "Min5", "Min10"];

pub fn bind_dropdown_selected(
    settings: &gio::Settings,
    dropdown: &gtk::DropDown,
    key: &str,
    values: &'static [&str],
) {
    settings
        .bind(key, dropdown, "selected")
        .mapping(|stored_variant, _| {
            let index = values
                .iter()
                .position(|value| *value == stored_variant.get::<String>().unwrap())
                .expect("values array corresponds to gschema enum");
            Some((index as u32).to_value())
        })
        .set_mapping(|index_value, _| {
            let value = values
                .get(index_value.get::<u32>().unwrap() as usize)
                .expect("values array corresponds to gschema enum")
                .to_variant();
            Some(value)
        })
        .build();
}
