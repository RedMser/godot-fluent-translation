use godot::{classes::{EditorPlugin, IEditorPlugin}, prelude::*};

use super::FluentExportPlugin;

#[derive(GodotClass)]
#[class(tool, editor_plugin, init, base=EditorPlugin)]
pub struct FluentEditorPlugin {
    export_plugin: Option<Gd<FluentExportPlugin>>,
    base: Base<EditorPlugin>,
}

#[godot_api]
impl IEditorPlugin for FluentEditorPlugin {
    fn enter_tree(&mut self) {
        let export_plugin = FluentExportPlugin::new_gd();
        self.export_plugin = Some(export_plugin.clone());
        self.base_mut().add_export_plugin(export_plugin);
    }

    fn exit_tree(&mut self) {
        let export_plugin = self.export_plugin.take();
        self.base_mut().remove_export_plugin(export_plugin);
        self.export_plugin = None;
    }
}
