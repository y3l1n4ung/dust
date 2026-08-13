use std::collections::HashSet;

use dust_dart_emit::{DART_BOOL, DART_DOUBLE, DART_INT, DART_STRING};
use dust_ir::{BuiltinType, ClassIr, ConstructorIr, ParamKind, TypeIr};
use dust_plugin_api::SymbolPlan;

use crate::plugin::{
    constants::ROUTES_ANALYSIS_KEY,
    model::{RouteFact, RouteParamSpec, RouteSpec},
    parse::{route_annotation, route_config},
};

/// Builds a local route spec from a lowered page class.
pub(super) fn build_route_spec(class: &ClassIr) -> Option<RouteSpec> {
    let annotation = route_annotation(route_config(&class.configs)?);
    let name = annotation
        .name
        .clone()
        .unwrap_or_else(|| derive_route_name(&class.name));
    let route_class = format!("{}Route", upper_camel(&name));
    let constructor = route_constructor(class);
    let path_params = path_params(&annotation.path);
    let params = constructor
        .map(|constructor| {
            constructor
                .params
                .iter()
                .filter(|param| param.name != "key")
                .map(|param| RouteParamSpec {
                    name: param.name.clone(),
                    ty: param.ty.clone(),
                    is_path: path_params.contains(&param.name),
                    is_named: matches!(param.kind, ParamKind::Named),
                    has_default: param.has_default,
                    default_value_source: param.default_value_source.clone(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Some(RouteSpec {
        page_class: class.name.clone(),
        route_class,
        path: annotation.path.clone(),
        name,
        result_type: annotation
            .result_type
            .clone()
            .unwrap_or_else(|| "void".to_owned()),
        annotation,
        params,
        import_uri: None,
        imports: Vec::new(),
    })
}

/// Builds route specs from workspace analysis facts outside the current library.
pub(super) fn workspace_route_specs(
    plan: &SymbolPlan,
    local_pages: &HashSet<String>,
) -> Vec<RouteSpec> {
    plan.workspace_string_set(ROUTES_ANALYSIS_KEY)
        .unwrap_or_default()
        .iter()
        .filter_map(|value| serde_json::from_str::<RouteFact>(value).ok())
        .filter(|fact| !local_pages.contains(&fact.class_name))
        .filter_map(build_route_spec_from_fact)
        .collect()
}

/// Returns the unnamed generative constructor used by route pages.
pub(super) fn route_constructor(class: &ClassIr) -> Option<&ConstructorIr> {
    class
        .constructors
        .iter()
        .find(|constructor| constructor.name.is_none() && !constructor.is_factory)
}

/// Parses a simple route parameter type name from Dart source.
pub(super) fn parse_route_type_name(source: Option<&str>) -> Option<String> {
    let raw = source?.trim().trim_end_matches('?').trim();
    let base = raw.split('<').next().unwrap_or(raw).trim();
    if base.is_empty() {
        None
    } else {
        Some(base.to_owned())
    }
}

/// Builds a route spec from a serialized workspace route fact.
fn build_route_spec_from_fact(fact: RouteFact) -> Option<RouteSpec> {
    let name = fact
        .name
        .clone()
        .unwrap_or_else(|| derive_route_name(&fact.class_name));
    let route_class = format!("{}Route", upper_camel(&name));
    let path_params = path_params(&fact.path);
    let params = fact
        .params
        .iter()
        .map(|param| {
            Some(RouteParamSpec {
                name: param.name.clone(),
                ty: parse_url_type(param.type_source.as_deref())?,
                is_path: path_params.contains(&param.name),
                is_named: param.is_named,
                has_default: param.has_default,
                default_value_source: param.default_value_source.clone(),
            })
        })
        .collect::<Option<Vec<_>>>()?;

    Some(RouteSpec {
        page_class: fact.class_name,
        route_class,
        path: fact.path,
        name: name.clone(),
        result_type: fact
            .annotation
            .result_type
            .clone()
            .unwrap_or_else(|| "void".to_owned()),
        annotation: fact.annotation,
        params,
        import_uri: Some(fact.import_uri),
        imports: fact.imports,
    })
}

/// Parses a URL-supported route parameter type from source text.
fn parse_url_type(source: Option<&str>) -> Option<TypeIr> {
    let raw = source?.trim();
    let (name, nullable) = raw
        .strip_suffix('?')
        .map_or((raw, false), |stripped| (stripped.trim(), true));
    let ty = if let Some(inner) = name
        .strip_prefix("List<")
        .and_then(|value| value.strip_suffix('>'))
    {
        TypeIr::list_of(parse_url_type(Some(inner.trim()))?)
    } else {
        match name {
            DART_STRING => TypeIr::builtin(BuiltinType::String),
            DART_INT => TypeIr::builtin(BuiltinType::Int),
            DART_DOUBLE => TypeIr::builtin(BuiltinType::Double),
            DART_BOOL => TypeIr::builtin(BuiltinType::Bool),
            "DateTime" | "Uri" => TypeIr::named(name),
            value if is_route_named_type(value) => TypeIr::named(value),
            _ => return None,
        }
    };
    Some(if nullable { ty.nullable() } else { ty })
}

/// Returns true for a named type source that generated route code can reference.
fn is_route_named_type(value: &str) -> bool {
    let mut parts = value.split('.');
    parts
        .next_back()
        .and_then(|name| name.chars().next())
        .is_some_and(|ch| ch.is_ascii_uppercase())
        && value.split('.').all(is_simple_dart_identifier)
}

/// Returns true when the source fragment is a simple Dart identifier.
fn is_simple_dart_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

/// Extracts `:param` placeholders from a route path.
fn path_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter_map(|segment| segment.strip_prefix(':'))
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Derives a route action name from a page class name.
fn derive_route_name(class_name: &str) -> String {
    let stem = class_name
        .strip_suffix("Page")
        .or_else(|| class_name.strip_suffix("Screen"))
        .or_else(|| class_name.strip_suffix("View"))
        .unwrap_or(class_name);
    lower_camel(stem)
}

/// Converts a snake, kebab, or spaced name to UpperCamelCase.
fn upper_camel(value: &str) -> String {
    value
        .split(|ch: char| ch == '_' || ch == '-' || ch.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

/// Converts a name to lowerCamelCase.
fn lower_camel(value: &str) -> String {
    let upper = upper_camel(value);
    let mut chars = upper.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => upper,
    }
}
