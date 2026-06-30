use std::collections::HashMap;

use dust_dart_emit::{DART_LIST, DART_STRING, DART_VOID, apply_rename_rule};
use dust_ir::{BuiltinType, ClassIr, DartFileIr, EnumIr, TypeIr};

use crate::plugin::emit::test_support::SampleValue;
use crate::plugin::util::is_string_keyed_map;

/// Lookup table used to build generated HTTP test fixture values.
pub(super) struct FixtureCatalog<'a> {
    /// Classes available in the source library, keyed by Dart type name.
    classes: HashMap<&'a str, &'a ClassIr>,
    /// Enums available in the source library, keyed by Dart type name.
    enums: HashMap<&'a str, &'a EnumIr>,
}

impl<'a> FixtureCatalog<'a> {
    /// Builds a fixture catalog from all classes and enums in a parsed library.
    pub(super) fn from_library(library: &'a DartFileIr) -> Self {
        Self {
            classes: library
                .classes
                .iter()
                .map(|class| (class.name.as_str(), class))
                .collect(),
            enums: library
                .enums
                .iter()
                .map(|enum_ir| (enum_ir.name.as_str(), enum_ir))
                .collect(),
        }
    }

    /// Returns a generated Dart sample expression for the given type.
    pub(super) fn sample_value(&self, ty: &TypeIr) -> Option<SampleValue> {
        self.sample_value_inner(ty, &mut Vec::new())
    }

    /// Returns a JSON literal that can decode into the given type.
    pub(super) fn json_value(&self, ty: &TypeIr) -> Option<String> {
        self.json_value_inner(ty, &mut Vec::new())
    }

    /// Recursively renders a sample value while guarding against class cycles.
    fn sample_value_inner(&self, ty: &TypeIr, stack: &mut Vec<&'a str>) -> Option<SampleValue> {
        match ty {
            TypeIr::Builtin { kind, .. } => Some(sample_builtin(*kind)),
            TypeIr::Dynamic | TypeIr::Unknown => Some(SampleValue::new("'dust'", Some("dust"))),
            TypeIr::Named { name, nullable, .. } if *nullable => {
                Some(SampleValue::new("null", Some("null")))
            }
            TypeIr::Named { name, .. } if name.as_ref() == DART_STRING => {
                Some(SampleValue::new("'dust-id'", Some("dust-id")))
            }
            TypeIr::Named { name, args, .. } if name.as_ref() == DART_LIST && args.len() == 1 => {
                self.sample_list_value(&args[0], stack)
            }
            TypeIr::Named { args, .. } if is_string_keyed_map(ty) => {
                self.sample_map_value(&args[1], stack)
            }
            TypeIr::Named { name, .. } if name.as_ref() == DART_VOID => {
                Some(SampleValue::new("null", Some("null")))
            }
            TypeIr::Named { name, .. } => self
                .sample_enum_value(name)
                .or_else(|| self.sample_class_value(name, stack)),
            TypeIr::Function { .. } | TypeIr::Record { .. } => None,
        }
    }

    /// Renders a sample `List<T>` expression.
    fn sample_list_value(&self, inner: &TypeIr, stack: &mut Vec<&'a str>) -> Option<SampleValue> {
        let item = self.sample_value_inner(inner, stack)?;
        Some(SampleValue::new(&format!("[{}]", item.expression), None))
    }

    /// Renders a sample `Map<String, T>` expression.
    fn sample_map_value(&self, inner: &TypeIr, stack: &mut Vec<&'a str>) -> Option<SampleValue> {
        let value = self.sample_value_inner(inner, stack)?;
        Some(SampleValue::new(
            &format!("{{'value': {}}}", value.expression),
            None,
        ))
    }

    /// Renders a sample enum expression using the first declared variant.
    fn sample_enum_value(&self, name: &str) -> Option<SampleValue> {
        let enum_ir = self.enums.get(name)?;
        let variant = enum_ir.variants.first()?;
        Some(SampleValue::new(
            &format!("{}.{}", enum_ir.name, variant.name),
            None,
        ))
    }

    /// Renders a sample data-class expression through `fromJson`.
    fn sample_class_value(&self, name: &str, stack: &mut Vec<&'a str>) -> Option<SampleValue> {
        let class = *self.classes.get(name)?;
        if stack.contains(&class.name.as_str()) {
            return None;
        }

        let from_json = class
            .constructors
            .iter()
            .any(|ctor| ctor.is_factory && ctor.name.as_deref() == Some("fromJson"));
        if !from_json {
            return None;
        }

        stack.push(class.name.as_str());
        let json = self.class_json_map(class, stack);
        stack.pop();

        json.map(|json| SampleValue::new(&format!("{}.fromJson({json})", class.name), None))
    }

