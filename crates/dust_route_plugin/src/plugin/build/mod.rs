use std::collections::HashSet;

use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, DartFileIr};
use dust_plugin_api::SymbolPlan;

use super::{
    constants::{GUARDS_ANALYSIS_KEY, ROUTER, ROUTERS_ANALYSIS_KEY},
    model::{GuardFact, RouteSpec, RouterAnnotation, RouterFieldSpec, RouterSpec},
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
use validation::{GeneratedRouteNames, validate_workspace_route_set};

/// Names the generated router uses, and the routes they are derived from.
mod names;
use self::names::*;

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
    validate_workspace_route_set(
        &routes,
        &names.generated_route_names(),
        &workspace_classes(library, plan),
    )?;
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
        route_base_class: names.route_base_class,
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
