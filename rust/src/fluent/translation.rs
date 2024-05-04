use std::borrow::Cow;
use std::sync::{Arc, RwLock};

use fluent::types::{FluentNumber, FluentNumberOptions};
use fluent::{FluentArgs, FluentBundle, FluentError, FluentResource, FluentValue};
use godot::engine::utilities::{str_to_var, var_to_str};
use godot::prelude::*;
use godot::engine::{ITranslation, ProjectSettings, Translation};
use godot::engine::global::Error as GdErr;
use unic_langid::{LanguageIdentifier, LanguageIdentifierError};

use super::project_settings::{PROJECT_SETTING_FALLBACK_LOCALE, PROJECT_SETTING_PARSE_ARGS_IN_MESSAGE};

#[derive(GodotClass)]
#[class(base=Translation)]
pub struct TranslationFluent {
    bundles: Arc<RwLock<Vec<FluentBundle<FluentResource>>>>,
    base: Base<Translation>,
}

#[godot_api]
impl ITranslation for TranslationFluent {
    fn init(base: Base<Translation>) -> Self {
        // HACK: To avoid crashes with unreferenced parent, intentionally leak memory. See https://github.com/godot-rust/gdext/issues/557
        std::mem::forget(base.to_gd());

        // Default to an empty locale, so that it must be explicitly specified when loading a FTL file.
        base.to_gd().set_locale(GString::new());

        Self {
            bundles: Arc::new(RwLock::new(Vec::new())),
            base,
        }
    }

    #[cfg(not(feature = "forked-godot"))]
    fn get_message(&self, src_message: StringName, context: StringName) -> StringName {
        self.get_message_impl(src_message, Default::default(), context)
    }

    #[cfg(feature = "forked-godot")]
    fn get_message(&self, src_message: StringName, args: Dictionary, context: StringName) -> StringName {
        self.get_message_impl(src_message, args, context)
    }

    fn get_plural_message(&self, src_message: StringName, _src_plural_message: StringName, _n: i32, _context: StringName) -> StringName {
        godot_warn!("TranslationFluent does not handle get_plural_message. \nUse get_message with args instead.");
        src_message
    }
}

#[godot_api]
impl TranslationFluent {
    fn get_message_impl(&self, src_message: StringName, args: Dictionary, context: StringName) -> StringName {
        let (msg, args) = if args.is_empty() {
            Self::extract_args(src_message)
        } else {
            (src_message, args)
        };

        let bundles = self.bundles.read().unwrap();

        let result = bundles
            .iter()
            .map(|bundle| {
                Self::translate(bundle, &msg, &args.clone(), if context.is_empty() { None } else { Some(&context) })
            })
            .filter(|v| v.is_some())
            .map(|v| v.unwrap())
            .next();

        match result {
            Some(text) => StringName::from(text),
            None => StringName::default(),
        }
    }

    fn map_langid_error(error: LanguageIdentifierError) -> GdErr {
        match error {
            LanguageIdentifierError::ParserError(error) => {
                match error {
                    unic_langid::parser::ParserError::InvalidLanguage => GdErr::ERR_DOES_NOT_EXIST,
                    unic_langid::parser::ParserError::InvalidSubtag => GdErr::ERR_INVALID_DATA,
                }
            },
            LanguageIdentifierError::Unknown => GdErr::ERR_INVALID_DATA,
        }
    }

    fn map_fluent_error(result: &Result<(), Vec<FluentError>>) -> GdErr {
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

    pub fn translate(bundle: &FluentBundle<FluentResource>, message_id: &StringName, args: &Dictionary, attribute: Option<&StringName>) -> Option<String> {
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
    pub fn args(msg: StringName, args: Dictionary) -> StringName {
        let args = var_to_str(Variant::from(args)).to_string();
        let msg = msg.to_string() + &args;
        msg.into()
    }

    fn extract_args(msg: StringName) -> (StringName, Dictionary) {
        let project_settings = ProjectSettings::singleton();
        let parse_args_in_message = bool::from_variant(&project_settings.get_setting(PROJECT_SETTING_PARSE_ARGS_IN_MESSAGE.into()));
        if parse_args_in_message {
            // Try parsing trailing dict as args.
            let msg_str = msg.to_string();
            if msg_str.ends_with('}') {
                let open_brace = msg_str.rfind('{');
                if let Some(open_brace) = open_brace {
                    let args = &msg_str[open_brace..];
                    let args = str_to_var(args.into());
                    if args.get_type() != VariantType::Dictionary {
                        return (msg, Default::default());
                    }
                    let args = Dictionary::from_variant(&args);
                    let msg_str = &msg_str[0..open_brace];
                    let msg_str = StringName::from(msg_str);
                    return (msg_str, args);
                }
            }
        }
        (msg, Default::default())
    }

    #[func]
    pub fn add_bundle_from_text(&mut self, text: String) -> GdErr {
        let res = FluentResource::try_new(text);
        if res.is_err() {
            // TODO: I could give more parser error details here, and probably should? :)
            return GdErr::ERR_PARSE_ERROR;
        }
        let lang = self.base().get_locale();
        if lang.is_empty() {
            // Give a user-friendly message.
            godot_error!("Failed to add bundle to TranslationFluent: locale has not been set.");
            return GdErr::ERR_DOES_NOT_EXIST;
        }
        let res = res.unwrap();
        let mut bundles = self.bundles.write().unwrap();
        let lang_id = String::from(lang).parse::<LanguageIdentifier>();
        match lang_id {
            Err(err) => Self::map_langid_error(err),
            Ok(lang_id) => {
                let mut locales = vec![lang_id];
                // Use TranslationServer fallback if it exists (same check as TS::translate).
                let fallback_locale = ProjectSettings::singleton().get_setting(PROJECT_SETTING_FALLBACK_LOCALE.into()).stringify();
                if fallback_locale.len() >= 2 {
                    let fallback_locale_id = fallback_locale.to_string().parse::<LanguageIdentifier>();
                    match fallback_locale_id {
                        Err(err) => return Self::map_langid_error(err),
                        Ok(fallback_locale_id) => {
                            locales.push(fallback_locale_id);
                        }
                    }
                }
                let mut bundle = FluentBundle::new(locales);
                // bundle.set_use_isolating(false);
                let err = Self::map_fluent_error(&bundle.add_resource(res));
                bundles.push(bundle);
                err
            }
        }
    }
}