    /// Renders a JSON map literal suitable for constructing a source class.
    fn class_json_map(&self, class: &ClassIr, stack: &mut Vec<&'a str>) -> Option<String> {
        let mut entries = Vec::new();

        for field in &class.fields {
            let serde = field.serde.as_ref();
            if serde.is_some_and(|config| config.skip_deserializing) {
                continue;
            }

            let Some(value) = self.json_value_inner(&field.ty, stack) else {
                if can_omit_field(field.has_default, field.ty.is_nullable(), serde) {
                    continue;
                }
                return None;
            };

            entries.push(format!(
                "'{}': {}",
                json_key(class, field.name.as_str(), serde),
                value
            ));
        }

        if entries.is_empty() {
            Some("const <String, Object?>{}".to_owned())
        } else {
            Some(format!("<String, Object?>{{{}}}", entries.join(", ")))
        }
    }

    /// Recursively renders a JSON literal for a type.
    fn json_value_inner(&self, ty: &TypeIr, stack: &mut Vec<&'a str>) -> Option<String> {
        match ty {
            TypeIr::Builtin { kind, .. } => Some(match kind {
                BuiltinType::String => "'dust'".to_owned(),
                BuiltinType::Int => "42".to_owned(),
                BuiltinType::Bool => "true".to_owned(),
                BuiltinType::Double => "3.14".to_owned(),
                BuiltinType::Num => "7".to_owned(),
                BuiltinType::Object => "'dust'".to_owned(),
            }),
            TypeIr::Dynamic | TypeIr::Unknown => Some("'dust'".to_owned()),
            TypeIr::Named { nullable, .. } if *nullable => Some("null".to_owned()),
            TypeIr::Named { name, .. } if name.as_ref() == DART_STRING => Some("'dust'".to_owned()),
            TypeIr::Named { name, args, .. } if name.as_ref() == DART_LIST && args.len() == 1 => {
                let item = self.json_value_inner(&args[0], stack)?;
                Some(format!("[{item}]"))
            }
            TypeIr::Named { args, .. } if is_string_keyed_map(ty) => {
                let value = self.json_value_inner(&args[1], stack)?;
                Some(format!("<String, Object?>{{'value': {value}}}"))
            }
            TypeIr::Named { name, .. } if name.as_ref() == DART_VOID => Some("null".to_owned()),
            TypeIr::Named { name, .. } => self
                .sample_enum_json(name)
                .or_else(|| self.sample_class_json(name, stack)),
            TypeIr::Function { .. } | TypeIr::Record { .. } => None,
        }
    }

    /// Renders sample JSON for an enum, respecting serde rename rules.
    fn sample_enum_json(&self, name: &str) -> Option<String> {
        let enum_ir = self.enums.get(name)?;
        let variant = enum_ir.variants.first()?;
        let wire_name = match enum_ir.serde.as_ref().and_then(|serde| serde.rename_all) {
            Some(rule) => apply_rename_rule(variant.name.as_str(), rule),
            None => variant.name.clone(),
        };
        Some(format!("'{}'", wire_name))
    }

    /// Renders sample JSON for a class while guarding against cycles.
    fn sample_class_json(&self, name: &str, stack: &mut Vec<&'a str>) -> Option<String> {
        let class = *self.classes.get(name)?;
        if stack.contains(&class.name.as_str()) {
            return None;
        }

        stack.push(class.name.as_str());
        let json = self.class_json_map(class, stack);
        stack.pop();
        json
    }
}

/// Returns the canonical generated test sample for a Dart builtin type.
fn sample_builtin(kind: BuiltinType) -> SampleValue {
    match kind {
        BuiltinType::String => SampleValue::new("'dust-id'", Some("dust-id")),
        BuiltinType::Int => SampleValue::new("42", Some("42")),
        BuiltinType::Bool => SampleValue::new("true", Some("true")),
        BuiltinType::Double => SampleValue::new("3.14", Some("3.14")),
        BuiltinType::Num => SampleValue::new("7", Some("7")),
        BuiltinType::Object => SampleValue::new("{'value': 'dust'}", None),
    }
}

/// Returns true when generated class JSON may omit an unsupported field sample.
fn can_omit_field(
    has_default: bool,
    is_nullable: bool,
    serde: Option<&dust_ir::SerdeFieldConfigIr>,
) -> bool {
    has_default
        || is_nullable
        || serde
            .and_then(|config| config.default_value_source.as_ref())
            .is_some()
}

/// Resolves the JSON key for a field using field and class serde config.
fn json_key(
    class: &ClassIr,
    field_name: &str,
    serde: Option<&dust_ir::SerdeFieldConfigIr>,
) -> String {
    if let Some(rename) = serde.and_then(|config| config.rename.as_deref()) {
        return rename.to_owned();
    }

    match class.serde.as_ref().and_then(|config| config.rename_all) {
        Some(rule) => apply_rename_rule(field_name, rule),
        None => field_name.to_owned(),
    }
}
