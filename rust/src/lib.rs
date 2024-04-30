use crate::fluent::FluentI18nSingleton;
use fluent::project_settings;
use godot::{engine::Engine, prelude::*};

pub mod fluent;

struct FluentI18n;

#[gdextension]
unsafe impl ExtensionLibrary for FluentI18n {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            project_settings::register();

            let singleton = FluentI18nSingleton::new_alloc();
            singleton.bind().register_scene();

            Engine::singleton()
                .register_singleton(FluentI18nSingleton::SINGLETON_NAME.into(), singleton.upcast());
        } else if level == InitLevel::Editor {
            let engine = Engine::singleton();

            let singleton = engine
                .get_singleton(FluentI18nSingleton::SINGLETON_NAME.into())
                .unwrap();

            singleton
                .clone()
                .cast::<FluentI18nSingleton>()
                .bind_mut()
                .register_editor();
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            let mut engine = Engine::singleton();

            let singleton = engine
                .get_singleton(FluentI18nSingleton::SINGLETON_NAME.into())
                .unwrap();

            engine.unregister_singleton(FluentI18nSingleton::SINGLETON_NAME.into());

            singleton
                .clone()
                .cast::<FluentI18nSingleton>()
                .bind()
                .unregister_scene();

            singleton.free();
        } else if level == InitLevel::Editor {
            let engine = Engine::singleton();

            let singleton = engine
                .get_singleton(FluentI18nSingleton::SINGLETON_NAME.into())
                .unwrap();
            
            singleton
                .clone()
                .cast::<FluentI18nSingleton>()
                .bind()
                .unregister_editor();
        }
    }
}
