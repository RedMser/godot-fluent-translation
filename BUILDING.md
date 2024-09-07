# Building Fluent Translation

## Default vs Custom vs Forked

Since Godot's translation system was not made with some of Fluent's features in mind, there are some choices you can make when building yourself.

This extension works with different versions of Godot, and deciding which one to use is an important first step to take.

### Default

> [!CAUTION]
> As of writing this, the latest stable version provided by godot-rust is too old and does not compile. For the time being, please use **Custom** or **Forked** instead.

This uses the latest stable version of Godot, as provided by godot-rust.

As explained in the [Available Versions](./README.md#default) section of the readme, certain features will be unavailable when using this option.

### Custom

This allows you to choose a custom version of Godot for building. This has the same drawbacks as **Default**, but allows using a newer or older version of Godot.

If you are compiling Godot engine yourself, you might want to consider **Forked** instead.

### Forked

For this option, you must use a specialized custom build of the Godot engine, which has [the changes of the fluent-translation branch](https://github.com/RedMser/godot/tree/fluent-translation) applied to it.

As explained in the [Available Versions](./README.md#forked) section of the readme, this option provides the best engine integration, and should be preferred whenever you are already doing a custom Godot engine build anyway.

## Version Compatibility

You must use [Godot v4.3](https://github.com/godotengine/godot/releases/tag/4.3-stable) or newer.

The GDExtension uses following APIs only available in this version:

- `FileAccess::resize()` (could be worked around by closing and reopening with `WRITE` perms).
- `AutoTranslateMode` (since it occurs in code which was ["copied" from Godot](https://github.com/godotengine/godot/blob/master/editor/plugins/packed_scene_translation_parser_plugin.cpp), an alternative implementation could use the old version of that code as reference).

If any interest for older Godot versions exist, consider tackling these compatibility issues first and then find out how low we can bring the minimum compatible version from this point onwards.

## Platform Compatibility

| Platform | Fork Build | GDExtension Build | Both Tested |
|----------|------------|-------------------|-------------|
| Windows  | ✅          | ✅                 | ✅           |
| macOS    | ✅          | ❌                 | ❌           |
| Linux    | ✅          | ✅                 | ❌           |
| iOS      | ✅          | ❌                 | ❌           |
| Android  | ✅          | ❌                 | ❌           |
| Web      | ✅          | ❌                 | ❌           |

A platform has a **Fork Build** if a [forked Godot release](https://github.com/RedMser/godot/releases) exists for it.
A platform has a **GDExtension Build** if the [GDExtension release CI/CD](https://github.com/RedMser/godot-fluent-translation/releases) targets this platform.
A platform is considered **Both Tested** if both the forked Godot release runs (editor or template), and the GDExtension works as expected.

[See here](https://github.com/RedMser/godot-fluent-translation/issues/22) if you want to add more checkmarks to this table!

## Build Instructions

* Decide which version of Godot you intend on working with (see above).
* Clone the [git repository](https://github.com/RedMser/godot-fluent-translation) somewhere.
* You must have Rust set up, see [this guide](https://godot-rust.github.io/book/intro/setup.html).
  * If you intend on using a custom or forked Godot build, you must follow the LLVM instructions as well.
* When using custom or forked: Open `rust/.cargo/config.toml` and ensure the `GODOT4_BIN` path inside points to your Godot executable of choice.
* Open a terminal and enter `cd rust`
* Build the project
  * Default: `cargo build --features default-godot`
  * Custom: `cargo build --features custom-godot`
  * Forked: `cargo build --features forked-godot`
* The build should complete without errors and produce a library file inside `rust/target/debug` named e.g. `godot_fluent_translation.dll`.

You should now be able to launch the Godot editor using the same version that was used for building, and import the default or forked demo project found in the `godot` folder.
