use std::collections::HashSet;

use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, DartFileIr};
use dust_plugin_api::SymbolPlan;

use super::{
    constants::{ROUTER, ROUTERS_ANALYSIS_KEY},
    model::{RouteSpec, RouterAnnotation, RouterFieldSpec, RouterSpec},
    parse::router_config,
};

/// Builds guard specs and router-field injections.
mod guards;
/// Builds local and workspace route specs.
mod routes;

use guards::build_guard_specs;
use routes::{build_route_spec, workspace_route_specs};

/// Builds the final router spec for a library containing the workspace router.
pub(crate) fn build_router_spec(
    library: &DartFileIr,
    plan: &SymbolPlan,
) -> Result<Option<RouterSpec>, Vec<Diagnostic>> {
    let router_classes = router_classes(library);
    let Some(router_class) = router_classes.first().copied() else {
        return Ok(None);
    };
    if router_classes.len() > 1 || workspace_router_count(plan) > 1 {
        return Err(vec![Diagnostic::error(
            "exactly one `@AppRouter` is allowed in a Dust route workspace",
        )]);
    }

    let router_annotation = router_config(&router_class.configs)
        .map(|config| RouterAnnotation {
            initial: config.initial.clone(),
            not_found: config.not_found.clone(),
        })
        .unwrap_or(RouterAnnotation {
            initial: None,
            not_found: None,
        });
    let mut routes = local_and_workspace_routes(library, plan);
    routes.sort_by(|a, b| a.path.cmp(&b.path).then_with(|| a.name.cmp(&b.name)));

    if routes.is_empty() {
        return Err(vec![Diagnostic::error(format!(
            "router `{}` needs at least one `@AppRoute` page in the workspace for current route generation",
            router_class.name
        ))]);
    }

    let names = router_generated_names(&router_class.name);
    apply_router_route_prefix(&mut routes, &names.route_class_prefix);
    validate_workspace_route_set(&routes)?;
    let initial_route_class = route_class_for_path(
        &routes,
        router_annotation.initial.as_deref(),
        &router_class.name,
        "initial",
    )?;
    let not_found_route_class = route_class_for_path(
        &routes,
        router_annotation.not_found.as_deref(),
        &router_class.name,
        "notFound",
    )?;
    validate_not_found_route(&routes, &not_found_route_class)?;

    let router_fields = router_fields(router_class);
    let refresh_listenable = discover_refresh_listenable(&router_fields)?;
    let guard_specs = build_guard_specs(library, plan, &routes, &router_fields)?;

    Ok(Some(RouterSpec {
        router_class: router_class.name.clone(),
        generated_base_class: names.generated_base_class,
        route_path_class: names.route_path_class,
        routes_variable: names.routes_variable,
        parse_route_function: names.parse_route_function,
        route_location_function: names.route_location_function,
        route_requires_auth_function: names.route_requires_auth_function,
        route_branch_function: names.route_branch_function,
        route_debug_info_function: names.route_debug_info_function,
        route_guards_function: names.route_guards_function,
        build_page_function: names.build_page_function,
        restore_stack_function: names.restore_stack_function,
        context_extension: names.context_extension,
        navigator_class: names.navigator_class,
        route_action_class: names.route_action_class,
        initial_route_class,
        not_found_route_class: Some(not_found_route_class),
        refresh_listenable,
        guard_specs,
        routes,
    }))
}

/// Generated public names derived from the handwritten router class.
struct RouterGeneratedNames {
    /// Prefix applied to every generated typed route data class.
    route_class_prefix: String,
    /// Generated base class extended by the handwritten router.
    generated_base_class: String,
    /// Generated sealed route path base class.
    route_path_class: String,
    /// Generated route metadata tree variable.
    routes_variable: String,
    /// Generated URI parser function.
    parse_route_function: String,
    /// Generated route location helper function.
    route_location_function: String,
    /// Generated route authentication helper function.
    route_requires_auth_function: String,
    /// Generated route branch lookup function.
    route_branch_function: String,
    /// Generated route debug metadata helper function.
    route_debug_info_function: String,
    /// Generated guard factory lookup function.
    route_guards_function: String,
    /// Generated route page builder function.
    build_page_function: String,
    /// Generated stack restoration function.
    restore_stack_function: String,
    /// Generated BuildContext extension name.
    context_extension: String,
    /// Generated typed navigator facade class.
    navigator_class: String,
    /// Generated route action wrapper class.
    route_action_class: String,
}

