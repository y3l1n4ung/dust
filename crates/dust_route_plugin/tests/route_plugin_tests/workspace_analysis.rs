use dust_plugin_api::{DustPlugin, WorkspaceAnalysisBuilder, WorkspaceAnalysisContext};
use dust_route_plugin::register_plugin;
use serde_json::{Value, json};

use super::support::{
    parsed_annotation, parsed_library_with_annotations, parsed_prefixed_annotation,
};

#[test]
fn collects_route_and_router_workspace_facts() {
    let plugin = register_plugin();
    let library = parsed_library_with_annotations(
        "DashboardPage",
        vec![
            parsed_annotation("AppRoute", "('/', name: 'dashboard')"),
            parsed_annotation("AppRouter", "(initial: '/', notFound: '/404')"),
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
    assert_json_eq(
        &routes[0],
        json!({
            "class_name": "DashboardPage",
            "path": "/",
            "name": "dashboard",
            "annotation": {
                "path": "/",
                "name": "dashboard",
                "shell": null,
                "guards": [],
                "guards_configured": false,
                "transition": null,
                "fullscreen_dialog": false,
                "maintain_state": true
            },
            "import_uri": "package:route_test/pages/dashboard_page.dart",
            "source_path": "lib/pages/dashboard_page.dart",
            "imports": [],
            "params": []
        }),
    );

    let routers = snapshot.string_set("dust_route.routers.v1").unwrap();
    assert_eq!(routers.len(), 1);
    assert_json_eq(
        &routers[0],
        json!({
            "class_name": "DashboardPage",
            "initial": "/",
            "not_found": "/404",
            "source_path": "lib/pages/dashboard_page.dart"
        }),
    );
}

#[test]
fn collects_prefixed_route_and_router_workspace_facts() {
    let plugin = register_plugin();
    let library = parsed_library_with_annotations(
        "DashboardPage",
        vec![
            parsed_prefixed_annotation("f", "AppRoute", "('/', name: 'dashboard')"),
            parsed_prefixed_annotation("f", "AppRouter", "(initial: '/', notFound: '/404')"),
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

    assert_eq!(
        snapshot.string_set("dust_route.routes.v1").unwrap().len(),
        1
    );
    assert_eq!(
        snapshot.string_set("dust_route.routers.v1").unwrap().len(),
        1
    );
}

fn assert_json_eq(actual: &str, expected: Value) {
    let actual = serde_json::from_str::<Value>(actual).unwrap();
    assert_eq!(actual, expected);
}
