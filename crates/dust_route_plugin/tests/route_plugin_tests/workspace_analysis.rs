use dust_plugin_api::{DustPlugin, WorkspaceAnalysisBuilder, WorkspaceAnalysisContext};
use dust_route_plugin::register_plugin;

use super::support::{parsed_annotation, parsed_library_with_annotations};

#[test]
fn collects_route_and_router_workspace_facts() {
    let plugin = register_plugin();
    let library = parsed_library_with_annotations(
        "DashboardPage",
        vec![
            parsed_annotation("Route", "('/', name: 'dashboard')"),
            parsed_annotation("Router", "(initial: DashboardPage, notFound: NotFoundPage)"),
        ],
    );
    let mut builder = WorkspaceAnalysisBuilder::default();

    plugin.collect_workspace_analysis(
        WorkspaceAnalysisContext {
            package_name: "route_test",
            package_root: std::path::Path::new("."),
            source_path: std::path::Path::new("lib/pages/dashboard_page.dart"),
        },
        &library,
        &mut builder,
    );
    let snapshot = builder.snapshot();

    let routes = snapshot.string_set("dust_route.routes.v1").unwrap();
    assert_eq!(routes.len(), 1);
    assert!(routes[0].contains("DashboardPage"));
    assert!(routes[0].contains("dashboard"));

    let routers = snapshot.string_set("dust_route.routers.v1").unwrap();
    assert_eq!(routers.len(), 1);
    assert!(routers[0].contains("DashboardPage"));
    assert!(routers[0].contains("NotFoundPage"));
}
