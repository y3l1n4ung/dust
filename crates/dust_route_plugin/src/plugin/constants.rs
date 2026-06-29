/// Class-level annotation that marks the application router root.
pub(crate) const ROUTER: &str = "AppRouter";
/// Class-level annotation that marks a page as routable.
pub(crate) const ROUTE: &str = "AppRoute";
/// Generated route marker used by route codegen.
pub(crate) const GENERATED_ROUTE: &str = "GeneratedRoute";

/// Workspace analysis key for discovered route pages.
pub(crate) const ROUTES_ANALYSIS_KEY: &str = "dust_route.routes.v1";
/// Workspace analysis key for discovered router roots.
pub(crate) const ROUTERS_ANALYSIS_KEY: &str = "dust_route.routers.v1";
/// Workspace analysis key for discovered guard classes.
pub(crate) const GUARDS_ANALYSIS_KEY: &str = "dust_route.guards.v1";

/// Fully qualified Dust symbols claimed by the route plugin.
pub(crate) const CLAIMED_CONFIG_SYMBOLS: &[&str] = &[
    "dust_flutter::AppRouter",
    "dust_flutter::AppRoute",
    "dust_flutter::GeneratedRoute",
];

/// Short annotation names supported by the route plugin.
pub(crate) const SUPPORTED_ANNOTATIONS: &[&str] = &[ROUTER, ROUTE, GENERATED_ROUTE];
