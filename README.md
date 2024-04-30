# Godot Fluent Translation

[Fluent](https://projectfluent.org/) Translation for Godot via a [Rust](https://github.com/projectfluent/fluent-rs) [GDExtension](https://github.com/godot-rust/gdext).

![Demo project](./docs/demo.gif)

## Sample

```gd
func _init():
    # Four ways to load FTL translations:
    # 1. [Project Settings -> Localization -> Translations] and add a .ftl file there.

    # 2. load(path) with locale in file name (Portuguese).
    var tr_filename = load("res://test.pt_PT.ftl")

    # 3. load(path) with locale in folder name (German).
    var tr_foldername = load("res://de/german-test.ftl")

    # 4. Manually create a TranslationFluent resource.
    var tr_inline = TranslationFluent.new()
    # Ensure that you fill the locale before adding any contents (English).
    tr_inline.locale = "en"

    # Godot automatically converts spaces to tabs for multi-line strings, but tabs are invalid in
    # FTL syntax. So convert tabs to four spaces. Returns an error that you should handle.
    var err_inline = tr_inline.add_bundle_from_text("""
-term = email
HELLO =
    { $unreadEmails ->
        [one] You have one unread { -term }.
       *[other] You have { $unreadEmails } unread { -term }s.
    }
    .meta = An attr.
""".replace("\t", "    "))

    # Register via TranslationServer.
    TranslationServer.add_translation(tr_filename)
    TranslationServer.add_translation(tr_foldername)
    TranslationServer.add_translation(tr_inline)


func _notification(what: int) -> void:
    if what == NOTIFICATION_TRANSLATION_CHANGED:
        # atr and tr have a new "args" Dictionary parameter which is used to fill $variables.
        $Label.text = atr("HELLO", { "unreadEmails": $SpinBox.value })
        # The context field is used to retrieve .attributes of a message.
        $Label2.text = atr("HELLO", {}, "meta")
```

## Project Settings

* `internationalization/fluent/locale_by_file_regex`: If specified, file name is first checked for locale via regex. Can contain a capture group which matches a possible locale. Always case-insensitive.
* `internationalization/fluent/locale_by_folder_regex`: If specified, the folder hierarchy is secondly traversed to check for locale via regex. Can contain a capture group which matches a possible locale. Always case-insensitive.
* `internationalization/locale/fallback`: Fallback locale is used when the selected language does not have a date/time/number formatter available.

If you don't see some of these settings, make sure you have Advanced Settings showing.

## Setup

* Needs Godot 4.1 or later - I used 4.3 master build based on commit [`d282e4f0e6`](https://github.com/godotengine/godot/commit/d282e4f0e6b6ebcf3bd6e05cd62f2a8fe1f9a238)
    * Due to a change in the Translation API, you must create a custom build of the engine with a patch applied. See instructions below.
* You must have Rust set up, see [this guide](https://godot-rust.github.io/book/intro/setup.html) and follow the LLVM instructions as well.

## Installation

1. Apply the following patch to a custom Godot build: https://github.com/RedMser/godot/pull/1
2. Clone this repository.
3. Modify `rust/.cargo/config.toml` to point to your custom Godot build path.
4. Build the rust library via `cargo build`
5. The demo project should now run successfully.

## About this Project

This is not a production-ready project and will likely have breaking API changes without warning. Please consider this if you intend on using this library.

Due to Godot needing breaking API changes to have this extension work, it is unlikely to become easily usable out-of-the-box. Not much I can do besides wait for another major release that would accept this breaking change.

Any help in continuing development for this library is welcome!
