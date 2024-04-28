use godot::prelude::*;

mod fluent;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
