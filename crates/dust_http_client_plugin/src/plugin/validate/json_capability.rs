use std::collections::{HashMap, HashSet};

use dust_dart_emit::{DART_LIST, DART_MAP, DART_VOID};
use dust_diagnostics::Diagnostic;
use dust_ir::{BuiltinType, ClassIr, DartFileIr, MethodIr, MethodParamIr, ParamKind, TypeIr};
use dust_parser_dart::{
    ParameterKind, ParsedAnnotation, ParsedClassSurface, ParsedDartFileSurface, ParsedTypeKind,
    ParsedTypeSurface,
};
use dust_plugin_api::{WorkspaceAnalysis, WorkspaceAnalysisBuilder};

use crate::plugin::emit::uses_direct_body_value;
use crate::plugin::model::ReturnSpec;
use crate::plugin::util::{is_response_body_type, label, type_name_is};

/// Workspace type names known to the HTTP plugin during validation.
const HTTP_JSON_TYPES_KEY: &str = "dust_http_client_plugin.json_types.v1";
/// Workspace type names that can provide the `toJson()` call emitted for request bodies.
const HTTP_JSON_SERIALIZABLE_TYPES_KEY: &str = "dust_http_client_plugin.json_serializable_types.v1";
/// Workspace type names that declare the `fromJson(...)` factory emitted for responses.
const HTTP_JSON_FROM_JSON_TYPES_KEY: &str = "dust_http_client_plugin.json_from_json_types.v1";

/// JSON capability facts used by HTTP request and response validation.
pub(crate) struct JsonCapabilityContext<'a> {
    /// Classes declared in the current lowered file by short Dart name.
    classes: HashMap<&'a str, &'a ClassIr>,
    /// Enums declared in the current lowered file by short Dart name.
    enums: HashSet<&'a str>,
    /// Workspace-wide known type names.
    workspace_types: &'a [String],
    /// Workspace-wide type names that can serialize through generated HTTP body code.
    workspace_serializable_types: &'a [String],
    /// Workspace-wide type names that can deserialize through generated HTTP response code.
    workspace_from_json_types: &'a [String],
}

impl<'a> JsonCapabilityContext<'a> {
    /// Builds JSON capability facts from one lowered file.
    pub(crate) fn new(library: &'a DartFileIr) -> Self {
        Self::with_workspace(library, None)
    }

    /// Builds JSON capability facts from one lowered file and optional workspace analysis.
    pub(crate) fn with_workspace(
        library: &'a DartFileIr,
        workspace: Option<&'a WorkspaceAnalysis>,
    ) -> Self {
        Self {
            classes: library
                .classes
                .iter()
                .map(|class| (class.name.as_str(), class))
                .collect(),
            enums: library
                .enums
                .iter()
                .map(|enum_| enum_.name.as_str())
                .collect(),
            workspace_types: workspace_string_slice(workspace, HTTP_JSON_TYPES_KEY),
            workspace_serializable_types: workspace_string_slice(
                workspace,
                HTTP_JSON_SERIALIZABLE_TYPES_KEY,
            ),
            workspace_from_json_types: workspace_string_slice(
                workspace,
                HTTP_JSON_FROM_JSON_TYPES_KEY,
            ),
        }
    }

    /// Returns whether `name` can satisfy the generated `value.toJson()` request path.
    fn has_json_serializer(&self, name: &str) -> bool {
        let name = short_name(name);
        match self.classes.get(name) {
            Some(class) => wants_serialize(class) || has_to_json_method(class),
            None if self.enums.contains(name) => false,
            None => {
                !contains_sorted_string(self.workspace_types, name)
                    || contains_sorted_string(self.workspace_serializable_types, name)
            }
        }
    }

    /// Returns whether `name` can satisfy the generated `Type.fromJson(...)` response path.
    fn has_from_json_factory(&self, name: &str) -> bool {
        let name = short_name(name);
        match self.classes.get(name) {
            Some(class) => has_from_json_factory(class),
            None if self.enums.contains(name) => false,
            None => {
                !contains_sorted_string(self.workspace_types, name)
                    || contains_sorted_string(self.workspace_from_json_types, name)
            }
        }
    }
}

