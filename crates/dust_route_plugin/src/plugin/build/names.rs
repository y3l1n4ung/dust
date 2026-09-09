//! Names the generated router uses, and the routes they are derived from.

use super::*;

/// Generated public names derived from the handwritten router class.
pub(super) struct RouterGeneratedNames {
    /// Generated base class extended by the handwritten router.
    pub(super) generated_base_class: String,
    /// Generated sealed route base class.
    pub(super) route_base_class: String,
    /// Generated route metadata tree variable.
    pub(super) routes_variable: String,
    /// Generated URI parser function.
    pub(super) parse_route_function: String,
    /// Generated route location helper function.
    pub(super) route_location_function: String,
    /// Generated route authentication helper function.
    pub(super) route_requires_auth_function: String,
    /// Generated route branch lookup function.
    pub(super) route_branch_function: String,
    /// Generated route debug metadata helper function.
    pub(super) route_debug_info_function: String,
    /// Generated guard factory lookup function.
    pub(super) route_guards_function: String,
    /// Generated route page builder function.
    pub(super) build_page_function: String,
    /// Generated stack restoration function.
    pub(super) restore_stack_function: String,
    /// Generated BuildContext extension name.
    pub(super) context_extension: String,
    /// Generated typed navigator facade class.
    pub(super) navigator_class: String,
    /// Generated route action wrapper class.
    pub(super) route_action_class: String,
}

/// Builds router-scoped names that are easier to remember and less collision-prone.
pub(super) fn router_generated_names(router_class: &str) -> RouterGeneratedNames {
    let stem = router_class.strip_suffix("Router").unwrap_or(router_class);
    let lower = lower_camel(stem);
    RouterGeneratedNames {
        generated_base_class: format!("${router_class}"),
        route_base_class: format!("{stem}Route"),
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

impl RouterGeneratedNames {
    /// Returns public generated names that share the Dart declaration namespace.
    pub(super) fn generated_route_names(&self) -> GeneratedRouteNames<'_> {
        GeneratedRouteNames {
            generated_base_class: &self.generated_base_class,
            route_base_class: &self.route_base_class,
            context_extension: &self.context_extension,
            navigator_class: &self.navigator_class,
            route_action_class: &self.route_action_class,
        }
    }
}

/// Returns classes annotated with `@AppRouter` in the current library.
pub(super) fn router_classes(library: &DartFileIr) -> Vec<&ClassIr> {
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
pub(super) fn local_and_workspace_routes(
    library: &DartFileIr,
    plan: &SymbolPlan,
) -> Vec<RouteSpec> {
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
pub(super) fn workspace_router_count(plan: &SymbolPlan) -> usize {
    plan.workspace_string_set(ROUTERS_ANALYSIS_KEY)
        .unwrap_or_default()
        .len()
}

/// Returns class names discovered by route workspace analysis.
pub(super) fn workspace_classes(library: &DartFileIr, plan: &SymbolPlan) -> HashSet<String> {
    let mut classes = library
        .classes
        .iter()
        .map(|class| class.name.clone())
        .collect::<HashSet<_>>();
    classes.extend(
        plan.workspace_string_set(GUARDS_ANALYSIS_KEY)
            .unwrap_or_default()
            .iter()
            .filter_map(|value| serde_json::from_str::<GuardFact>(value).ok())
            .map(|fact| fact.class_name),
    );
    classes
}

/// Resolves an annotation path to the generated route class for router settings.
pub(super) fn route_class_for_path(
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
pub(super) fn validate_not_found_route(
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
pub(super) fn router_fields(router_class: &ClassIr) -> Vec<RouterFieldSpec> {
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
pub(super) fn discover_refresh_listenable(
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
pub(super) fn is_listenable_type(name: &str) -> bool {
    matches!(name, "Listenable" | "ChangeNotifier" | "ValueNotifier") || name.ends_with("ViewModel")
}
