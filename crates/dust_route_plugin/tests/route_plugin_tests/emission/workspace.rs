use dust_ir::TypeIr;
use dust_plugin_api::{SymbolPlan, WorkspaceAnalysisBuilder};
use serde_json::json;
use std::sync::Arc;

use crate::support::{constructor_param, library_with_classes, route_page_class, router_class};

use super::support::{
    assert_route_snapshot, generate_route_output, generate_route_output_with_plan,
    route_outputs_snapshot,
};

#[test]
fn emits_workspace_page_imports_and_query_defaults() {
    let library =
        library_with_classes(vec![router_class("(initial: '/search', notFound: '/404')")]);
    let mut analysis = WorkspaceAnalysisBuilder::default();
    analysis.add_string_set_value(
        "dust_route.routes.v1",
        json!({
            "class_name": "SearchPage",
            "path": "/search",
            "name": "search",
            "annotation": {
                "path": "/search",
                "name": "search",
                "shell": null,
                "guards": [],
                "guards_configured": false,
                "transition": null,
                "fullscreen_dialog": false,
                "maintain_state": true
            },
            "import_uri": "package:route_test/pages/search_page.dart",
            "source_path": "lib/pages/search_page.dart",
            "imports": [
                {
                    "uri": "package:route_test/layout/app_shell.dart",
                    "prefix": null,
                    "show": [],
                    "hide": [],
                    "is_deferred": false
                }
            ],
            "params": [
                {
                    "name": "page",
                    "type_source": "int",
                    "is_named": true,
                    "has_default": true,
                    "default_value_source": "1"
                }
            ]
        })
        .to_string(),
    );
    let mut plan = SymbolPlan::default();
    plan.set_workspace_analysis(Arc::new(analysis.build()));

    let contribution = generate_route_output_with_plan(&library, &plan);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("workspace_default_route.dart.snapshot", &output);
}

#[test]
fn emits_shell_import_from_route_page_library_imports() {
    let library = library_with_classes(vec![router_class(
        "(initial: '/projects/:projectId', notFound: '/404')",
    )]);
    let mut analysis = WorkspaceAnalysisBuilder::default();
    analysis.add_string_set_value(
        "dust_route.routes.v1",
        json!({
            "class_name": "ProjectPage",
            "path": "/projects/:projectId",
            "name": "project",
            "annotation": {
                "path": "/projects/:projectId",
                "name": "project",
                "shell": "ui.AppShell",
                "guards": [],
                "guards_configured": false,
                "transition": null,
                "fullscreen_dialog": false,
                "maintain_state": true
            },
            "import_uri": "package:route_test/pages/project_page.dart",
            "source_path": "lib/pages/project_page.dart",
            "imports": [
                {
                    "uri": "package:route_test/layout/app_shell.dart",
                    "prefix": "ui",
                    "show": ["AppShell"],
                    "hide": ["InternalShell"],
                    "is_deferred": false
                }
            ],
            "params": [
                {
                    "name": "projectId",
                    "type_source": "int",
                    "is_named": true,
                    "has_default": false,
                    "default_value_source": null
                }
            ]
        })
        .to_string(),
    );
    let mut plan = SymbolPlan::default();
    plan.set_workspace_analysis(Arc::new(analysis.build()));

    let contribution = generate_route_output_with_plan(&library, &plan);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("workspace_shell_route.dart.snapshot", &output);
}

#[test]
fn prunes_workspace_imports_to_referenced_route_symbols() {
    let library = library_with_classes(vec![router_class(
        "(initial: '/purchase', notFound: '/404')",
    )]);
    let mut analysis = WorkspaceAnalysisBuilder::default();
    analysis.add_string_set_value(
        "dust_route.routes.v1",
        json!({
            "class_name": "PurchasePage",
            "path": "/purchase",
            "name": "purchase",
            "annotation": {
                "path": "/purchase",
                "name": "purchase",
                "result_type": "types.PurchaseResult",
                "shell": "layout.ShopShell",
                "guards": ["auth.RequireLogin"],
                "guards_configured": true,
                "transition": "const SlideTransitionBuilder()",
                "fullscreen_dialog": true,
                "maintain_state": false
            },
            "import_uri": "package:route_test/pages/purchase_page.dart",
            "source_path": "lib/pages/purchase_page.dart",
            "imports": [
                {
                    "uri": "package:route_test/auth/require_login.dart",
                    "prefix": "auth",
                    "show": ["RequireLogin"],
                    "hide": [],
                    "is_deferred": false
                },
                {
                    "uri": "package:route_test/layout/shop_shell.dart",
                    "prefix": "layout",
                    "show": ["ShopShell"],
                    "hide": [],
                    "is_deferred": false
                },
                {
                    "uri": "package:route_test/models/purchase_result.dart",
                    "prefix": "types",
                    "show": ["PurchaseResult"],
                    "hide": [],
                    "is_deferred": false
                },
                {
                    "uri": "package:route_test/motion/slide_transition_builder.dart",
                    "prefix": null,
                    "show": [],
                    "hide": [],
                    "is_deferred": false
                },
                {
                    "uri": "package:route_test/models/product.dart",
                    "prefix": null,
                    "show": [],
                    "hide": [],
                    "is_deferred": false
                },
                {
                    "uri": "package:route_test/widgets/unrelated_badge.dart",
                    "prefix": "badge",
                    "show": ["UnrelatedBadge"],
                    "hide": [],
                    "is_deferred": false
                }
            ],
            "params": []
        })
        .to_string(),
    );
    let mut plan = SymbolPlan::default();
    plan.set_workspace_analysis(Arc::new(analysis.build()));

    let contribution = generate_route_output_with_plan(&library, &plan);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("workspace_pruned_imports.dart.snapshot", &output);
}

#[test]
fn emits_large_route_sets_without_excessive_output_growth() {
    let mut classes = vec![router_class("(initial: '/section/0', notFound: '/404')")];
    for index in 0..150 {
        classes.push(route_page_class(
            &format!("Page{index}"),
            &format!("('/section/{index}', name: 'route{index}')"),
            Vec::new(),
        ));
    }
    let library = library_with_classes(classes);

    let contribution = generate_route_output(&library);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("large_route_set.dart.snapshot", &output);
}

#[test]
fn emits_deep_nested_route_tree_metadata() {
    let library = library_with_classes(vec![
        router_class(
            "(initial: '/orgs/:orgId/projects/:projectId/reports/:reportId', notFound: '/404')",
        ),
        route_page_class(
            "ReportPage",
            "('/orgs/:orgId/projects/:projectId/reports/:reportId', name: 'report')",
            vec![
                constructor_param("orgId", TypeIr::int()),
                constructor_param("projectId", TypeIr::int()),
                constructor_param("reportId", TypeIr::int()),
            ],
        ),
    ]);

    let contribution = generate_route_output(&library);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("deep_nested_route.dart.snapshot", &output);
}
