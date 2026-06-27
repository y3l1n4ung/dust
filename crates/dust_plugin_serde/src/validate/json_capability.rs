//! Local JSON conversion capability checks for SerDe validation.

use std::collections::{HashMap, HashSet};

use dust_dart_emit::{DART_BIG_INT, DART_DATE_TIME, DART_MAP, DART_OBJECT, DART_URI};
use dust_ir::{BuiltinType, ClassIr, DartFileIr, ParamKind, TypeIr};
use dust_plugin_api::WorkspaceAnalysis;

use super::{wants_deserialize, wants_serialize};
use crate::analysis::{JSON_DESERIALIZABLE_TYPES_KEY, JSON_SERIALIZABLE_TYPES_KEY, JSON_TYPES_KEY};

/// Local JSON conversion facts available to SerDe validation.
pub(super) struct JsonModelContext<'a> {
    /// Classes declared in this library by name.
    classes: HashMap<&'a str, &'a ClassIr>,
    /// Classes for which Dust will generate `toJson` helpers.
    serializable_classes: HashSet<&'a str>,
    /// Classes for which Dust will generate `fromJson` helpers.
    deserializable_classes: HashSet<&'a str>,
    /// Enums for which Dust will generate `toJson` helpers.
    serializable_enums: HashSet<&'a str>,
    /// Enums for which Dust will generate `fromJson` helpers.
    deserializable_enums: HashSet<&'a str>,
    /// Type names discovered across workspace source files.
    workspace_types: &'a [String],
    /// Workspace type names that can serialize to JSON.
    workspace_serializable_types: &'a [String],
    /// Workspace type names that can deserialize from JSON.
    workspace_deserializable_types: &'a [String],
}

impl<'a> JsonModelContext<'a> {
    /// Collects JSON conversion facts from the current library IR.
    pub(super) fn new(library: &'a DartFileIr) -> Self {
        Self::with_workspace(library, None)
    }

    /// Collects JSON conversion facts from current IR and workspace analysis.
    pub(super) fn with_workspace(
        library: &'a DartFileIr,
        workspace: Option<&'a WorkspaceAnalysis>,
    ) -> Self {
        Self {
            classes: library
                .classes
                .iter()
                .map(|class| (class.name.as_str(), class))
                .collect(),
            serializable_classes: library
                .classes
                .iter()
                .filter(|class| wants_serialize(class))
                .map(|class| class.name.as_str())
                .collect(),
            deserializable_classes: library
                .classes
                .iter()
                .filter(|class| wants_deserialize(class))
                .map(|class| class.name.as_str())
                .collect(),
            serializable_enums: library
                .enums
                .iter()
                .filter(|item| {
                    item.traits
                        .iter()
                        .any(|trait_| trait_.symbol.0 == "dust_dart::Serialize")
                })
                .map(|item| item.name.as_str())
                .collect(),
            deserializable_enums: library
                .enums
                .iter()
                .filter(|item| {
                    item.traits
                        .iter()
                        .any(|trait_| trait_.symbol.0 == "dust_dart::Deserialize")
                })
                .map(|item| item.name.as_str())
                .collect(),
            workspace_types: workspace_string_slice(workspace, JSON_TYPES_KEY),
            workspace_serializable_types: workspace_string_slice(
                workspace,
                JSON_SERIALIZABLE_TYPES_KEY,
            ),
            workspace_deserializable_types: workspace_string_slice(
                workspace,
                JSON_DESERIALIZABLE_TYPES_KEY,
            ),
        }
    }
}

/// Returns whether a named non-generic type has a verified JSON conversion path.
pub(super) fn has_verified_json_conversion(
    context: &JsonModelContext<'_>,
    name: &str,
    direction: &str,
) -> bool {
    match direction {
        "Serialize" => {
            (match context.classes.get(name) {
                Some(class) => {
                    context.serializable_classes.contains(name) || has_to_json_method(class)
                }
                None => has_workspace_json_conversion(
                    context,
                    name,
                    context.workspace_serializable_types,
                ),
            }) || context.serializable_enums.contains(name)
        }
        "Deserialize" => {
            (match context.classes.get(name) {
                Some(class) => {
                    context.deserializable_classes.contains(name) || has_from_json_factory(class)
                }
                None => has_workspace_json_conversion(
                    context,
                    name,
                    context.workspace_deserializable_types,
                ),
            }) || context.deserializable_enums.contains(name)
        }
        _ => false,
    }
}

/// Returns one workspace string-set as a borrowed sorted slice.
fn workspace_string_slice<'a>(workspace: Option<&'a WorkspaceAnalysis>, key: &str) -> &'a [String] {
    workspace
        .and_then(|analysis| analysis.string_set(key))
        .unwrap_or_default()
}

/// Returns whether workspace facts prove or intentionally cannot disprove JSON support.
fn has_workspace_json_conversion(
    context: &JsonModelContext<'_>,
    name: &str,
    capable_types: &[String],
) -> bool {
    !contains_sorted_string(context.workspace_types, name)
        || contains_sorted_string(capable_types, name)
}

/// Returns whether a sorted string slice contains `needle`.
fn contains_sorted_string(values: &[String], needle: &str) -> bool {
    values
        .binary_search_by(|value| value.as_str().cmp(needle))
        .is_ok()
}

/// Returns the JSON member required for one conversion direction.
pub(super) fn required_json_member(name: &str, direction: &str) -> String {
    match direction {
        "Serialize" => format!("{name}.toJson()"),
        "Deserialize" => format!("{name}.fromJson(Map<String, Object?>)"),
        _ => name.to_owned(),
    }
}

/// Returns true for named Dart SDK scalars SerDe handles directly.
pub(super) fn is_supported_named_scalar(name: &str) -> bool {
    matches!(name, DART_OBJECT | DART_DATE_TIME | DART_URI | DART_BIG_INT)
}

/// Returns whether a class declares a usable instance `toJson` method.
fn has_to_json_method(class: &ClassIr) -> bool {
    class.methods.iter().any(|method| {
        method.name == "toJson"
            && !method.is_static
            && method.params.is_empty()
            && is_json_map_type(&method.return_type)
    })
}

/// Returns whether a class declares a usable factory `fromJson` constructor.
fn has_from_json_factory(class: &ClassIr) -> bool {
    class.constructors.iter().any(|constructor| {
        constructor.name.as_deref() == Some("fromJson")
            && constructor.is_factory
            && constructor.params.len() == 1
            && constructor.params[0].kind == ParamKind::Positional
            && is_json_map_type(&constructor.params[0].ty)
    })
}

/// Returns whether a type is a JSON object map.
fn is_json_map_type(ty: &TypeIr) -> bool {
    ty.is_named(DART_MAP) && ty.args().len() == 2 && ty.args()[0].is_builtin(BuiltinType::String)
}
