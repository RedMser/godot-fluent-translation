use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use fluent::types::{FluentNumber, FluentNumberOptions};
use fluent::{FluentArgs, FluentBundle, FluentError, FluentResource, FluentValue};
use godot::prelude::*;
use godot::engine::{ITranslation, Translation};
use godot::engine::global::Error as GdErr;
use unic_langid::LanguageIdentifier;

#[derive(GodotClass)]
#[class(base=Translation)]
struct TranslationFluent {
    bundles: Arc<RwLock<HashMap<StringName, FluentBundle<FluentResource>>>>,
    languages: Arc<RwLock<Vec<StringName>>>,
}

#[godot_api]
impl ITranslation for TranslationFluent {
    fn init(_base: Base<Translation>) -> Self {
        Self {
            bundles: Arc::new(RwLock::new(HashMap::new())),
            languages: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn get_message(&self, src_message: StringName, args: Dictionary, context: StringName) -> StringName {
        let bundles = self.bundles.read().unwrap();
        let languages = self.languages.read().unwrap();

        let result = languages
            .iter()
            .map(|lang| {
                let bundle = bundles.get(lang)?;
                Self::translate(bundle, &src_message, &args, if context.is_empty() { None } else { Some(&context) })
            })
            .filter(|v| v.is_some())
            .map(|v| v.unwrap())
            .next();

        match result {
            Some(text) => StringName::from(text),
            None => StringName::default(),
        }
    }

    fn get_plural_message(&self, src_message: StringName, _src_plural_message: StringName, _n: i32, _context: StringName) -> StringName {
        godot_warn!("TranslationFluent does not handle get_plural_message. \nUse get_message with args instead.");
        src_message
    }
}

#[godot_api]
impl TranslationFluent {
    fn map_fluent_error<T>(result: &Result<T, Vec<FluentError>>) -> GdErr {
        match result {
            Ok(_) => GdErr::OK,
            Err(errs) => {
                // TODO: Just take first error for now...
                let err = errs.first();
                match err {
                    Some(FluentError::Overriding { kind, id }) => {
                        godot_warn!("{} with id {} already exists!", kind, id);
                        GdErr::ERR_ALREADY_EXISTS
                    }
                    Some(FluentError::ParserError(_err)) => {
                        // TODO: figure out string from err instance via "kind" / "thiserror" derive
                        GdErr::ERR_PARSE_ERROR
                    }
                    Some(FluentError::ResolverError(err)) => {
                        godot_warn!("{}", err);
                        GdErr::ERR_CANT_RESOLVE
                    }
                    None => GdErr::FAILED
                }
            }
        }
    }

    fn variant_to_fluent<'a>(input: Variant) -> FluentValue<'a> {
        match input.get_type() {
            VariantType::String | VariantType::StringName | VariantType::NodePath => {
                let stringified = input.stringify();
                let stringified = String::from(stringified);
                FluentValue::String(Cow::Owned(stringified))
            },
            VariantType::Int => {
                let casted: i64 = input.to();
                let mut options = FluentNumberOptions::default();
                options.maximum_fraction_digits = Some(0);
                FluentValue::Number(FluentNumber::new(casted as f64, options))
            }
            VariantType::Float => {
                let casted: f64 = input.to();
                FluentValue::Number(FluentNumber::new(casted, Default::default()))
            }
            VariantType::Nil => FluentValue::None,
            _ => FluentValue::Error,
        }
    }

    fn dict_to_args(input: &Dictionary) -> FluentArgs {
        let mut output = FluentArgs::with_capacity(input.len());
        for (key, value) in input.iter_shared() {
            let key = key.stringify();
            let key = String::from(key);
            let fluent_value = Self::variant_to_fluent(value.clone());
            match fluent_value {
                FluentValue::Error => { godot_warn!(
                    "Args contained value {} of unsupported type {:?} - must be one of String, int, float, StringName, NodePath or null",
                    value, value.get_type()
                ); },
                _ => output.set(key, fluent_value),
            };
        }
        output
    }

    fn translate(bundle: &FluentBundle<FluentResource>, message_id: &StringName, args: &Dictionary, attribute: Option<&StringName>) -> Option<String> {
        let message = bundle.get_message(&String::from(message_id));
        if message.is_none() {
            return None;
        }
        let message = message.unwrap();
        let pattern = match attribute {
            Some(attribute) => {
                message.get_attribute(&String::from(attribute))
                    .map(|attr| attr.value())
            },
            None => message.value(),
        };
        if pattern.is_none() {
            return None;
        }
        let pattern = pattern.unwrap();
        let mut errors = vec![];
        let args = Self::dict_to_args(args);
        let text = bundle.format_pattern(pattern, Some(&args), &mut errors);
        if !errors.is_empty() {
            godot_warn!("Errors in message {}: {:?}", message_id, errors);
            return None;
        }
        Some(text.into_owned())
    }

    #[func]
    fn add_bundle_from_text(&mut self, lang: StringName, text: String) -> GdErr {
        let res = FluentResource::try_new(text);
        if res.is_err() {
            // TODO: I could give more parser error details here, and probably should? :)
            return GdErr::ERR_PARSE_ERROR;
        }
        let res = res.unwrap();
        let mut bundles = self.bundles.write().unwrap();
        let entry = bundles.entry(lang.clone());
        let err = match entry {
            Entry::Occupied(mut e) => {
                let bundle = e.get_mut();
                Self::map_fluent_error(&bundle.add_resource(res))
            }
            Entry::Vacant(e) => {
                let lang_id = String::from(lang.clone()).parse::<LanguageIdentifier>();
                match lang_id {
                    Err(_err) => {
                        // TODO: I could give more error info here, but likely not helpful?
                        GdErr::ERR_INVALID_DATA
                    },
                    Ok(lang_id) => {
                        let mut bundle = FluentBundle::new(vec!(lang_id));
                        // bundle.set_use_isolating(false);
                        let err = Self::map_fluent_error(&bundle.add_resource(res));
                        e.insert(bundle);
                        err
                    }
                }
            }
        };
        if err == GdErr::OK {
            let mut languages = self.languages.write().unwrap();
            languages.push(lang);
        }
        err
    }
}