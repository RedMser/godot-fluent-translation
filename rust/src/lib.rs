use godot::prelude::*;

pub mod fluent;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
