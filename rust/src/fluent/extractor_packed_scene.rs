use std::collections::{HashMap, HashSet};

use godot::engine::node::AutoTranslateMode;
use godot::engine::{ClassDb, RegEx};
use godot::engine::{resource_loader::CacheMode, ResourceLoader};
use godot::prelude::*;

use super::{FluentTranslationParser, MessageGeneration};

pub struct FluentPackedSceneTranslationParser {
    lookup_properties: HashSet<Gd<RegEx>>,
    exception_list: HashMap<StringName, HashSet<Gd<RegEx>>>,
}

/// Implementation is closely related to that of PackedSceneEditorTranslationParserPlugin.
/// Main difference is that it does not (yet) call parsers recursively.
impl FluentTranslationParser for FluentPackedSceneTranslationParser {
    fn get_recognized_extensions(&self) -> Vec<GString> {
        ResourceLoader::singleton().get_recognized_extensions_for_type("PackedScene".into()).to_vec()
    }

    fn extract_messages(&self, path: &GString) -> MessageGeneration {
        let class_db = ClassDb::singleton();

        let loaded_res = ResourceLoader::singleton().load_ex(path.clone())
            .type_hint("PackedScene".into())
            .cache_mode(CacheMode::REUSE)
            .done();
        if loaded_res.is_none() {
            godot_error!("Failed loading {path}.");
            return Default::default();
        }
        let loaded_res = loaded_res.unwrap().cast::<PackedScene>();
        let state = loaded_res.get_state().unwrap();

        let mut parsed_strings = Vec::<GString>::new();
        let mut atr_owners = Vec::<(NodePath, bool)>::new();
        let mut tabcontainer_paths = Vec::<GString>::new();
        for i in 0..state.get_node_count() {
            let mut node_type = state.get_node_type(i);
            let parent_path = state.get_node_path_ex(i).for_parent(true).done();

            // Handle instanced scenes.
            if node_type.is_empty() {
                let instance = state.get_node_instance(i);
                if let Some(instance) = instance {
                    let _state = instance.get_state().unwrap();
                    node_type = _state.get_node_type(0);
                }
            }

            // Find the `auto_translate_mode` property.
            let mut auto_translating = true;
            let mut auto_translate_mode_found = false;
            for j in 0..state.get_node_property_count(i) {
                if state.get_node_property_name(i, j) != "auto_translate_mode".into() {
                    continue;
                }

                auto_translate_mode_found = true;

                let idx_last = (atr_owners.len() as i64) - 1;
                if idx_last > 0 && !parent_path.to_string().starts_with(&GString::from(atr_owners[idx_last as usize].0.clone()).to_string()) {
                    // Switch to the previous auto translation owner this was nested in, if that was the case.
                    atr_owners.remove(idx_last as usize);
                }
                
                let auto_translate_mode = i32::from_variant(&state.get_node_property_value(i, j));
                if auto_translate_mode == AutoTranslateMode::DISABLED.ord() {
                    auto_translating = false;
                }

                atr_owners.push((state.get_node_path(i), auto_translating));

                break;
            }

            // If `auto_translate_mode` wasn't found, that means it is set to its default value (`AUTO_TRANSLATE_MODE_INHERIT`).
            if !auto_translate_mode_found {
                let idx_last = (atr_owners.len() as i64) - 1;
                if idx_last > 0 && atr_owners[idx_last as usize].0 == parent_path {
                    auto_translating = atr_owners[idx_last as usize].1;
                } else {
                    atr_owners.push((state.get_node_path(i), true));
                }
            }

            // Parse the names of children of `TabContainer`s, as they are used for tab titles.
            if !tabcontainer_paths.is_empty() {
                if !parent_path.to_string().starts_with(&GString::from(tabcontainer_paths[((tabcontainer_paths.len() as i64) - 1) as usize].clone()).to_string()) {
                    // Switch to the previous `TabContainer` this was nested in, if that was the case.
                    tabcontainer_paths.pop();
                }

                if auto_translating && !tabcontainer_paths.is_empty() && class_db.is_parent_class(node_type.clone(), "Control".into())
                    && GString::from(parent_path) == tabcontainer_paths[((tabcontainer_paths.len() as i64) - 1) as usize] {
                    parsed_strings.push(GString::from(state.get_node_name(i)));
                }
            }

            if !auto_translating {
                continue;
            }

            if node_type == "TabContainer".into() {
                tabcontainer_paths.push(GString::from(state.get_node_path(i)));
            }

            for j in 0..state.get_node_property_count(i) {
                let property_name = state.get_node_property_name(i, j);

                if !self.match_property(&property_name, &node_type) {
                    continue;
                }

                let property_value = state.get_node_property_value(i, j);
                /*
                if property_name == "script" && property_value.get_type() == Variant::OBJECT && !property_value.is_null() {
                    // Parse built-in script.
                    let s = property_value.cast::<Script>();
                    if !s.is_built_in() {
                        continue;
                    }

                    if let Some(extension) = s.get_language().get_extension() {
                        if EditorTranslationParser::get_singleton().can_parse(&extension) {
                            let (temp, ids_context_plural) = EditorTranslationParser::get_singleton()
                                .get_parser(&extension)
                                .parse_file(s.get_path());
                            parsed_strings.extend(temp);
                            r_ids_ctx_plural.extend(ids_context_plural);
                        }
                    }
                } else */ if node_type == "FileDialog".into() && property_name == "filters".into() {
                    // Extract FileDialog's filters property with values in format "*.png ; PNG Images","*.gd ; GDScript Files".
                    let str_values = PackedStringArray::from_variant(&property_value);
                    for str_value in str_values.to_vec() {
                        let str_value = str_value.to_string();
                        let desc = str_value.split_once(';').unwrap_or(("", "")).1.trim();
                        if !desc.is_empty() {
                            parsed_strings.push(GString::from(desc));
                        }
                    }
                } else if property_value.get_type() == VariantType::String {
                    // Prevent reading text containing only spaces.
                    let str_value = GString::from_variant(&property_value);
                    if !str_value.to_string().trim().is_empty() {
                        parsed_strings.push(str_value.into());
                    }
                }
            }
        }

        // Assume that ids = messages.
        return parsed_strings
            .into_iter()
            .map(|string| (string.to_string(), string.to_string()))
            .collect();
    }
}

