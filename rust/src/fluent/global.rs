use godot::prelude::*;
use godot::classes::{Engine, GDScript, ResourceLoader};

use super::ResourceFormatLoaderFluent;

/// Singleton for handling Fluent Translation. For internal use only.
#[derive(GodotClass)]
#[class(base=Object, init)]
pub struct FluentI18nSingleton {
    plugin: Option<Gd<Object>>,
    loader: Option<Gd<ResourceFormatLoaderFluent>>,
}

impl FluentI18nSingleton {
    pub(crate) const SINGLETON_NAME: &'static str = "FluentI18nSingleton";

    pub(crate) fn singleton() -> Gd<Self> {
        let engine = Engine::singleton();
        let singleton = engine
            .get_singleton(Self::SINGLETON_NAME)
            .expect(&format!("Tried accessing {} before registration.", Self::SINGLETON_NAME));
        singleton.cast::<Self>()
    }

    pub(crate) fn plugin_get_foo(&self) -> Variant {
        self.plugin.clone().map_or(Variant::nil(), |mut plugin| plugin.call("foo", &[]))
    }

    pub(crate) fn register(&mut self) {
        // HACK: Resource format loader crashes editor on startup, see https://github.com/godot-rust/gdext/issues/597
        if !Engine::singleton().is_editor_hint() {
            self.loader = Some(ResourceFormatLoaderFluent::new_gd());
            ResourceLoader::singleton().add_resource_format_loader(&self.loader.clone().unwrap());
        }

        let script = ResourceLoader::singleton().load("res://fluent_plugin.gd");
        if let Some(script) = script {
            match script.try_cast::<GDScript>() {
                Ok(mut script) => {
                    match script.try_instantiate(&[]) {
                        Ok(plugin) => {
                            match plugin.try_to::<Gd<Object>>() {
                                Ok(plugin) => {
                                    self.plugin = Some(plugin);
                                },
                                Err(_) => {
                                    godot_error!("Fluent plugin could not be instantiated, Variant typed {:?}.", plugin.get_type());
                                }
                            }
                        },
                        Err(err) => {
                            godot_error!("Fluent plugin could not be instantiated: {err}");
                        },
                    }
                },
                Err(_) => {
                    godot_error!("Fluent plugin could not be casted to GDScript.");
                }
            }
        }
    }

    pub(crate) fn unregister(&mut self) {
        if let Some(loader) = self.loader.take() {
            ResourceLoader::singleton().remove_resource_format_loader(&loader);
        }
        if let Some(plugin) = self.plugin.take() {
            plugin.free();
        }
    }
}
