use godot::prelude::*;
use godot::classes::{Engine, ResourceLoader};

use super::ResourceFormatLoaderFluent;

/// Singleton for handling Fluent Translation. For internal use only.
#[derive(GodotClass)]
#[class(base=Object, init)]
pub struct FluentI18nSingleton {
    loader: Option<Gd<ResourceFormatLoaderFluent>>,
}

impl FluentI18nSingleton {
    pub(crate) const SINGLETON_NAME: &'static str = "FluentI18nSingleton";

    pub(crate) fn register(&mut self) {
        // HACK: Resource format loader crashes editor on startup, see https://github.com/godot-rust/gdext/issues/597
        if !Engine::singleton().is_editor_hint() {
            self.loader = Some(ResourceFormatLoaderFluent::new_gd());
            ResourceLoader::singleton().add_resource_format_loader(&self.loader.clone().unwrap());
        }
    }

    pub(crate) fn unregister(&self) {
        if let Some(loader) = &self.loader {
            ResourceLoader::singleton().remove_resource_format_loader(loader);
        }
    }
}
