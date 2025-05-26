use std::path::PathBuf;

use godot::prelude::*;
use godot::classes::{FileAccess, IResourceFormatLoader, ProjectSettings, RegEx, ResourceFormatLoader};
use godot::global::Error as GdErr;

use super::{locale::{compute_locale, compute_message_pattern}, project_settings::*, TranslationFluent, FluentI18nSingleton};

/// Loads Fluent Translation List (FTL) files.
/// 
/// This loader is already registered and does usually not need to be manually used. Use [method @GDScript.load] on a `.ftl` file instead.
#[derive(GodotClass)]
#[class(base=ResourceFormatLoader)]
pub struct ResourceFormatLoaderFluent {
    base: Base<ResourceFormatLoader>,
}

#[godot_api]
impl IResourceFormatLoader for ResourceFormatLoaderFluent {
    fn init(base: Base<ResourceFormatLoader>) -> Self {
        Self {
            base,
        }
    }

    fn get_recognized_extensions(&self) -> PackedStringArray {
        PackedStringArray::from(&["ftl".to_godot()])
    }

    fn handles_type(&self, type_name: StringName) -> bool {
        type_name == "Translation".into()
    }

    fn get_resource_type(&self, path: GString) -> GString {
        if path.to_string().to_lowercase().ends_with("ftl") {
            "Translation"
        } else {
            ""
        }.into()
    }

    fn load(&self, path: GString, _original_path: GString, _use_sub_threads: bool, _cache_mode: i32) -> Variant {
        let path_buf: String = path.clone().into();
        let path_buf = PathBuf::from(path_buf);
        let locale = compute_locale(&path_buf);
        if locale.is_none() {
            return GdErr::ERR_INVALID_PARAMETER.ord().to_variant();
        }

        let text = FileAccess::get_file_as_string(&path);
        let err = FileAccess::get_open_error();
        if err != GdErr::OK {
            return err.ord().to_variant();
        }

        {
            let singleton = FluentI18nSingleton::singleton();
            let singleton = singleton.bind();
            godot_print!("we loading and my plugin says {}", singleton.plugin_get_foo());
        }

        let mut translation = TranslationFluent::new_gd();
        translation.bind_mut().base_mut().set_locale(&locale.unwrap());

        let pattern_match = compute_message_pattern(&path_buf);
        if let Some(pattern_match) = pattern_match {
            let mut pattern_target = String::from_godot(ProjectSettings::singleton().get_setting(PROJECT_SETTING_LOADER_MESSAGE_PATTERN).stringify());
            for group_index in 0..=pattern_match.get_group_count() {
                let group_value = pattern_match.get_string_ex().name(&group_index.to_variant()).done();
                pattern_target = pattern_target.replace(&format!("{{${}}}", group_index), &group_value.to_string());
            }

            let pattern_target = GString::from(pattern_target);
            let pattern_regex = RegEx::create_from_string(&pattern_target).unwrap();
            if pattern_regex.get_group_count() != 1 {
                godot_warn!(
                    "Expected {} to have exactly one capture group, but got {} instead.\nIgnoring message pattern!", 
                    PROJECT_SETTING_LOADER_MESSAGE_PATTERN, pattern_regex.get_group_count()
                );
            } else {
                translation.bind_mut().set_message_pattern(pattern_target);
            }
        }

        let err = translation.bind_mut().append_from_text(text.to_string());
        if err != GdErr::OK {
            return err.ord().to_variant();
        }

        translation.to_variant()
    }
}
