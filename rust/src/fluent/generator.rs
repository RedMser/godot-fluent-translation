use godot::{engine::{utilities::error_string, ProjectSettings, RegEx, RegExMatch}, prelude::*};
use itertools::Itertools;
use std::collections::HashMap;
use fluent_syntax::{ast, parser::parse};
use fluent_syntax::serializer::serialize;

use crate::utils::{create_or_open_file_for_read_write, get_files_recursive};
use godot::engine::global::Error as GdErr;

use super::project_settings::{INVALID_MESSAGE_HANDLING_SKIP, PROJECT_SETTING_GENERATOR_INVALID_MESSAGE_HANDLING, PROJECT_SETTING_GENERATOR_LOCALES, PROJECT_SETTING_GENERATOR_PATTERNS};

#[derive(GodotClass)]
#[class(no_init)]
pub struct FluentGenerator {
    locales: Vec<String>,
    file_patterns: HashMap<Gd<RegEx>, String>,
    invalid_message_handling: i32,
}

/// Uses a HashMap<id, msg> to disallow duplicate messages to be generated.
type MessageGeneration = HashMap<String, String>;

#[godot_api]
impl FluentGenerator {
    #[func]
    pub fn create() -> Gd<Self> {
        let project_settings = ProjectSettings::singleton();
        let locales = PackedStringArray::from_variant(&project_settings.get_setting(PROJECT_SETTING_GENERATOR_LOCALES.into()));
        let locales = locales.as_slice().into_iter().map(|s| s.to_string()).collect();
        let file_patterns = Dictionary::from_variant(&project_settings.get_setting(PROJECT_SETTING_GENERATOR_PATTERNS.into()));
        let file_patterns = file_patterns.iter_shared().map(|(k, v)| {
            let k = GString::from_variant(&k);
            let k = RegEx::create_from_string(k).unwrap();
            let v = GString::from_variant(&v).to_string();
            (k, v)
        }).collect();
        Gd::from_object(Self {
            locales,
            file_patterns,
            invalid_message_handling: i32::from_variant(&project_settings.get_setting(PROJECT_SETTING_GENERATOR_INVALID_MESSAGE_HANDLING.into())),
        })
    }

    #[func]
    pub fn generate(&self) -> () {
        // Collect source files and batched write operations.
        let files = self.get_matching_files();
        let mut generate_tasks = HashMap::<String, MessageGeneration>::new();
        for (source, pattern) in files {
            let targets = self.apply_pattern(source, pattern);
            for target in targets {
                let entry = generate_tasks.entry(target.clone());
                let mut messages = Self::get_messages(target).into_iter().filter_map(|(id, message)| {
                    if id.is_empty() {
                        return None;
                    }

                    let safe_id = Self::make_safe_identifier(id.clone());
                    if id != safe_id {
                        if self.invalid_message_handling == INVALID_MESSAGE_HANDLING_SKIP {
                            // Skip invalid message.
                            return None;
                        } else {
                            return Some((safe_id, message));
                        }
                    }
                    return Some((id, message));
                });
                entry.or_default().extend(&mut messages);
            }
        }

        // Do the writes.
        for (ftl, messages) in generate_tasks {
            Self::create_or_update_ftl(ftl, messages);
        }
    }

    fn get_messages(file: String) -> MessageGeneration {
        // TODO: extract messages from file (mainly tscn) - via plugin system similar to POT generator?
        [(format!("message for {}", file), format!("message for {}", file))].into()
    }

    fn get_matching_files(&self) -> Vec<(Gd<RegExMatch>, String)> {
        get_files_recursive("res://".into())
            .into_iter()
            .filter_map(|str| {
                // Check all patterns until the first matches (returns Some(RegExMatch)).
                self.file_patterns.iter().find_map(|(regex, pattern)| {
                    // None gets discarded, Some is remapped to include the pattern.
                    regex.search(str.clone())
                        .map(|regex_match| (regex_match, pattern.to_owned()))
                })
            })
            .collect()
    }

