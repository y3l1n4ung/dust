use dust_ir::{
    AnnotationValueIr, ConfigApplicationIr, NormalizedConfigIr, RouteConfigIr, RouterConfigIr,
};

/// Normalizes class-level `AppRoute` configuration after symbol resolution.
pub(crate) fn normalize_route(configs: &mut [ConfigApplicationIr]) {
    for config in configs {
        let snapshot = config.clone();
        let normalized = match snapshot.symbol.0.as_str() {
            "dust_flutter::AppRoute" => route_config(&snapshot).map(NormalizedConfigIr::Route),
            "dust_flutter::AppRouter" => router_config(&snapshot).map(NormalizedConfigIr::Router),
            _ => None,
        };
        if let Some(normalized) = normalized {
            config.normalized = Some(normalized);
        }
    }
}

/// Builds typed route configuration from one resolved application.
fn route_config(config: &ConfigApplicationIr) -> Option<RouteConfigIr> {
    let path = string_value(config.positional_argument_value(0))?.to_owned();

    Some(RouteConfigIr {
        path,
        name: string_value(config.named_argument_value("name")).map(str::to_owned),
        shell: member_value(config.named_argument_value("shell")).map(str::to_owned),
        result_type: member_value(config.named_argument_value("result")).map(str::to_owned),
        guards: member_list(config.named_argument_value("guards")),
        guards_configured: config.named_args.contains_key("guards"),
        transition: config
            .named_argument_source("transition")
            .map(normalize_transition_source),
        fullscreen_dialog: bool_value(config.named_argument_value("fullscreenDialog"))
            .unwrap_or(false),
        maintain_state: bool_value(config.named_argument_value("maintainState")).unwrap_or(true),
    })
}

/// Builds typed router configuration from one resolved application.
fn router_config(config: &ConfigApplicationIr) -> Option<RouterConfigIr> {
    Some(RouterConfigIr {
        initial: string_value(config.named_argument_value("initial")).map(str::to_owned),
        not_found: string_value(config.named_argument_value("notFound")).map(str::to_owned),
    })
}

/// Returns a structured string value.
fn string_value(value: Option<&AnnotationValueIr>) -> Option<&str> {
    match value? {
        AnnotationValueIr::String(value) => Some(value),
        _ => None,
    }
}

/// Returns a structured member reference with its source prefix preserved.
fn member_value(value: Option<&AnnotationValueIr>) -> Option<&str> {
    match value? {
        AnnotationValueIr::Member(name) => Some(&name.source),
        _ => None,
    }
}

/// Returns structured member references from a list value.
fn member_list(value: Option<&AnnotationValueIr>) -> Vec<String> {
    match value {
        Some(AnnotationValueIr::List(values)) => values
            .iter()
            .filter_map(|value| member_value(Some(value)).map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    }
}

/// Returns a structured boolean value.
fn bool_value(value: Option<&AnnotationValueIr>) -> Option<bool> {
    match value? {
        AnnotationValueIr::Bool(value) => Some(*value),
        _ => None,
    }
}

/// Removes syntax and known import prefixes not needed by generated route code.
fn normalize_transition_source(source: &str) -> String {
    let source = source
        .trim()
        .strip_prefix("const ")
        .unwrap_or(source.trim());
    for prefix in ["cupertino.", "material."] {
        if let Some(stripped) = source.strip_prefix(prefix) {
            return stripped.to_owned();
        }
    }
    source.to_owned()
}
