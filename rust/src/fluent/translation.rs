use std::borrow::Cow;
use fluent::types::FluentNumber;
use fluent::{FluentArgs, FluentBundle, FluentError, FluentResource, FluentValue};
use godot::prelude::*;
use godot::classes::{ITranslation, ProjectSettings, RegEx, Translation};
use godot::global::{str_to_var, var_to_str};
use godot::global::Error as GdErr;
use unic_langid::{LanguageIdentifier, LanguageIdentifierError};

use crate::hacks::SyncSendCallable;
use crate::utils::get_single_regex_match;

use super::project_settings::{PROJECT_SETTING_FALLBACK_LOCALE, PROJECT_SETTING_PARSE_ARGS_IN_MESSAGE, PROJECT_SETTING_UNICODE_ISOLATION};

/// Translation resource containing one or more Fluent Translation Lists (FTLs).
/// 
/// Can be loaded from a `.ftl` file via [method @GDScript.load] or manually populated using [method add_bundle_from_text].
/// When using the forked build of the add-on, you can also add a `.ftl` file to the Project Settings in the Localization -> Translations tab.
/// 
/// Any time a [TranslationFluent] instance is created by the add-on, the [member message_pattern] and [member locale] are automatically filled
/// depending on the corresponding Project Settings.
#[derive(GodotClass)]
#[class(base=Translation)]
#[allow(dead_code)]
pub struct TranslationFluent {
    /// Automatically wrap every message with the specified regex pattern, defined as a string pattern.
    /// This can be specified in order to create custom namespaces for your translations.
    /// 
    /// For example, if a message `hello = abc` is defined with the pattern `^test_(.+)$`, then you must call `tr("test_hello")` to get the message's translation.
    /// **Note**: The regex pattern is case sensitive by default. Prefix your pattern with `(?i)` in order to make it case insensitive.
    #[var(get = get_message_pattern, set = set_message_pattern)]
    message_pattern: GString,
    message_pattern_regex: Option<Gd<RegEx>>,
    bundle: Option<FluentBundle<FluentResource>>,
    base: Base<Translation>,
}

