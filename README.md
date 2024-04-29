# Godot Fluent Translation

[Fluent](https://projectfluent.org/) Translation for Godot via a [Rust](https://github.com/projectfluent/fluent-rs) [GDExtension](https://github.com/godot-rust/gdext).

![Demo project](./docs/demo.gif)

## Sample

The API might change in the near future!

```gd
func _init():
    # Currently, there is no resource importer for FTL files.
    var translation = TranslationFluent.new()

    # "locale" is what Godot's language is set to, while the add_bundle_from_text parameter
    # is what fluent uses as language identifier. This might be unified in the future.
    translation.locale = "en"

    # Godot automatically converts spaces to tabs for multi-line strings, but tabs are invalid in
    # FTL syntax. So convert tabs to four spaces.
    translation.add_bundle_from_text(&"en", """
-term = email
HELLO =
    { $unreadEmails ->
        [one] You have one unread { -term }.
       *[other] You have { $unreadEmails } unread { -term }s.
    }
    .meta = An attr.
""".replace("\t", "    "))

    # Register the translation.
    TranslationServer.add_translation(translation)

    # Repeat this process for all of your languages...


func _notification(what: int) -> void:
    if what == NOTIFICATION_TRANSLATION_CHANGED:
        # atr and tr have a new "args" Dictionary parameter which is used to fill $variables.
        $Label.text = atr("HELLO", { "unreadEmails": $SpinBox.value })
        # The context field is used to retrieve .attributes of a message.
        $Label2.text = atr("HELLO", {}, "meta")
```

## Setup

* Needs Godot 4.1 or later - I used 4.3 master build based on commit [`6118592c6d`](https://github.com/godotengine/godot/commit/6118592c6d88350d01f74faff6fd49754f84a7d0)
    * Due to a change in the Translation API, you must create a custom build of the engine with a patch applied. See instructions below.
* You must have Rust set up, see [this guide](https://godot-rust.github.io/book/intro/setup.html) and follow the LLVM instructions as well.

## Installation

1. Apply the following patch to a custom Godot build: https://github.com/RedMser/godot/pull/1
2. Clone this repository.
3. Modify `rust/.cargo/config.toml` to point to your custom Godot build path.
4. Build the rust library via `cargo build`
5. The demo project should now run successfully.

## About this Project

This is not a production-ready project and will likely have large breaking changes changes to come. Please consider this if you intend on using this library.

Due to Godot needing breaking API changes to have this extension work, it is unlikely to become easily usable out-of-the-box. Not much I can do besides wait for another major release that would accept this breaking change.

Any help in continuing development for this library is welcome!