    fn apply_pattern(&self, source_match: Gd<RegExMatch>, pattern: String) -> Vec<String> {
        self.locales
            .iter()
            .map(|locale| {
                let mut pattern = pattern.replace("{$locale}", &locale);
                for group_index in 0..=source_match.get_group_count() {
                    let group_value = source_match.get_string_ex().name(group_index.to_variant()).done();
                    pattern = pattern.replace(&format!("{{${}}}", group_index), &group_value.to_string());
                }
                pattern
            })
            .unique()
            .collect()
    }

    fn create_or_update_ftl(path: String, messages: MessageGeneration) -> () {
        // Load existing or create new FTL file.
        let fa = create_or_open_file_for_read_write(path.clone().into());
        if fa.is_err() {
            godot_error!("Unable to open file {} for writing: {}", path, error_string(fa.err().unwrap().ord() as i64));
            return;
        }
        let mut fa = fa.unwrap();

        let ftl = parse(fa.get_as_text().to_string());
        let mut ftl = match ftl {
            Ok(ftl) => ftl,
            Err((ftl, err)) => {
                godot_warn!("Error parsing {}: {:?}", path, err);
                ftl
            },
        };

        // Rewrite FTL.
        let existing_messages: Vec<&ast::Message<String>> = ftl.body.iter().filter_map(|entry| {
            if let ast::Entry::Message(msg) = entry {
                Some(msg)
            } else {
                None
            }
        }).collect();
        let mut new_messages = Vec::new();

        for (identifier, message) in messages {
            // Check if exists.
            let existing = existing_messages.iter().find(|entry| entry.id.name == identifier);
            if existing.is_none() {
                // Add new message.
                godot_print!("{} added new message: {}", path, message);
                new_messages.push(ast::Entry::Message(ast::Message {
                    id: ast::Identifier {
                        name: identifier
                    },
                    value: Some(ast::Pattern {
                        elements: vec![
                            ast::PatternElement::TextElement {
                                value: message.clone(),
                            },
                        ],
                    }),
                    attributes: Default::default(),
                    comment: None,
                }));
            }
        }
        // TODO: Add an option to check for deleted messages, and mark them with a comment in the file:
        /*
        let mut deleted_messages: Vec<&ast::Message<String>> = existing_messages.clone().into_iter().collect();
        
        if existing.is_some() {
            // Acknowledge existing message.
            // TODO: could be optimized by creating a wrapper trait so that messages can be put into a HashSet
            let index = deleted_messages.iter().position(|entry| entry.id.name == message);
            if let Some(index) = index {
                deleted_messages.remove(index);
            }
        }

        // Mark messages that no longer exist with some comment, to have user check for possible deletion.
        for message in deleted_messages {
            // TODO: append to existing comment, instead of replacing.
            message.comment = Some(ast::Comment {
                content: vec!["TODO: Currently unused, check for deletion?"],
            });
        }
        */

        drop(existing_messages);
        ftl.body.append(&mut new_messages);

        // Save back to file.
        let ftl = serialize(&ftl);
        if fa.resize(0) != GdErr::OK {
            godot_error!("Failed to resize file {}", path);
            return;
        }
        fa.store_string(ftl.into());
    }

    fn make_safe_identifier(name: String) -> String {
        // Identifiers are [a-zA-Z][a-zA-Z0-9_-]*
        if name.is_empty() {
            panic!("Identifier name can't be empty.");
        }

        const REPLACEMENT_CHAR: char = '_';
        let mut new_name = String::new();
        let mut chars = name.chars();

        let first_char = chars.next().unwrap();
        if !first_char.is_ascii_alphabetic() {
            new_name.push(REPLACEMENT_CHAR);
        } else {
            new_name.push(first_char);
        }

        for char in chars {
            if !char.is_ascii_alphanumeric() && char != '-' && char != '_' {
                new_name.push(REPLACEMENT_CHAR);
            } else {
                new_name.push(char);
            }
        }

        new_name
    }
}