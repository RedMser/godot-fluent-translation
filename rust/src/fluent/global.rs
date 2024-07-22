use godot::prelude::*;
use godot::classes::ResourceLoader;

use super::ResourceFormatLoaderFluent;

/// Singleton for handling Fluent Translation. For internal use only.
#[derive(GodotClass)]
#[class(base=Object, init)]
pub struct FluentI18nSingleton {
    loader: Gd<ResourceFormatLoaderFluent>,
}

impl FluentI18nSingleton {
    pub(crate) const SINGLETON_NAME: &'static str = "FluentI18nSingleton";

    pub(crate) fn register(&self) {
        ResourceLoader::singleton().add_resource_format_loader(self.loader.clone());
    }

    pub(crate) fn unregister(&self) {
        ResourceLoader::singleton().remove_resource_format_loader(self.loader.clone());
    }
}
