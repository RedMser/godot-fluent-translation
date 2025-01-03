use std::path::{self, PathBuf};

use godot::classes::{ProjectSettings, RegEx, RegExMatch};
use godot::prelude::*;
use unic_langid::LanguageIdentifier;

use crate::utils::get_single_regex_match;

use super::project_settings::*;

pub fn compute_message_pattern(path: &PathBuf) -> Option<Gd<RegExMatch>> {
    let project_settings = ProjectSettings::singleton();

    // Requires pattern string as well.
    if project_settings.get_setting(PROJECT_SETTING_LOADER_MESSAGE_PATTERN).stringify().is_empty() {
        return None;
    }

    // 1. File regex.
    let file_regex = project_settings.get_setting(PROJECT_SETTING_LOADER_PATTERN_BY_FILE_REGEX).stringify();
    if !file_regex.is_empty() {
        let file_name = path.file_name()?;
        let file_name = GString::from(file_name.to_owned().into_string().unwrap());
        let file_regex = RegEx::create_from_string(&file_regex).unwrap();
        if let Some(regex_match) = file_regex.search(&file_name) {
            return Some(regex_match);
        }
    }

    // 2. Folder regex.
    let folder_regex = project_settings.get_setting(PROJECT_SETTING_LOADER_PATTERN_BY_FOLDER_REGEX).stringify();
    if !folder_regex.is_empty() {
        let folder_regex = RegEx::create_from_string(&folder_regex).unwrap();
        for folder in path.iter().rev() {
            let folder = folder.to_owned().into_string().unwrap();
            if folder == path::MAIN_SEPARATOR_STR {
                continue;
            }

            if let Some(regex_match) = folder_regex.search(&folder) {
                return Some(regex_match);
            }
        }
    }

    // Unable to find a message pattern.
    None
}

pub fn compute_locale(path: &PathBuf) -> Option<String> {
    let project_settings = ProjectSettings::singleton();

    // 1. File regex.
    let file_regex = project_settings.get_setting(PROJECT_SETTING_LOADER_LOCALE_BY_FILE_REGEX).stringify();
    if !file_regex.is_empty() {
        let file_name = path.file_name()?;
        let file_name = GString::from(file_name.to_owned().into_string().unwrap());
        // Force regex to be case insensitive.
        let file_regex = "(?i)".to_owned() + &file_regex.to_string();
        let file_regex = RegEx::create_from_string(&file_regex).unwrap();
        if let Some(regex_match) = file_regex.search(&file_name) {
            let locale = get_single_regex_match(regex_match, PROJECT_SETTING_LOADER_LOCALE_BY_FILE_REGEX).to_string();
            if is_valid_locale(&locale) {
                return Some(locale);
            }
        }
    }

    // 2. Folder regex.
    let folder_regex = project_settings.get_setting(PROJECT_SETTING_LOADER_LOCALE_BY_FOLDER_REGEX).stringify();
    if !folder_regex.is_empty() {
        // Force regex to be case insensitive.
        let folder_regex = "(?i)".to_owned() + &folder_regex.to_string();
        let folder_regex = RegEx::create_from_string(&folder_regex).unwrap();
        for folder in path.iter().rev() {
            let folder = folder.to_owned().into_string().unwrap();
            if folder == path::MAIN_SEPARATOR_STR {
                continue;
            }

            if let Some(regex_match) = folder_regex.search(&folder) {
                let locale = get_single_regex_match(regex_match, PROJECT_SETTING_LOADER_LOCALE_BY_FOLDER_REGEX).to_string();
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
