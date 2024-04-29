use godot::{engine::ResourceLoader, prelude::*};

use super::importer::ResourceFormatLoaderFluent;

#[derive(GodotClass)]
#[class(base=Object, init)]
pub struct FluentI18nSingleton {
    loader: Gd<ResourceFormatLoaderFluent>,
}

impl FluentI18nSingleton {
    pub(crate) const SINGLETON_NAME: &'static str = "FluentI18nSingleton";

    pub(crate) fn register(&self) {
        ResourceLoader::singleton().add_resource_format_loader(self.loader.clone().upcast());
    }

    pub(crate) fn unregister(&self) {
        ResourceLoader::singleton().remove_resource_format_loader(self.loader.clone().upcast());
    }
}