/// Builds router-scoped names that are easier to remember and less collision-prone.
fn router_generated_names(router_class: &str) -> RouterGeneratedNames {
    let stem = router_class.strip_suffix("Router").unwrap_or(router_class);
    let lower = lower_camel(stem);
    RouterGeneratedNames {
        route_class_prefix: stem.to_owned(),
        generated_base_class: format!("{router_class}Base"),
        route_path_class: format!("{stem}RoutePath"),
        routes_variable: format!("${lower}Routes"),
        parse_route_function: format!("parse{stem}Route"),
        route_location_function: format!("{lower}RouteLocation"),
        route_requires_auth_function: format!("{lower}RouteRequiresAuth"),
        route_branch_function: format!("{lower}RouteBranch"),
        route_debug_info_function: format!("{lower}RouteDebugInfo"),
        route_guards_function: format!("{lower}RouteGuards"),
        build_page_function: format!("build{stem}RoutePage"),
        restore_stack_function: format!("restore{stem}RouteStack"),
        context_extension: format!("{stem}RouterContext"),
        navigator_class: format!("{stem}RoutesNavigator"),
        route_action_class: format!("{stem}RouteAction"),
    }
}

/// Prefixes route classes with the router stem, unless already scoped.
fn apply_router_route_prefix(routes: &mut [RouteSpec], prefix: &str) {
    for route in routes {
        if !route.route_class.starts_with(prefix) {
            route.route_class = format!("{prefix}{}", route.route_class);
        }
    }
}

/// Returns classes annotated with `@AppRouter` in the current library.
fn router_classes(library: &DartFileIr) -> Vec<&ClassIr> {
    library
        .classes
        .iter()
        .filter(|class| {
            class
                .configs
                .iter()
                .any(|config| config.symbol.0.rsplit("::").next() == Some(ROUTER))
        })
        .collect()
}

/// Merges local route specs with workspace route facts from other files.
fn local_and_workspace_routes(library: &DartFileIr, plan: &SymbolPlan) -> Vec<RouteSpec> {
    let mut routes = library
        .classes
        .iter()
        .filter_map(build_route_spec)
        .collect::<Vec<_>>();
    let local_pages = routes
        .iter()
        .map(|route| route.page_class.clone())
        .collect::<HashSet<_>>();
    routes.extend(workspace_route_specs(plan, &local_pages));
    routes
}

/// Counts discovered routers across the workspace analysis set.
fn workspace_router_count(plan: &SymbolPlan) -> usize {
    plan.workspace_string_set(ROUTERS_ANALYSIS_KEY)
        .unwrap_or_default()
        .len()
}

