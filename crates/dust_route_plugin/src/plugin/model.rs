use dust_ir::TypeIr;
use serde::{Deserialize, Serialize};

/// One route page fact collected during workspace analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RouteFact {
    /// Page class annotated with `@Route`.
    pub(crate) class_name: String,
    /// Absolute route path from the annotation.
    pub(crate) path: String,
    /// Optional explicit route name from the annotation.
    pub(crate) name: Option<String>,
    /// Parsed annotation settings for the route.
    pub(crate) annotation: RouteAnnotation,
    /// Package import URI for the page class.
    pub(crate) import_uri: String,
    /// Source path that contributed the route fact.
    pub(crate) source_path: String,
    /// Imports required by generated code for this route.
    pub(crate) imports: Vec<String>,
    /// Constructor parameters available for route generation.
    pub(crate) params: Vec<RouteParamFact>,
}

/// One route constructor parameter fact collected during workspace analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RouteParamFact {
    /// Constructor parameter name.
    pub(crate) name: String,
    /// Original Dart type source when it can be recovered.
    pub(crate) type_source: Option<String>,
    /// Whether the parameter is named.
    pub(crate) is_named: bool,
    /// Whether the constructor parameter declares a default.
    pub(crate) has_default: bool,
    /// Source expression for the preserved default value.
    pub(crate) default_value_source: Option<String>,
}

/// One router root fact collected during workspace analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RouterFact {
    /// Router class annotated with `@Router`.
    pub(crate) class_name: String,
    /// Optional initial route path.
    pub(crate) initial: Option<String>,
    /// Optional not-found route path.
    pub(crate) not_found: Option<String>,
    /// Source path that contributed the router fact.
    pub(crate) source_path: String,
}

/// One guard class fact collected during workspace analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GuardFact {
    /// Guard class name.
    pub(crate) class_name: String,
    /// Whether the class has an unnamed generative constructor.
    pub(crate) has_unnamed_constructor: bool,
    /// Package import URI for the guard class.
    pub(crate) import_uri: String,
    /// Source path that contributed the guard fact.
    pub(crate) source_path: String,
    /// Constructor parameters available for dependency injection.
    pub(crate) params: Vec<GuardParamFact>,
}

/// One guard constructor parameter fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GuardParamFact {
    /// Constructor parameter name.
    pub(crate) name: String,
    /// Original Dart type source when it can be recovered.
    pub(crate) type_source: Option<String>,
    /// Whether the parameter is named.
    pub(crate) is_named: bool,
    /// Whether the constructor parameter declares a default.
    pub(crate) has_default: bool,
}

/// Parsed route annotation values used by validation and emission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RouteAnnotation {
    /// Absolute route path, including `:param` placeholders.
    pub(crate) path: String,
    /// Optional explicit route action name.
    pub(crate) name: Option<String>,
    /// Optional shell widget class wrapping the page.
    pub(crate) shell: Option<String>,
    /// Guard class names applied to the route.
    pub(crate) guards: Vec<String>,
    /// Whether the `guards:` argument was present.
    pub(crate) guards_configured: bool,
    /// Optional page transition builder expression.
    pub(crate) transition: Option<String>,
    /// Whether generated pages should be fullscreen dialogs.
    pub(crate) fullscreen_dialog: bool,
    /// Whether generated pages should preserve state.
    pub(crate) maintain_state: bool,
}

/// Parsed router annotation values used by validation and emission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RouterAnnotation {
    /// Required initial path when building a router.
    pub(crate) initial: Option<String>,
    /// Required not-found path when building a router.
    pub(crate) not_found: Option<String>,
}

/// One route constructor parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RouteParamSpec {
    /// Constructor parameter name.
    pub(crate) name: String,
    /// Lowered parameter type.
    pub(crate) ty: TypeIr,
    /// Whether the parameter is bound from the path.
    pub(crate) is_path: bool,
    /// Whether the generated constructor argument is named.
    pub(crate) is_named: bool,
    /// Whether the source constructor parameter has a default.
    pub(crate) has_default: bool,
    /// Preserved default expression source.
    pub(crate) default_value_source: Option<String>,
}

/// One route available to the generated router.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RouteSpec {
    /// Flutter page class built for this route.
    pub(crate) page_class: String,
    /// Generated route data class name.
    pub(crate) route_class: String,
    /// Absolute route path.
    pub(crate) path: String,
    /// Generated route action name.
    pub(crate) name: String,
    /// Parsed annotation settings.
    pub(crate) annotation: RouteAnnotation,
    /// Constructor parameters used by generated route classes.
    pub(crate) params: Vec<RouteParamSpec>,
    /// Optional import URI for workspace routes outside the current library.
    pub(crate) import_uri: Option<String>,
    /// Additional imports needed by workspace routes.
    pub(crate) imports: Vec<String>,
}

/// One router field available for generated refresh and guard injection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RouterFieldSpec {
    /// Field name on the router class.
    pub(crate) name: String,
    /// Simple type name used for refresh/listener and guard matching.
    pub(crate) type_name: String,
}

/// One guard constructor dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GuardParamSpec {
    /// Constructor parameter name.
    pub(crate) name: String,
    /// Simple type name required by the guard, if Dust can resolve it.
    pub(crate) type_name: Option<String>,
    /// Whether the guard constructor argument is named.
    pub(crate) is_named: bool,
    /// Whether the guard parameter has its own default.
    pub(crate) has_default: bool,
    /// Router field name injected into this guard parameter.
    pub(crate) inject_field: Option<String>,
}

/// One guard class available to generated guard lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GuardSpec {
    /// Guard class name.
    pub(crate) class_name: String,
    /// Whether generated code can call the unnamed generative constructor.
    pub(crate) has_unnamed_constructor: bool,
    /// Constructor dependencies for the guard.
    pub(crate) params: Vec<GuardParamSpec>,
}

/// One router root generation spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RouterSpec {
    /// Source router class.
    pub(crate) router_class: String,
    /// Generated abstract base class mixed into the router.
    pub(crate) generated_base_class: String,
    /// Generated route class used as the initial route.
    pub(crate) initial_route_class: String,
    /// Generated route class used as the not-found fallback.
    pub(crate) not_found_route_class: Option<String>,
    /// Router field used as a refresh listenable, if exactly one exists.
    pub(crate) refresh_listenable: Option<String>,
    /// All local and workspace routes available to the router.
    pub(crate) routes: Vec<RouteSpec>,
    /// Guard specs required by the route set.
    pub(crate) guard_specs: Vec<GuardSpec>,
}
