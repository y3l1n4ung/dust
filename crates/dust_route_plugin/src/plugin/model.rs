use dust_ir::TypeIr;
use serde::{Deserialize, Serialize};

/// One route page fact collected during workspace analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RouteFact {
    pub(crate) class_name: String,
    pub(crate) path: String,
    pub(crate) name: Option<String>,
    pub(crate) annotation: RouteAnnotation,
    pub(crate) import_uri: String,
    pub(crate) source_path: String,
    pub(crate) imports: Vec<String>,
    pub(crate) params: Vec<RouteParamFact>,
}

/// One route constructor parameter fact collected during workspace analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RouteParamFact {
    pub(crate) name: String,
    pub(crate) type_source: Option<String>,
    pub(crate) is_named: bool,
    pub(crate) has_default: bool,
    pub(crate) default_value_source: Option<String>,
}

/// One router root fact collected during workspace analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RouterFact {
    pub(crate) class_name: String,
    pub(crate) initial: Option<String>,
    pub(crate) not_found: Option<String>,
}

/// Parsed route annotation values used by validation and emission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RouteAnnotation {
    pub(crate) path: String,
    pub(crate) name: Option<String>,
    pub(crate) shell: Option<String>,
    pub(crate) guards: Vec<String>,
    pub(crate) guards_configured: bool,
    pub(crate) transition: Option<String>,
    pub(crate) fullscreen_dialog: bool,
    pub(crate) maintain_state: bool,
}

/// Parsed router annotation values used by validation and emission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RouterAnnotation {
    pub(crate) initial: Option<String>,
    pub(crate) not_found: Option<String>,
}

/// One route constructor parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RouteParamSpec {
    pub(crate) name: String,
    pub(crate) ty: TypeIr,
    pub(crate) is_path: bool,
    pub(crate) is_named: bool,
    pub(crate) has_default: bool,
    pub(crate) default_value_source: Option<String>,
}

/// One route available to the generated router.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RouteSpec {
    pub(crate) page_class: String,
    pub(crate) route_class: String,
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) annotation: RouteAnnotation,
    pub(crate) params: Vec<RouteParamSpec>,
    pub(crate) import_uri: Option<String>,
    pub(crate) imports: Vec<String>,
}

/// One router root generation spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RouterSpec {
    pub(crate) router_class: String,
    pub(crate) generated_base_class: String,
    pub(crate) initial_route_class: String,
    pub(crate) not_found_route_class: Option<String>,
    pub(crate) routes: Vec<RouteSpec>,
    pub(crate) guard_classes: Vec<String>,
}
