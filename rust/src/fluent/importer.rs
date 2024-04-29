use std::path::PathBuf;

use godot::{engine::{FileAccess, IResourceFormatLoader, ResourceFormatLoader}, prelude::*};
use godot::engine::global::Error as GdErr;

use super::{locale::compute_locale, TranslationFluent};

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
        PackedStringArray::from(&["ftl".into_godot()])
    }

    fn handles_type(&self, type_name: StringName) -> bool {
        type_name == "Translation".into()
    }

    fn get_resource_type(&self, path: GString) -> GString {
        if path.to_string().to_lowercase().ends_with("ftl") {
            "Translation".into()
        } else {
            "".into()
        }
    }

    fn load(&self, path: GString, _original_path: GString, _use_sub_threads: bool, _cache_mode: i32) -> Variant {
        let path_buf: String = path.clone().into();
        let path_buf = PathBuf::from(path_buf);
        let locale = compute_locale(&path_buf);
        godot_warn!("{:?}", locale);
        if locale.is_none() {
            return GdErr::ERR_INVALID_PARAMETER.ord().to_variant();
        }

        let text = FileAccess::get_file_as_string(path);
        let err = FileAccess::get_open_error();
        if err != GdErr::OK {
            return err.ord().to_variant();
        }

        let mut translation = TranslationFluent::new_gd();
        translation.bind_mut().base_mut().set_locale(locale.unwrap().into());
        let err = translation.bind_mut().add_bundle_from_text(text.to_string());
        if err != GdErr::OK {
            return err.ord().to_variant();
        }
        translation.to_variant()
    }
}
