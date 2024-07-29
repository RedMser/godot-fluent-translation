use godot::global::PropertyHint;
use godot::prelude::*;
use godot::classes::ProjectSettings;
use constcat::concat as constcat;

const PROJECT_SETTING_PREFIX: &str = "internationalization/fluent/";
pub(crate) const PROJECT_SETTING_FALLBACK_LOCALE: &str = "internationalization/locale/fallback";
pub(crate) const PROJECT_SETTING_UNICODE_ISOLATION: &str = constcat!(PROJECT_SETTING_PREFIX, "use_unicode_isolation");
pub(crate) const PROJECT_SETTING_PARSE_ARGS_IN_MESSAGE: &str = constcat!(PROJECT_SETTING_PREFIX, "parse_args_in_message");
pub(crate) const PROJECT_SETTING_LOADER_LOCALE_BY_FOLDER_REGEX: &str = constcat!(PROJECT_SETTING_PREFIX, "loader/locale_by_folder_regex");
pub(crate) const PROJECT_SETTING_LOADER_LOCALE_BY_FILE_REGEX: &str = constcat!(PROJECT_SETTING_PREFIX, "loader/locale_by_file_regex");
pub(crate) const PROJECT_SETTING_LOADER_PATTERN_BY_FOLDER_REGEX: &str = constcat!(PROJECT_SETTING_PREFIX, "loader/pattern_by_folder_regex");
pub(crate) const PROJECT_SETTING_LOADER_PATTERN_BY_FILE_REGEX: &str = constcat!(PROJECT_SETTING_PREFIX, "loader/pattern_by_file_regex");
pub(crate) const PROJECT_SETTING_LOADER_MESSAGE_PATTERN: &str = constcat!(PROJECT_SETTING_PREFIX, "loader/message_pattern");
pub(crate) const PROJECT_SETTING_GENERATOR_LOCALES: &str = constcat!(PROJECT_SETTING_PREFIX, "generator/locales");
pub(crate) const PROJECT_SETTING_GENERATOR_PATTERNS: &str = constcat!(PROJECT_SETTING_PREFIX, "generator/file_patterns");
pub(crate) const PROJECT_SETTING_GENERATOR_INVALID_MESSAGE_HANDLING: &str = constcat!(PROJECT_SETTING_PREFIX, "generator/invalid_message_handling");

pub(crate) const INVALID_MESSAGE_HANDLING_SKIP: i32 = 0;
pub(crate) const INVALID_MESSAGE_HANDLING_CONVERT_TO_VALID: i32 = 1;

pub fn register() {
    register_setting(PROJECT_SETTING_UNICODE_ISOLATION.to_string(), false.to_variant());
    // Default to true for default builds (no args parameter), false for forked builds.
    register_setting(PROJECT_SETTING_PARSE_ARGS_IN_MESSAGE.to_string(), cfg!(not(feature = "forked-godot")).to_variant());
    register_setting(PROJECT_SETTING_LOADER_LOCALE_BY_FOLDER_REGEX.to_string(), "^.+$".to_variant());
    register_setting(PROJECT_SETTING_LOADER_LOCALE_BY_FILE_REGEX.to_string(), "\\.(.+?)\\.ftl$".to_variant());
    register_setting(PROJECT_SETTING_LOADER_PATTERN_BY_FOLDER_REGEX.to_string(), "".to_variant());
    register_setting(PROJECT_SETTING_LOADER_PATTERN_BY_FILE_REGEX.to_string(), "".to_variant());
    register_setting(PROJECT_SETTING_LOADER_MESSAGE_PATTERN.to_string(), "".to_variant());
    register_setting_hint(PROJECT_SETTING_GENERATOR_LOCALES.to_string(), PackedStringArray::new().to_variant(), PropertyHint::NONE, format!("{}/{}:", VariantType::STRING.ord(), PropertyHint::LOCALE_ID.ord()));
    register_setting(PROJECT_SETTING_GENERATOR_PATTERNS.to_string(), Dictionary::new().to_variant());
    register_setting_hint(PROJECT_SETTING_GENERATOR_INVALID_MESSAGE_HANDLING.to_string(), 0.to_variant(), PropertyHint::ENUM, "Skip message,Convert to valid".into());
}

fn register_setting(name: String, value: Variant) {
    register_setting_hint(name, value, PropertyHint::NONE, String::new());
}

fn register_setting_hint(name: String, value: Variant, hint: PropertyHint, hint_string: String) {
    let mut project_settings = ProjectSettings::singleton();
    
    if !project_settings.has_setting(name.clone().into()) {
        project_settings.set_setting(name.clone().into(), value.clone());
    }

    let mut property_info = Dictionary::new();
    property_info.set("name", GString::from(name.clone()));
    property_info.set("type", value.get_type());
    property_info.set("hint", hint);
    property_info.set("hint_string", GString::from(hint_string));

    project_settings.add_property_info(property_info);
    project_settings.set_initial_value(GString::from(name), value);
}
