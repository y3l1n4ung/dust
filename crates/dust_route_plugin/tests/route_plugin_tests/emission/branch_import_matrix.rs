use dust_plugin_api::{SymbolPlan, WorkspaceAnalysisBuilder};
use serde_json::json;
use std::sync::Arc;

use crate::support::{library_with_classes, router_class};

use super::support::{
    assert_route_snapshot, generate_route_output_with_plan, route_outputs_snapshot,
};

#[test]
fn emits_branch_collision_constants_and_current_entrypoint_imports() {
    let library = library_with_classes(vec![router_class("(initial: '/alpha', notFound: '/404')")]);
    let mut analysis = WorkspaceAnalysisBuilder::default();
    for route in [
        workspace_route(
            "AlphaPage",
            "/alpha",
            "alpha",
            "shop-tabs",
            json!({
                "name": "tab",
                "type_source": "RouteTab",
                "is_named": true,
                "has_default": false,
                "default_value_source": null
            }),
            json!({
                "uri": "package:route_test/route.dart",
                "prefix": null,
                "show": ["RouteTab"],
                "hide": [],
                "is_deferred": false
            }),
        ),
        workspace_route(
            "BetaPage",
            "/beta",
            "beta",
            "shop tabs",
            json!({
                "name": "filter",
                "type_source": "entry.RouteFilter",
                "is_named": true,
                "has_default": false,
                "default_value_source": null
            }),
            json!({
                "uri": "package:route_test/route.dart",
                "prefix": "entry",
                "show": [],
                "hide": [],
                "is_deferred": false
            }),
        ),
        workspace_route(
            "SymbolBranchPage",
            "/symbols",
            "symbols",
            "!!!",
            json!({
                "name": "sort",
                "type_source": "SortMode",
                "is_named": true,
                "has_default": false,
                "default_value_source": null
            }),
            json!({
                "uri": "package:route_test/route.dart",
                "prefix": null,
                "show": [],
                "hide": ["HiddenType"],
                "is_deferred": false
            }),
        ),
    ] {
        analysis.add_string_set_value("dust_route.routes.v1", route.to_string());
    }
    let mut plan = SymbolPlan::default();
    plan.set_workspace_analysis(Arc::new(analysis.build()));

    let contribution = generate_route_output_with_plan(&library, &plan);
    assert_eq!(contribution.diagnostics, []);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("branch_import_matrix.dart.snapshot", &output);
}

fn workspace_route(
    class_name: &str,
    path: &str,
    name: &str,
    branch: &str,
    param: serde_json::Value,
    import: serde_json::Value,
) -> serde_json::Value {
    json!({
        "class_name": class_name,
        "path": path,
        "name": name,
        "annotation": {
            "path": path,
            "name": name,
            "shell": null,
            "branch": branch,
            "guards": [],
            "guards_configured": false,
            "transition": null,
            "fullscreen_dialog": false,
            "maintain_state": true
        },
        "import_uri": format!("package:route_test/pages/{}.dart", snake_case(class_name)),
        "source_path": format!("lib/pages/{}.dart", snake_case(class_name)),
        "imports": [import],
        "params": [param]
    })
}

fn snake_case(value: &str) -> String {
    let mut out = String::new();
    for (index, ch) in value.chars().enumerate() {
        if index > 0 && ch.is_ascii_uppercase() {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}
