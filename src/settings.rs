use std::collections::HashMap;

use ron;
use serde::{Deserialize};

#[derive(Debug, Deserialize)]
enum Setting {
    TitleSetting { key: String, description: String, subsettings: Option<Vec<Setting>>, },
    BoolSetting { key: String, description: String, default: Option<bool>, },
}

pub fn initialise_settings(ron_string: &str) -> Option<HashMap<String, bool>> {
    let mut settings_map: HashMap<String, bool> = HashMap::new();

    let settings = ron::from_str::<Vec<Setting>>(ron_string);
    if settings.is_err() {
        let err = settings.unwrap_err();
        asr::print_message(&format!("{:?}", err));
        
        return None;
    }
    let settings = settings.unwrap();
    asr::print_message(&format!("DEST, {:?}", settings));

    for setting in settings.iter() {
        asr::print_message(&format!("DEST3, {:?}", setting));
        initialise_setting(&mut settings_map, &setting, 0);
    }

    Some(settings_map)
}

fn initialise_setting(settings_map: &mut HashMap<String, bool>, setting: &Setting, heading_level: u32) {
    asr::print_message("in the thingo");
    match setting {
        Setting::TitleSetting { key, description, subsettings } => {
            asr::print_message(&format!("{}: {}", key, description));
            // asr::user_settings::add_title("a", "a", 0);
            if subsettings.is_none() {
                return;
            }

            for ss in subsettings.as_ref().unwrap() {
                initialise_setting(settings_map, &ss, heading_level + 1);
            }
        }
        Setting::BoolSetting { key, description, default } => {
            asr::print_message(&format!("{}", key));
            let default_value = default.unwrap_or_default();
            settings_map.insert(String::from(key), asr::user_settings::add_bool(key, description, default_value));
        },
    }
}