/// Collects parse-only workspace JSON facts used by HTTP validation.
pub(crate) fn collect_workspace_analysis(
    library: &ParsedDartFileSurface,
    analysis: &mut WorkspaceAnalysisBuilder,
) {
    for class in &library.classes {
        analysis.add_string_set_value(HTTP_JSON_TYPES_KEY, class.name.clone());
        if class_has_serialize(class) || parsed_has_to_json_method(class) {
            analysis.add_string_set_value(HTTP_JSON_SERIALIZABLE_TYPES_KEY, class.name.clone());
        }
        if parsed_has_from_json_factory(class) {
            analysis.add_string_set_value(HTTP_JSON_FROM_JSON_TYPES_KEY, class.name.clone());
        }
    }

    for enum_ in &library.enums {
        analysis.add_string_set_value(HTTP_JSON_TYPES_KEY, enum_.name.clone());
    }
}

/// Validates the JSON support required by a generated `@Body()` expression.
pub(super) fn validate_body_json_capability(
    context: &JsonCapabilityContext<'_>,
    class: &ClassIr,
    method: &MethodIr,
    param: &MethodParamIr,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if uses_direct_body_value(&param.ty) {
        return;
    }

    let Some(name) = param.ty.name().map(short_name) else {
        diagnostics.push(unsupported_body_type(class, method, param));
        return;
    };
    if context.has_json_serializer(name) {
        return;
    }

    diagnostics.push(
        Diagnostic::error(format!(
            "parameter `{}` on `{}.{}` uses `@Body()` with `{}` but generated HTTP serialization requires `{}.toJson()`",
            param.name, class.name, method.name, name, name
        ))
        .with_label(label(
            param.span,
            format!("add `{name}.toJson()`, derive `Serialize`, or change this body type to a primitive, `Map`, `List`, or `dynamic`"),
        )),
    );
}

/// Validates the JSON support required by generated response decoding.
pub(super) fn validate_response_json_capability(
    context: &JsonCapabilityContext<'_>,
    class: &ClassIr,
    method: &MethodIr,
    return_spec: &ReturnSpec,
    diagnostics: &mut Vec<Diagnostic>,
) {
    validate_response_type(context, class, method, &return_spec.ty, diagnostics);
}

/// Recursively validates response payload types that generated HTTP code decodes.
fn validate_response_type(
    context: &JsonCapabilityContext<'_>,
    class: &ClassIr,
    method: &MethodIr,
    ty: &TypeIr,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match ty {
        TypeIr::Dynamic | TypeIr::Unknown | TypeIr::Builtin { .. } => {}
        TypeIr::Named { name, args, .. } if name.as_ref() == DART_LIST && args.len() == 1 => {
            validate_response_type(context, class, method, &args[0], diagnostics);
        }
        TypeIr::Named { name, .. } if name.as_ref() == DART_MAP => {}
        TypeIr::Named { name, .. } if name.as_ref() == DART_VOID => {}
        TypeIr::Named { .. } if is_response_body_type(ty) => {}
        TypeIr::Named { name, .. } if context.has_from_json_factory(name) => {}
        TypeIr::Named { name, .. } => {
            let name = short_name(name);
            diagnostics.push(
                Diagnostic::error(format!(
                    "method `{}` on `{}` returns `{}` but generated HTTP deserialization requires `{}.fromJson(Map<String, Object?>)`",
                    method.name, class.name, name, name
                ))
                .with_label(label(
                    method.span,
                    format!("add a `factory {name}.fromJson(Map<String, Object?> json)` or change this response type to a primitive, `Map`, `List`, `dynamic`, or `ResponseBody`"),
                )),
            );
        }
        TypeIr::Function { .. } | TypeIr::Record { .. } => {}
    }
}

/// Builds a diagnostic for a body type the emitter cannot serialize safely.
fn unsupported_body_type(class: &ClassIr, method: &MethodIr, param: &MethodParamIr) -> Diagnostic {
    Diagnostic::error(format!(
        "parameter `{}` on `{}.{}` uses `@Body()` with an unsupported body type",
        param.name, class.name, method.name
    ))
    .with_label(label(
        param.span,
        "use a primitive, `Map`, `List`, `dynamic`, or a model with `toJson()`",
    ))
}

