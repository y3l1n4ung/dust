use std::sync::Arc;

use dust_plugin_api::{SymbolPlan, WorkspaceAnalysisBuilder};
use serde_json::json;

use crate::support::{library_with_classes, router_class};

use super::support::{
    assert_route_snapshot, generate_route_output_with_plan, route_outputs_snapshot,
};

#[test]
fn emits_workspace_route_type_matrix() {
    let library = library_with_classes(vec![router_class(
        "(initial: '/filters/:code', notFound: '/404')",
    )]);
    let mut analysis = WorkspaceAnalysisBuilder::default();
    analysis.add_string_set_value(
        "dust_route.routes.v1",
        json!({
            "class_name": "FilterPage",
            "path": "/filters/:code",
            "name": "filters",
            "annotation": {
                "path": "/filters/:code",
                "name": "filters",
                "shell": null,
                "guards": [],
                "guards_configured": false,
                "transition": null,
                "fullscreen_dialog": false,
                "maintain_state": true
            },
            "import_uri": "package:route_test/pages/filter_page.dart",
            "source_path": "lib/pages/filter_page.dart",
            "imports": [
                {
                    "uri": "package:route_test/models/route_tab.dart",
                    "prefix": "tabs",
                    "show": ["RouteTab"],
                    "hide": [],
                    "is_deferred": false
                }
            ],
            "params": [
                {
                    "name": "code",
                    "type_source": "String",
                    "is_named": true,
                    "has_default": false,
                    "default_value_source": null
                },
                {
                    "name": "team",
                    "type_source": "String?",
                    "is_named": true,
                    "has_default": false,
                    "default_value_source": null
                },
                {
                    "name": "minRating",
                    "type_source": "double?",
                    "is_named": true,
                    "has_default": false,
                    "default_value_source": null
                },
                {
                    "name": "featured",
                    "type_source": "bool",
                    "is_named": true,
                    "has_default": true,
                    "default_value_source": "false"
                },
                {
                    "name": "from",
                    "type_source": "DateTime",
                    "is_named": true,
                    "has_default": false,
                    "default_value_source": null
                },
                {
                    "name": "redirect",
                    "type_source": "Uri?",
                    "is_named": true,
                    "has_default": false,
                    "default_value_source": null
                },
                {
                    "name": "tags",
                    "type_source": "List<String>",
                    "is_named": true,
                    "has_default": false,
                    "default_value_source": null
                },
                {
                    "name": "ids",
                    "type_source": "List<int>?",
                    "is_named": true,
                    "has_default": false,
                    "default_value_source": null
                },
                {
                    "name": "tab",
                    "type_source": "tabs.RouteTab",
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

    assert_route_snapshot("workspace_route_type_matrix.dart.snapshot", &output);
}