impl FluentPackedSceneTranslationParser {
    pub fn init() -> Self {
        let lookup_properties = ["^text$", "^.+_text$", "^popup/.+/text$", "^title$", "^filters$", /* "^script$", */]
            .map(|str| RegEx::create_from_string(str.into()).unwrap())
            .into();
        let exception_list = [
            ("LineEdit", ["^text$"]),
            ("TextEdit", ["^text$"]),
            ("CodeEdit", ["^text$"]),
        ].map(|(typename, strs)|
            (StringName::from(typename), HashSet::from(
                strs.map(|str| RegEx::create_from_string(str.into()).unwrap())
            ))
        )
        .into();

        Self {
            lookup_properties,
            exception_list,
        }
    }

    fn match_property(&self, property_name: &StringName, node_type: &StringName) -> bool {
        let class_db = ClassDb::singleton();
        
        for (exception_node_type, exception_properties) in &self.exception_list {
            if class_db.is_parent_class(node_type.clone(), exception_node_type.clone()) {
                if exception_properties.iter().any(|exception_property| Self::matches(GString::from(property_name), exception_property.clone())) {
                    return false;
                }
            }
        }
        if self.lookup_properties.iter().any(|lookup_property| Self::matches(GString::from(property_name), lookup_property.clone())) {
            return true;
        }
        return false;
    }

    fn matches(string: GString, pattern: Gd<RegEx>) -> bool {
        pattern.search(string).is_some()
    }
}
