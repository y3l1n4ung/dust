/// Resolver-normalized `AppRoute` configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteConfigIr {
    /// Absolute route path, including `:param` placeholders.
    pub path: String,
    /// Optional explicit route action name.
    pub name: Option<String>,
    /// Optional result type returned by route pushes.
    pub result_type: Option<String>,
    /// Optional shell widget class wrapping the page.
    pub shell: Option<String>,
    /// Guard class names applied to the route.
    pub guards: Vec<String>,
    /// Whether the `guards:` argument was present.
    pub guards_configured: bool,
    /// Optional page transition builder expression.
    pub transition: Option<String>,
    /// Whether generated pages should be fullscreen dialogs.
    pub fullscreen_dialog: bool,
    /// Whether generated pages should preserve state.
    pub maintain_state: bool,
}

/// Resolver-normalized `AppRouter` configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouterConfigIr {
    /// Optional initial route path.
    pub initial: Option<String>,
    /// Optional not-found route path.
    pub not_found: Option<String>,
}