#[godot_api]
impl ITranslation for TranslationFluent {
    fn init(base: Base<Translation>) -> Self {
        // HACK: To avoid crashes with unreferenced parent, intentionally leak memory. See https://github.com/godot-rust/gdext/issues/557
        std::mem::forget(base.to_gd());

        // Default to an empty locale, so that it must be explicitly specified when loading a FTL file.
        base.to_gd().set_locale(&GString::new());

        Self {
            message_pattern: GString::new(),
            message_pattern_regex: None,
            bundle: None,
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
    #[func]
    pub fn get_message_pattern(&self) -> GString {
        self.message_pattern_regex.clone().map_or(GString::new(), |regex| regex.get_pattern())
    }

    #[func]
    pub fn set_message_pattern(&mut self, value: GString) {
        self.message_pattern_regex = if value.is_empty() {
            None
        } else {
            RegEx::create_from_string(&value)
        };
    }

    fn get_message_impl(&self, src_message: StringName, args: Dictionary, context: StringName) -> StringName {
        let (mut msg, args) = if args.is_empty() {
            Self::extract_args(src_message.clone())
        } else {
            (src_message, args)
        };

        if let Some(regex) = &self.message_pattern_regex {
            // Get actual message and see if it matches.
            if let Some(regex_match) = regex.search(msg.arg()) {
                msg = get_single_regex_match(regex_match, "message_pattern").into();
            } else {
                // Did not match, can not translate.
                return StringName::default();
            }
        }

        let result = self.translate(&msg, &args.clone(), if context.is_empty() { None } else { Some(&context) });
        match result {
            Some(text) => StringName::from(text),
            None => StringName::default(),
        }
    }

    fn map_langid_error(error: &LanguageIdentifierError) -> GdErr {
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

    fn map_fluent_error(error: &FluentError) -> GdErr {
        match error {
            FluentError::Overriding { kind, id } => {
                godot_warn!("{} with id {} already exists!", kind, id);
                GdErr::ERR_ALREADY_EXISTS
            }
            FluentError::ParserError(_err) => {
                // TODO: figure out string from err instance via "kind" / "thiserror" derive
                GdErr::ERR_PARSE_ERROR
            }
            FluentError::ResolverError(err) => {
                godot_warn!("{}", err);
                GdErr::ERR_CANT_RESOLVE
            }
        }
    }

    fn map_fluent_error_list(errors: &[FluentError]) -> GdErr {
        // TODO: Just take first error for now...
        let error = errors.first();
        match error {
            Some(error) => Self::map_fluent_error(error),
            None => GdErr::FAILED,
        }
    }

    fn fluent_to_variant(input: &FluentValue) -> Variant {
        match input {
            FluentValue::String(str) => str.clone().into_owned().to_godot().to_variant(),
            FluentValue::Number(num) => {
                // TODO: unsure what the default value for maximum_fraction_digits is, but likely not zero
                if let Some(0) = num.options.maximum_fraction_digits {
                    // int
                    (num.value as i64).to_godot().to_variant()
                } else {
                    // float
                    num.value.to_godot().to_variant()
                }
            },
            FluentValue::Custom(_custom) => todo!("Custom FluentValue conversion"),
            FluentValue::None => Variant::nil(),
            FluentValue::Error => {
                godot_error!("Tried to convert FluentValue::Error to a Variant.");
                Variant::nil()
            }
        }
    }

    fn variant_to_fluent<'a>(input: Variant) -> FluentValue<'a> {
        match input.get_type() {
            VariantType::STRING | VariantType::STRING_NAME | VariantType::NODE_PATH => {
                let stringified = input.stringify();
                let stringified = String::from(stringified);
                FluentValue::String(Cow::Owned(stringified))
            },
            VariantType::INT => {
                let casted: i64 = input.to();
                FluentValue::Number(FluentNumber::new(casted as f64, Default::default()))
            }
            VariantType::FLOAT => {
                let casted: f64 = input.to();
                FluentValue::Number(FluentNumber::new(casted, Default::default()))
            }
            VariantType::NIL => FluentValue::None,
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

    pub fn translate(&self, message_id: &StringName, args: &Dictionary, attribute: Option<&StringName>) -> Option<String> {
        if self.bundle.is_none() {
            godot_error!("Unable to translate before adding at least one FTL file to translation. Use append_from_text() or load().");
            return None;
        }

        let bundle = self.bundle.as_ref().unwrap();
        let message = bundle.get_message(&String::from(message_id));
        message.as_ref()?;

        let message = message.unwrap();
        let pattern = match attribute {
            Some(attribute) => {
                message.get_attribute(&String::from(attribute))
                    .map(|attr| attr.value())
            },
            None => message.value(),
        }?;

        let mut errors = vec![];
        let args = Self::dict_to_args(args);
        let text = bundle.format_pattern(pattern, Some(&args), &mut errors);
        if !errors.is_empty() {
            godot_warn!("Errors in message {}: {:?}", message_id, errors);
            return None;
        }
        Some(text.into_owned())
    }

    /// Attach arguments (also known as variables) to a message.
    /// A translation can use these values using `{ $variableName }` syntax in the FTL.
    /// 
    /// This method is only needed when using the default version of the add-on, as the forked build includes an additional `args` parameter to the different translation methods.
    #[func]
    pub fn args(msg: StringName, args: Dictionary) -> StringName {
        let args = var_to_str(&Variant::from(args)).to_string();
        let msg = msg.to_string() + &args;
        msg.into()
    }

    fn extract_args(msg: StringName) -> (StringName, Dictionary) {
        let parse_args_in_message = {
            let project_settings = ProjectSettings::singleton();
            bool::from_variant(&project_settings.get_setting(PROJECT_SETTING_PARSE_ARGS_IN_MESSAGE))
        };

        if parse_args_in_message {
            // Try parsing trailing dict as args.
            let msg_str = msg.to_string();
            if msg_str.ends_with('}') {
                let open_brace = msg_str.rfind('{');
                if let Some(open_brace) = open_brace {
                    let args = &msg_str[open_brace..];
                    let args = str_to_var(args);
                    if args.get_type() != VariantType::DICTIONARY {
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

    /// Add a Fluent Translation List (FTL) text to this translation.
    /// This method is automatically called when the add-on creates a [TranslationFluent] resource for you (e.g. when using [method @GDScript.load]).
    /// 
    /// Returns an [enum Error] value whether the data was successfully added.
    /// 
    /// **Note**: When this method is called, certain Project Settings values are read.
    /// Changing these Project Settings after this call will not update already existing [TranslationFluent] resources.
    #[func]
    pub fn append_from_text(&mut self, text: String) -> GdErr {
        let bundle = match &mut self.bundle {
            Some(bundle) => bundle,
            None => &mut {
                let bundle = self.create_bundle();
                match bundle {
                    Ok(bundle) => {
                        self.bundle = Some(bundle);
                        self.bundle.as_mut().unwrap()
                    },
                    Err(err) => return err
                }
            },
        };

        let Ok(res) = FluentResource::try_new(text) else {
            // TODO: I could give more parser error details here, and probably should? :)
            return GdErr::ERR_PARSE_ERROR;
        };

        match bundle.add_resource(res) {
            Ok(_) => GdErr::OK,
            Err(errors) => Self::map_fluent_error_list(&errors),
        }
    }

    fn create_bundle(&self) -> Result<FluentBundle<FluentResource>, GdErr> {
        let mut bundle = FluentBundle::new(self.get_fluent_locales()?);
        let project_settings = ProjectSettings::singleton();
        bundle.set_use_isolating(project_settings.get_setting(PROJECT_SETTING_UNICODE_ISOLATION).booleanize());
        Ok(bundle)
    }

    // TODO: Listen to NOTIFICATION_TRANSLATION_CHANGED on MainLoop. On notification, update the existing bundle's "locales" field.
    fn get_fluent_locales(&self) -> Result<Vec<LanguageIdentifier>, GdErr> {
        let lang = self.base().get_locale();
        if lang.is_empty() {
            // Give a user-friendly message.
            godot_error!("Failed to create bundle for TranslationFluent: locale has not been set.");
            return Err(GdErr::ERR_DOES_NOT_EXIST);
        }

        let lang_id = String::from(lang).parse::<LanguageIdentifier>();
        match lang_id {
            Err(err) => Err(Self::map_langid_error(&err)),
            Ok(lang_id) => {
                let project_settings = ProjectSettings::singleton();
                let mut locales = vec![lang_id];
                // Use TranslationServer fallback if it exists (same check as TS::translate).
                let fallback_locale = project_settings.get_setting(PROJECT_SETTING_FALLBACK_LOCALE).stringify();
                if fallback_locale.len() >= 2 {
                    let fallback_locale_id = fallback_locale.to_string().parse::<LanguageIdentifier>();
                    match fallback_locale_id {
                        Err(err) => return Err(Self::map_langid_error(&err)),
                        Ok(fallback_locale_id) => {
                            locales.push(fallback_locale_id);
                        }
                    }
                }
                Ok(locales)
            }
        }
    }

    /// Defines a custom function that can be called in a placeable.
    /// 
    /// [param name] is the name of the custom function to register. It must be an all-uppercase string.
    /// 
    /// [param callable] takes two parameters `positional: Array[String|int|float]` and `named: Dictionary[String, String|int|float]` and must return `String|int|float|null`.
    #[func]
    pub fn add_function(&mut self, name: GString, callable: SyncSendCallable) -> GdErr {
        {
            let args_count = callable.get_argument_count();
            if args_count != 2 {
                godot_error!("add_function expects a callable with exactly 2 arguments, but provided callable has {args_count}.",)
            }
        }

        let bundle = match &mut self.bundle {
            Some(bundle) => bundle,
            None => &mut {
                let bundle = self.create_bundle();
                match bundle {
                    Ok(bundle) => {
                        self.bundle = Some(bundle);
                        self.bundle.as_mut().unwrap()
                    },
                    Err(err) => return err
                }
            },
        };

        let name = String::from(name);
        let name_upper = name.to_uppercase();
        if name != name_upper {
            godot_warn!("add_function requires function names to be uppercase. Registered function as {name_upper}");
        }

        let add_result = bundle.add_function(&name_upper, move |positional, named| {
            // Convert args to variants
            let positional_variants = positional.iter()
                .map(|value| Self::fluent_to_variant(value))
                .collect::<VariantArray>();
            let named_variants = named.iter()
                .map(|(key, value)| (key, Self::fluent_to_variant(value)))
                .collect::<Dictionary>();

            // Run the function and convert its result.
            let args = varray![positional_variants, named_variants];
            let result = callable.callv(&args);
            let result_variant = Self::variant_to_fluent(result);
            result_variant
        });

        match add_result {
            Ok(_) => GdErr::OK,
            Err(error) => Self::map_fluent_error(&error),
        }
    }
}
