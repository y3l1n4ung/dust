pub(crate) const ROUTER: &str = "Router";
pub(crate) const ROUTE: &str = "Route";
pub(crate) const GENERATED_ROUTE: &str = "GeneratedRoute";

pub(crate) const ROUTES_ANALYSIS_KEY: &str = "dust_route.routes.v1";
pub(crate) const ROUTERS_ANALYSIS_KEY: &str = "dust_route.routers.v1";

pub(crate) const CLAIMED_CONFIG_SYMBOLS: &[&str] = &[
    "dust_router::Router",
    "dust_router::Route",
    "dust_router::GeneratedRoute",
];

pub(crate) const SUPPORTED_ANNOTATIONS: &[&str] = &[ROUTER, ROUTE, GENERATED_ROUTE];
