use godot::{engine::{EditorInterface, EditorSyntaxHighlighter, ResourceLoader}, prelude::*};

use super::{EditorFluentSyntaxHighlighter, ResourceFormatLoaderFluent};

#[derive(GodotClass)]
#[class(base=Object, init)]
pub struct FluentI18nSingleton {
    loader: Gd<ResourceFormatLoaderFluent>,
    syntax_highlighter: Option<Gd<EditorFluentSyntaxHighlighter>>,
}

#[godot_api]
impl FluentI18nSingleton {
    pub(crate) const SINGLETON_NAME: &'static str = "FluentI18nSingleton";

    pub(crate) fn register_scene(&self) {
        ResourceLoader::singleton().add_resource_format_loader(self.loader.clone().upcast());
    }

    // TODO: need to also register the .ftl extension as something the script editor can open
    #[func]
    pub(crate) fn register_editor(&mut self) {
        self.syntax_highlighter = Some(EditorFluentSyntaxHighlighter::new_gd());
        // TODO: this is too early! wait until editor is ready...
        EditorInterface::singleton().get_script_editor().map(|mut script_editor| {
            script_editor.register_syntax_highlighter(self.syntax_highlighter.clone().unwrap().clone().upcast::<EditorSyntaxHighlighter>()) }
        );
    }

    pub(crate) fn unregister_scene(&self) {
        ResourceLoader::singleton().remove_resource_format_loader(self.loader.clone().upcast());
    }

    pub(crate) fn unregister_editor(&self) {
        EditorInterface::singleton().get_script_editor().map(|mut script_editor| {
            script_editor.unregister_syntax_highlighter(self.syntax_highlighter.clone().unwrap().clone().upcast::<EditorSyntaxHighlighter>()) }
        );
    }
}