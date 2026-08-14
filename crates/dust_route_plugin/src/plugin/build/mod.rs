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
/// Validates generated Dart identifiers.
mod identifiers;
/// Builds local and workspace route specs.
mod routes;
/// Validates route sets before generation.
mod validation;

use guards::build_guard_specs;
use identifiers::lower_camel;
use routes::{build_route_spec, workspace_route_specs};
use validation::validate_workspace_route_set;

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
        generated_base_class: format!("${router_class}"),
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
