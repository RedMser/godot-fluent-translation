use godot::{classes::{EditorExportPlatform, EditorExportPlugin, IEditorExportPlugin}, prelude::*};
use constcat::concat as constcat;

use super::strip_comments;

const EXPORT_OPTION_PREFIX: &str = "fluent/";
const EXPORT_OPTION_STRIP_COMMENTS: &str = constcat!(EXPORT_OPTION_PREFIX, "strip_comments");

#[derive(GodotClass)]
#[class(base=EditorExportPlugin)]
pub struct FluentExportPlugin {
    base: Base<EditorExportPlugin>,
}

#[godot_api]
impl IEditorExportPlugin for FluentExportPlugin {
    fn init(base: Base<EditorExportPlugin>) -> Self {
        Self {
            base,
        }
    }

    fn get_export_options(&self, _platform: Gd<EditorExportPlatform>) -> Array<Dictionary> {
        array![dict! {
            "option": dict! {
                "name": GString::from(EXPORT_OPTION_STRIP_COMMENTS),
                "type": VariantType::BOOL,
            },
            "default_value": Variant::from(true),
        }]
    }

    fn export_file(&mut self, path: GString, _type: GString, _features: PackedStringArray) {
        if !path.to_string().to_lowercase().ends_with("ftl") {
            return;
        }

        if self.base().get_option(StringName::from(EXPORT_OPTION_STRIP_COMMENTS)).booleanize() {
            // Strip comments from file
            let contents = strip_comments(path.clone());
            let binary = PackedByteArray::from_iter(contents.bytes());

            self.base_mut().skip();
            self.base_mut().add_file(path, binary, false);
        }
    }
}