/// Returns one workspace string-set as a sorted borrowed slice.
fn workspace_string_slice<'a>(workspace: Option<&'a WorkspaceAnalysis>, key: &str) -> &'a [String] {
    workspace
        .and_then(|analysis| analysis.string_set(key))
        .unwrap_or_default()
}

/// Returns whether a sorted string slice contains `needle`.
fn contains_sorted_string(values: &[String], needle: &str) -> bool {
    values
        .binary_search_by(|value| value.as_str().cmp(needle))
        .is_ok()
}

/// Returns the short Dart type name after dropping any import prefix.
fn short_name(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

/// Returns whether a lowered class asks Dust to generate serialization support.
fn wants_serialize(class: &ClassIr) -> bool {
    class
        .traits
        .iter()
        .any(|item| item.symbol.0 == "dust_dart::Serialize")
}

/// Returns whether a lowered class declares a usable instance `toJson()` method.
fn has_to_json_method(class: &ClassIr) -> bool {
    class.methods.iter().any(|method| {
        method.name == "toJson"
            && !method.is_static
            && method.params.is_empty()
            && is_json_map_type(&method.return_type)
    })
}

/// Returns whether a lowered class declares a usable factory `fromJson(...)` constructor.
fn has_from_json_factory(class: &ClassIr) -> bool {
    class.constructors.iter().any(|constructor| {
        constructor.name.as_deref() == Some("fromJson")
            && constructor.is_factory
            && constructor.params.len() == 1
            && constructor.params[0].kind == ParamKind::Positional
            && is_json_map_type(&constructor.params[0].ty)
    })
}

/// Returns whether a lowered type is a string-keyed JSON object map.
fn is_json_map_type(ty: &TypeIr) -> bool {
    type_name_is(ty, DART_MAP)
        && ty.args().len() == 2
        && ty.args()[0].is_builtin(BuiltinType::String)
}

/// Returns whether a parsed class asks Dust to generate serialization support.
fn class_has_serialize(class: &ParsedClassSurface) -> bool {
    annotations_have_trait(&class.annotations, "Serialize")
}

/// Returns whether parsed annotations include a direct trait or `@Derive([...])` member.
fn annotations_have_trait(annotations: &[ParsedAnnotation], trait_name: &str) -> bool {
    annotations.iter().any(|annotation| {
        annotation.is_named(trait_name)
            || (annotation.is_named("Derive")
                && annotation
                    .positional_constructor_names()
                    .iter()
                    .any(|name| name == trait_name))
    })
}

/// Returns whether a parsed class declares a usable instance `toJson()` method.
fn parsed_has_to_json_method(class: &ParsedClassSurface) -> bool {
    class.methods.iter().any(|method| {
        method.name == "toJson"
            && !method.is_static
            && method.params.is_empty()
            && parsed_is_json_map_type(method.parsed_return_type.as_ref())
    })
}

/// Returns whether a parsed class declares a usable factory `fromJson(...)` constructor.
fn parsed_has_from_json_factory(class: &ParsedClassSurface) -> bool {
    class.constructors.iter().any(|constructor| {
        constructor.name.as_deref() == Some("fromJson")
            && constructor.is_factory
            && constructor.params.len() == 1
            && constructor.params[0].kind == ParameterKind::Positional
            && parsed_is_json_map_type(constructor.params[0].parsed_type.as_ref())
    })
}

/// Returns whether a parsed type is a string-keyed JSON object map.
fn parsed_is_json_map_type(ty: Option<&ParsedTypeSurface>) -> bool {
    let Some(ty) = ty else {
        return false;
    };
    let ParsedTypeKind::Named { name, args } = &ty.kind else {
        return false;
    };

    name == DART_MAP && args.len() == 2 && parsed_is_string_type(&args[0])
}

/// Returns whether a parsed type is `String`.
fn parsed_is_string_type(ty: &ParsedTypeSurface) -> bool {
    matches!(&ty.kind, ParsedTypeKind::Builtin(name) if name == "String")
}
