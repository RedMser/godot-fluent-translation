use crate::fluent::FluentI18nSingleton;
use fluent::project_settings;
use godot::prelude::*;
use godot::classes::Engine;

pub mod fluent;
pub(crate) mod hacks;
pub(crate) mod utils;

struct FluentI18n;

#[cfg(not(any(feature = "default-godot", feature = "custom-godot", feature = "forked-godot")))]
compile_error!("You must enable one of features the `default-godot` | `custom-godot` | `forked-godot` to compile.\nSee the README to help you decide which one to use.");

#[cfg(any(
    all(feature = "default-godot", feature = "custom-godot"),
    all(feature = "forked-godot", feature = "custom-godot"),
    all(feature = "default-godot", feature = "forked-godot"),
))]
compile_error!("You may only enable one of the features `default-godot` | `custom-godot` | `forked-godot` to compile.\nSee the README to help you decide which one to use.");

#[gdextension]
unsafe impl ExtensionLibrary for FluentI18n {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            project_settings::register();

            let singleton = FluentI18nSingleton::new_alloc();
            singleton.bind().register();

            Engine::singleton()
                .register_singleton(FluentI18nSingleton::SINGLETON_NAME, &singleton);
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            let mut engine = Engine::singleton();

            let singleton = engine
                .get_singleton(FluentI18nSingleton::SINGLETON_NAME)
                .unwrap();

            engine.unregister_singleton(FluentI18nSingleton::SINGLETON_NAME);

            singleton
                .clone()
                .cast::<FluentI18nSingleton>()
                .bind()
                .unregister();

            singleton.free();
        }
    }
}
