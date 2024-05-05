use std::path::{self, PathBuf};

use godot::engine::{ProjectSettings, RegEx};
use godot::prelude::*;
use unic_langid::LanguageIdentifier;

use super::project_settings::*;

pub fn compute_locale(path: &PathBuf) -> Option<String> {
    let project_settings = ProjectSettings::singleton();

    // 1. File regex.
    let file_regex = project_settings.get_setting(PROJECT_SETTING_LOCALE_BY_FILE_REGEX.into()).stringify();
    if !file_regex.is_empty() {
        let file_name = path.file_name()?;
        let file_name = GString::from(file_name.to_owned().into_string().unwrap());
        // Force regex to be case insensitive.
        let file_regex = "(?i)".to_owned() + &file_regex.to_string();
        let file_regex = RegEx::create_from_string(file_regex.into()).unwrap();
        if let Some(regex_match) = file_regex.search(file_name) {
            // Ensure there is only one capture group.
            if regex_match.get_group_count() > 1 {
                godot_warn!(
                    "{} is set to a RegEx with {} capture groups. Only one should be capturing, the rest should be (?:) non-capturing. \nUsing last capture group as a fallback.",
                    PROJECT_SETTING_LOCALE_BY_FILE_REGEX, regex_match.get_group_count()
                );
            }

            // Get the last capture group's value.
            let locale = regex_match.get_string_ex().name(regex_match.get_group_count().to_variant()).done().to_string();
            if is_valid_locale(&locale) {
                return Some(locale);
            }
        }
    }

    // 2. Folder regex.
    let folder_regex = project_settings.get_setting(PROJECT_SETTING_LOCALE_BY_FOLDER_REGEX.into()).stringify();
    if !folder_regex.is_empty() {
        // Force regex to be case insensitive.
        let folder_regex = "(?i)".to_owned() + &folder_regex.to_string();
        let folder_regex = RegEx::create_from_string(folder_regex.into()).unwrap();
        for folder in path.iter().rev() {
            let folder = folder.to_owned().into_string().unwrap();
            if folder == path::MAIN_SEPARATOR_STR {
                continue;
            }

            if let Some(regex_match) = folder_regex.search(folder.into()) {
                // Ensure there is only one capture group.
                if regex_match.get_group_count() > 1 {
                    godot_warn!(
                        "{} is set to a RegEx with {} capture groups. Only one should be capturing, the rest should be (?:) non-capturing. \nUsing last capture group as a fallback.",
                        PROJECT_SETTING_LOCALE_BY_FOLDER_REGEX, regex_match.get_group_count()
                    );
                }
    
                // Get the last capture group's value.
                let locale = regex_match.get_string_ex().name(regex_match.get_group_count().to_variant()).done().to_string();
                if is_valid_locale(&locale) {
                    return Some(locale);
                }
            }
        }
    }

    // Unable to find a locale.
    None
}

fn is_valid_locale(locale: &str) -> bool {
    if locale.is_empty() {
        return false;
    }

    let identifier = locale.parse::<LanguageIdentifier>();
    identifier.is_ok()
}