/// Validates duplicate route paths and names.
fn validate_workspace_route_set(routes: &[RouteSpec]) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let mut paths = HashSet::new();
    let mut names = HashSet::new();
    let mut route_classes = HashSet::new();
    let mut helper_names = HashSet::new();
    for route in routes {
        if !paths.insert(route.path.clone()) {
            diagnostics.push(Diagnostic::error(format!(
                "duplicate route path `{}`",
                route.path
            )));
        }
        if !names.insert(route.name.clone()) {
            diagnostics.push(Diagnostic::error(format!(
                "duplicate route name `{}`",
                route.name
            )));
        }
        if !is_valid_dart_identifier(&route.name) || is_dart_reserved_word(&route.name) {
            diagnostics.push(Diagnostic::error(format!(
                "route name `{}` must be a valid non-reserved Dart identifier",
                route.name
            )));
        }
        if route.name == "pop" {
            diagnostics.push(Diagnostic::error(
                "route name `pop` conflicts with the generated navigator `pop` helper",
            ));
        }
        if !helper_names.insert(route.name.clone()) {
            diagnostics.push(Diagnostic::error(format!(
                "generated route helper `{}` is emitted more than once",
                route.name
            )));
        }
        if !route_classes.insert(route.route_class.clone()) {
            diagnostics.push(Diagnostic::error(format!(
                "generated route class `{}` is emitted by more than one route name",
                route.route_class
            )));
        }
        validate_duplicate_path_params(route, &mut diagnostics);
    }
    validate_ambiguous_path_siblings(routes, &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// Returns true when [name] can be emitted as a Dart identifier.
fn is_valid_dart_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_dart_identifier_start(first) && chars.all(is_dart_identifier_part)
}

/// Returns true when [ch] is allowed at the start of a Dart identifier.
fn is_dart_identifier_start(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphabetic()
}

/// Returns true when [ch] is allowed after the first Dart identifier character.
fn is_dart_identifier_part(ch: char) -> bool {
    is_dart_identifier_start(ch) || ch.is_ascii_digit()
}

/// Returns true for Dart reserved and contextual words that should not be
/// generated as route helper names.
fn is_dart_reserved_word(name: &str) -> bool {
    matches!(
        name,
        "abstract"
            | "as"
            | "assert"
            | "async"
            | "await"
            | "base"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "covariant"
            | "default"
            | "deferred"
            | "do"
            | "dynamic"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "extension"
            | "external"
            | "factory"
            | "false"
            | "final"
            | "finally"
            | "for"
            | "Function"
            | "get"
            | "hide"
            | "if"
            | "implements"
            | "import"
            | "in"
            | "interface"
            | "is"
            | "late"
            | "library"
            | "mixin"
            | "new"
            | "null"
            | "of"
            | "on"
            | "operator"
            | "part"
            | "required"
            | "rethrow"
            | "return"
            | "sealed"
            | "set"
            | "show"
            | "static"
            | "super"
            | "switch"
            | "sync"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typedef"
            | "var"
            | "void"
            | "when"
            | "while"
            | "with"
            | "yield"
    )
}

/// Rejects paths that bind the same `:param` name more than once.
fn validate_duplicate_path_params(route: &RouteSpec, diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = HashSet::new();
    let mut reported = HashSet::new();
    for segment in path_segments(&route.path) {
        let Some(param) = path_param_name(segment) else {
            continue;
        };
        if !seen.insert(param) && reported.insert(param) {
            diagnostics.push(Diagnostic::error(format!(
                "route `{}` path `{}` declares duplicate path parameter `:{param}`",
                route.page_class, route.path
            )));
        }
    }
}

/// Rejects same-length path patterns with overlapping static/dynamic segments.
fn validate_ambiguous_path_siblings(routes: &[RouteSpec], diagnostics: &mut Vec<Diagnostic>) {
    for (index, route) in routes.iter().enumerate() {
        let route_segments = path_segments(&route.path).collect::<Vec<_>>();
        for sibling in routes.iter().skip(index + 1) {
            let sibling_segments = path_segments(&sibling.path).collect::<Vec<_>>();
            let Some(parent) = ambiguous_parent(&route_segments, &sibling_segments) else {
                continue;
            };
            diagnostics.push(ambiguous_sibling_diagnostic(
                &sibling.path,
                &route.path,
                &parent,
            ));
        }
    }
}

