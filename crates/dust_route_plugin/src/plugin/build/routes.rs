use std::collections::HashSet;

use dust_dart_emit::{DART_BOOL, DART_DOUBLE, DART_INT, DART_STRING};
use dust_ir::{BuiltinType, ClassIr, ConstructorIr, ParamKind, TypeIr};
use dust_plugin_api::SymbolPlan;

use crate::plugin::{
    constants::ROUTES_ANALYSIS_KEY,
    model::{RouteFact, RouteParamSpec, RouteSpec},
    parse::{parse_route_config, route_config},
};

pub(super) fn build_route_spec(class: &ClassIr) -> Option<RouteSpec> {
    let config = route_config(&class.configs)?;
    let annotation = parse_route_config(config)?;
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
        annotation,
        params,
        import_uri: None,
        imports: Vec::new(),
    })
}

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

pub(super) fn route_constructor(class: &ClassIr) -> Option<&ConstructorIr> {
    class
        .constructors
        .iter()
        .find(|constructor| constructor.name.is_none() && !constructor.is_factory)
}

pub(super) fn parse_route_type_name(source: Option<&str>) -> Option<String> {
    let raw = source?.trim().trim_end_matches('?').trim();
    let base = raw.split('<').next().unwrap_or(raw).trim();
    if base.is_empty() {
        None
    } else {
        Some(base.to_owned())
    }
}

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
        annotation: fact.annotation,
        params,
        import_uri: Some(fact.import_uri),
        imports: fact.imports,
    })
}

fn parse_url_type(source: Option<&str>) -> Option<TypeIr> {
    let raw = source?.trim();
    let (name, nullable) = raw
        .strip_suffix('?')
        .map_or((raw, false), |stripped| (stripped.trim(), true));
    let kind = match name {
        DART_STRING => BuiltinType::String,
        DART_INT => BuiltinType::Int,
        DART_DOUBLE => BuiltinType::Double,
        DART_BOOL => BuiltinType::Bool,
        _ => return None,
    };
    Some(TypeIr::Builtin { kind, nullable })
}

fn path_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter_map(|segment| segment.strip_prefix(':'))
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

fn derive_route_name(class_name: &str) -> String {
    let stem = class_name
        .strip_suffix("Page")
        .or_else(|| class_name.strip_suffix("Screen"))
        .or_else(|| class_name.strip_suffix("View"))
        .unwrap_or(class_name);
    lower_camel(stem)
}

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

fn lower_camel(value: &str) -> String {
    let upper = upper_camel(value);
    let mut chars = upper.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => upper,
    }
}
