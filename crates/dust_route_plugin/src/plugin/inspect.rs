use dust_plugin_api::LibraryAnalysisSnapshot;

use super::{
    constants::ROUTES_ANALYSIS_KEY,
    model::{RouteAnnotation, RouteFact},
};

/// One route row shown by route inspection tooling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteTableRow {
    /// Effective route name.
    pub name: String,
    /// Absolute route path.
    pub path: String,
    /// Flutter page class.
    pub page: String,
    /// Effective shell widget class, including inherited shells.
    pub shell: Option<String>,
    /// Effective branch name, including inherited branches.
    pub branch: Option<String>,
    /// Guard class names applied directly to this route.
    pub guards: Vec<String>,
    /// Route push result type.
    pub result_type: String,
}

/// One route node shown by route graph inspection tooling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteGraphNode {
    /// Effective route name.
    pub name: String,
    /// Absolute route path.
    pub path: String,
    /// Parent route path, or absent for a root route.
    pub parent_path: Option<String>,
    /// Flutter page class.
    pub page: String,
    /// Effective shell widget class, including inherited shells.
    pub shell: Option<String>,
    /// Effective branch name, including inherited branches.
    pub branch: Option<String>,
    /// Guard class names applied directly to this route.
    pub guards: Vec<String>,
}

/// Builds deterministic route table rows from workspace analysis snapshots.
pub fn route_table_rows(snapshots: &[LibraryAnalysisSnapshot]) -> Vec<RouteTableRow> {
    let facts = route_facts(snapshots);

    facts
        .iter()
        .map(|fact| RouteTableRow {
            name: route_name(fact),
            path: fact.path.clone(),
            page: fact.class_name.clone(),
            shell: effective_shell(fact, &facts).map(str::to_owned),
            branch: effective_branch(fact, &facts).map(str::to_owned),
            guards: fact.annotation.guards.clone(),
            result_type: result_type(&fact.annotation),
        })
        .collect()
}

/// Builds deterministic route graph nodes from workspace analysis snapshots.
pub fn route_graph_nodes(snapshots: &[LibraryAnalysisSnapshot]) -> Vec<RouteGraphNode> {
    let facts = route_facts(snapshots);

    facts
        .iter()
        .map(|fact| RouteGraphNode {
            name: route_name(fact),
            path: fact.path.clone(),
            parent_path: nearest_parent(fact, &facts).map(|parent| parent.path.clone()),
            page: fact.class_name.clone(),
            shell: effective_shell(fact, &facts).map(str::to_owned),
            branch: effective_branch(fact, &facts).map(str::to_owned),
            guards: fact.annotation.guards.clone(),
        })
        .collect()
}

/// Returns deterministic route facts from workspace analysis snapshots.
fn route_facts(snapshots: &[LibraryAnalysisSnapshot]) -> Vec<RouteFact> {
    let mut facts = snapshots
        .iter()
        .filter_map(|snapshot| snapshot.string_set(ROUTES_ANALYSIS_KEY))
        .flatten()
        .filter_map(|value| serde_json::from_str::<RouteFact>(value).ok())
        .collect::<Vec<_>>();
    facts.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then_with(|| route_name(a).cmp(&route_name(b)))
            .then_with(|| a.class_name.cmp(&b.class_name))
    });
    facts
}

/// Returns the explicit route name or the generated fallback name.
fn route_name(fact: &RouteFact) -> String {
    fact.name
        .clone()
        .unwrap_or_else(|| derive_route_name(&fact.class_name))
}

/// Returns the explicit route result type or the generated default.
fn result_type(annotation: &RouteAnnotation) -> String {
    annotation
        .result_type
        .clone()
        .unwrap_or_else(|| "void".to_owned())
}

/// Returns the shell applied to this route after inheritance.
fn effective_shell<'a>(route: &'a RouteFact, routes: &'a [RouteFact]) -> Option<&'a str> {
    route
        .annotation
        .shell
        .as_deref()
        .or_else(|| inherited(route, routes, |annotation| annotation.shell.as_deref()))
}

/// Returns the branch applied to this route after inheritance.
fn effective_branch<'a>(route: &'a RouteFact, routes: &'a [RouteFact]) -> Option<&'a str> {
    route
        .annotation
        .branch
        .as_deref()
        .or_else(|| inherited(route, routes, |annotation| annotation.branch.as_deref()))
}

/// Returns the nearest route whose path is a segment-wise parent.
fn nearest_parent<'a>(route: &RouteFact, routes: &'a [RouteFact]) -> Option<&'a RouteFact> {
    let current_segments = route_segments(&route.path);
    routes
        .iter()
        .filter(|candidate| candidate.path != route.path)
        .filter_map(|candidate| {
            let candidate_segments = route_segments(&candidate.path);
            (candidate_segments.len() < current_segments.len()
                && current_segments.starts_with(&candidate_segments))
            .then_some((candidate_segments.len(), candidate))
        })
        .max_by_key(|(length, _)| *length)
        .map(|(_, candidate)| candidate)
}

/// Returns the nearest inherited annotation value from a parent path.
fn inherited<'a>(
    route: &RouteFact,
    routes: &'a [RouteFact],
    value: impl Fn(&'a RouteAnnotation) -> Option<&'a str>,
) -> Option<&'a str> {
    let current_segments = route_segments(&route.path);
    routes
        .iter()
        .filter(|candidate| candidate.path != route.path)
        .filter_map(|candidate| {
            let candidate_value = value(&candidate.annotation)?;
            let candidate_segments = route_segments(&candidate.path);
            (candidate_segments.len() < current_segments.len()
                && current_segments.starts_with(&candidate_segments))
            .then_some((candidate_segments.len(), candidate_value))
        })
        .max_by_key(|(length, _)| *length)
        .map(|(_, value)| value)
}

/// Splits an absolute route path into non-empty path segments.
fn route_segments(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

/// Derives the same fallback route helper name used by generation.
fn derive_route_name(class_name: &str) -> String {
    let stem = class_name
        .strip_suffix("Page")
        .or_else(|| class_name.strip_suffix("Screen"))
        .or_else(|| class_name.strip_suffix("View"))
        .unwrap_or(class_name);
    lower_camel(stem)
}

/// Converts one identifier-like string to lower camel case.
fn lower_camel(value: &str) -> String {
    let upper = upper_camel(value);
    let mut chars = upper.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => upper,
    }
}

/// Converts snake, kebab, or spaced text to upper camel case.
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
