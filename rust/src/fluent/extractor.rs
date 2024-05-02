use godot::prelude::*;

use super::MessageGeneration;

// There's currently no way to create classes that inherit from others (upstream https://github.com/godot-rust/gdext/issues/426)
// So for now we create a trait but don't register or use it for the public API. A plugin system can always be retrofitted here later.
pub trait FluentTranslationParser {
    fn get_recognized_extensions(&self) -> Vec<GString>;
    fn extract_messages(&self, path: &GString) -> MessageGeneration;
}
