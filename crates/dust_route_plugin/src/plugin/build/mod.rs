use std::collections::HashSet;

use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, LibraryIr};
use dust_plugin_api::SymbolPlan;

use super::{
    constants::ROUTERS_ANALYSIS_KEY,
    model::{RouteSpec, RouterFieldSpec, RouterSpec},
    parse::parse_router_config,
};

mod guards;
mod routes;

use guards::build_guard_specs;
use routes::{build_route_spec, workspace_route_specs};

pub(crate) fn build_router_spec(
    library: &LibraryIr,
    plan: &SymbolPlan,
) -> Result<Option<RouterSpec>, Vec<Diagnostic>> {
    let router_classes = router_classes(library);
    let Some(router_class) = router_classes.first().copied() else {
        return Ok(None);
    };
    if router_classes.len() > 1 || workspace_router_count(plan) > 1 {
        return Err(vec![Diagnostic::error(
            "exactly one `@Router` is allowed in a Dust route workspace",
        )]);
    }

    let router_config = router_class
        .configs
        .iter()
        .find(|config| config.symbol.0.ends_with("::Router"));
    let router_annotation = parse_router_config(router_config);
    let mut routes = local_and_workspace_routes(library, plan);
    routes.sort_by(|a, b| a.path.cmp(&b.path).then_with(|| a.name.cmp(&b.name)));

    if routes.is_empty() {
        return Err(vec![Diagnostic::error(format!(
            "router `{}` needs at least one `@Route` page in the workspace for current route generation",
            router_class.name
        ))]);
    }

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
        generated_base_class: format!("${}", router_class.name),
        initial_route_class,
        not_found_route_class: Some(not_found_route_class),
        refresh_listenable,
        guard_specs,
        routes,
    }))
}

fn router_classes(library: &LibraryIr) -> Vec<&ClassIr> {
    library
        .classes
        .iter()
        .filter(|class| {
            class
                .configs
                .iter()
                .any(|config| config.symbol.0.ends_with("::Router"))
        })
        .collect()
}

fn local_and_workspace_routes(library: &LibraryIr, plan: &SymbolPlan) -> Vec<RouteSpec> {
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

fn workspace_router_count(plan: &SymbolPlan) -> usize {
    plan.workspace_string_set(ROUTERS_ANALYSIS_KEY)
        .unwrap_or_default()
        .len()
}

fn validate_workspace_route_set(routes: &[RouteSpec]) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let mut paths = HashSet::new();
    let mut names = HashSet::new();
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
        for param in &route.params {
            if !param.is_path && !param.ty.is_nullable() && !param.has_default {
                diagnostics.push(Diagnostic::error(format!(
                    "route query parameter `{}` on `{}` must be nullable or have a default value",
                    param.name, route.page_class
                )));
            }
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

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
                "router `{router_class}` {label} path `{path}` does not match any discovered `@Route` path"
            ))]
        })
}

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

fn is_listenable_type(name: &str) -> bool {
    matches!(name, "Listenable" | "ChangeNotifier" | "ValueNotifier") || name.ends_with("ViewModel")
}