/// Returns the shared parent path when sibling patterns can match the same URL.
fn ambiguous_parent(left: &[&str], right: &[&str]) -> Option<String> {
    if left.len() != right.len() {
        return None;
    }

    let mut first_static_dynamic_parent = None;
    for (index, (left_segment, right_segment)) in left.iter().zip(right).enumerate() {
        if left_segment == right_segment {
            continue;
        }
        let left_dynamic = path_param_name(left_segment).is_some();
        let right_dynamic = path_param_name(right_segment).is_some();
        if !left_dynamic && !right_dynamic {
            return None;
        }
        if left_dynamic != right_dynamic {
            first_static_dynamic_parent.get_or_insert_with(|| {
                display_parent_path(
                    &left[..index]
                        .iter()
                        .map(|segment| (*segment).to_owned())
                        .collect::<Vec<_>>(),
                )
            });
        }
    }

    first_static_dynamic_parent
}

/// Builds a diagnostic for ambiguous static and dynamic sibling segments.
fn ambiguous_sibling_diagnostic(route_path: &str, sibling_path: &str, parent: &str) -> Diagnostic {
    Diagnostic::error(format!(
        "route path `{route_path}` conflicts with sibling `{sibling_path}`; static and dynamic segments under `{parent}` are ambiguous"
    ))
}

/// Iterates over non-empty slash-delimited path segments.
fn path_segments(path: &str) -> impl Iterator<Item = &str> {
    path.split('/').filter(|segment| !segment.is_empty())
}

/// Returns the parameter name for a dynamic path segment.
fn path_param_name(segment: &str) -> Option<&str> {
    segment.strip_prefix(':').filter(|name| !name.is_empty())
}

/// Renders parent path segments with a leading slash.
fn display_parent_path(segments: &[String]) -> String {
    if segments.is_empty() {
        "/".to_owned()
    } else {
        format!("/{}", segments.join("/"))
    }
}

/// Resolves an annotation path to the generated route class for router settings.
fn route_class_for_path(
    routes: &[RouteSpec],
    path: Option<&str>,
    router_class: &str,
    label: &str,
) -> Result<String, Vec<Diagnostic>> {
    let Some(path) = path else {
        return Err(vec![Diagnostic::error(format!(
            "router `{router_class}` requires `{label}` path"
        ))]);
    };
    routes
        .iter()
        .find(|route| route.path == path)
        .map(|route| route.route_class.clone())
        .ok_or_else(|| {
            vec![Diagnostic::error(format!(
                "router `{router_class}` {label} path `{path}` does not match any discovered `@AppRoute` path"
            ))]
        })
}

/// Ensures the not-found route remains unconditional.
fn validate_not_found_route(
    routes: &[RouteSpec],
    route_class: &str,
) -> Result<(), Vec<Diagnostic>> {
    let Some(route) = routes.iter().find(|route| route.route_class == route_class) else {
        return Ok(());
    };
    if route.annotation.guards_configured && !route.annotation.guards.is_empty() {
        return Err(vec![Diagnostic::error(format!(
            "notFound route `{}` must not declare guards",
            route.page_class
        ))]);
    }
    Ok(())
}

/// Extracts router fields available for refresh and guard injection.
fn router_fields(router_class: &ClassIr) -> Vec<RouterFieldSpec> {
    router_class
        .fields
        .iter()
        .filter_map(|field| {
            Some(RouterFieldSpec {
                name: field.name.clone(),
                type_name: field.ty.name()?.to_owned(),
            })
        })
        .collect()
}

/// Finds a single Listenable-like router field for refresh notifications.
fn discover_refresh_listenable(
    fields: &[RouterFieldSpec],
) -> Result<Option<String>, Vec<Diagnostic>> {
    let candidates = fields
        .iter()
        .filter(|field| is_listenable_type(&field.type_name))
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [] => Ok(None),
        [field] => Ok(Some(field.name.clone())),
        _ => Err(vec![Diagnostic::error(
            "router has more than one Listenable-like field; keep exactly one refresh source",
        )]),
    }
}

/// Returns true when a router field type can refresh Navigator state.
fn is_listenable_type(name: &str) -> bool {
    matches!(name, "Listenable" | "ChangeNotifier" | "ValueNotifier") || name.ends_with("ViewModel")
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
