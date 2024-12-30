fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    godot_bindings::emit_godot_version_cfg();
}
