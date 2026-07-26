use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, WorkspaceAnalysisBuilder};
use dust_route_plugin::register_plugin;
use serde_json::Value;

use super::support::{constructor_param, library_with_classes, route_page_class, router_class};

#[test]
fn collects_route_and_router_workspace_facts_from_canonical_ir() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        route_page_class(
            "DashboardPage",
            "('/', name: 'dashboard', result: bool)",
            vec![constructor_param("id", TypeIr::string())],
        ),
        router_class("(initial: '/', notFound: '/404')"),
    ]);
    let mut builder = WorkspaceAnalysisBuilder::default();

    plugin.collect_workspace_analysis_ir(&library, &mut builder);
    let snapshot = builder.snapshot();

    let routes = snapshot.string_set("dust_route.routes.v1").unwrap();
    assert!(routes.iter().any(|route| {
        let route = serde_json::from_str::<Value>(route).unwrap();
        route["class_name"] == "DashboardPage"
            && route["import_uri"] == "package:route_test/route.dart"
            && route["annotation"]["result_type"] == "bool"
            && route["params"][0]["type_source"] == "String"
    }));
    let routers = snapshot.string_set("dust_route.routers.v1").unwrap();
    assert!(routers.iter().any(|router| {
        let router = serde_json::from_str::<Value>(router).unwrap();
        router["class_name"] == "TestRouter"
            && router["initial"] == "/"
            && router["not_found"] == "/404"
    }));
}
