use dust_ir::{ImportIr, TypeIr};
use dust_plugin_api::{DustPlugin, WorkspaceAnalysisBuilder};
use dust_route_plugin::register_plugin;
use serde_json::{Value, json};

use super::support::{
    constructor_param, defaulted_param_source, guard_class, library_with_classes, route_page_class,
    router_class, span,
};

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

#[test]
fn collects_route_imports_and_params_from_canonical_ir() {
    let plugin = register_plugin();
    let mut library = library_with_classes(vec![
        route_page_class(
            "ProductDetailPage",
            "('/products/:id', name: 'productDetail')",
            vec![
                constructor_param("id", TypeIr::int()),
                defaulted_param_source("preview", TypeIr::bool(), "false"),
                constructor_param("key", TypeIr::named("Key")),
            ],
        ),
        guard_class(
            "ProductGuard",
            vec![
                constructor_param("cache", TypeIr::unknown()),
                defaulted_param_source("enabled", TypeIr::bool(), "true"),
            ],
        ),
    ]);
    library.package_root = "/workspace/shop".to_owned();
    library.source_path = "/workspace/shop/lib/features/product/routes.dart".to_owned();
    library.import_directives = vec![
        import("package:flutter/material.dart"),
        import("package:dust_flutter/route.dart"),
        import("dart:async"),
        import("package:shop/models/product.dart"),
        import("route.g.dart"),
        import("routing_core.dart"),
        import("route_annotations.dart"),
        import("../../route_annotations.dart"),
        import("../models/product_filter.dart"),
        import("./widgets/product_shell.dart"),
        ImportIr {
            uri: "../guards/product_guard.dart".to_owned(),
            prefix: Some("guards".to_owned()),
            show: vec!["ProductGuard".to_owned()],
            hide: vec!["DebugGuard".to_owned()],
            is_deferred: true,
            span: span(0, 0),
        },
    ];
    let mut builder = WorkspaceAnalysisBuilder::default();

    plugin.collect_workspace_analysis_ir(&library, &mut builder);
    let snapshot = builder.snapshot();

    let route = json_by_class(
        snapshot.string_set("dust_route.routes.v1").unwrap(),
        "ProductDetailPage",
    );
    assert_eq!(
        route,
        json!({
            "annotation": {
                "branch": null,
                "fullscreen_dialog": false,
                "guards": [],
                "guards_configured": false,
                "maintain_state": true,
                "name": "productDetail",
                "path": "/products/:id",
                "result_type": null,
                "shell": null,
                "transition": null
            },
            "class_name": "ProductDetailPage",
            "import_uri": "package:route_test/features/product/routes.dart",
            "imports": [
                {
                    "hide": [],
                    "is_deferred": false,
                    "prefix": null,
                    "show": [],
                    "uri": "dart:async"
                },
                {
                    "hide": ["DebugGuard"],
                    "is_deferred": true,
                    "prefix": "guards",
                    "show": ["ProductGuard"],
                    "uri": "package:route_test/features/guards/product_guard.dart"
                },
                {
                    "hide": [],
                    "is_deferred": false,
                    "prefix": null,
                    "show": [],
                    "uri": "package:route_test/features/models/product_filter.dart"
                },
                {
                    "hide": [],
                    "is_deferred": false,
                    "prefix": null,
                    "show": [],
                    "uri": "package:route_test/features/product/widgets/product_shell.dart"
                },
                {
                    "hide": [],
                    "is_deferred": false,
                    "prefix": null,
                    "show": [],
                    "uri": "package:shop/models/product.dart"
                }
            ],
            "name": "productDetail",
            "params": [
                {
                    "default_value_source": null,
                    "has_default": false,
                    "is_named": true,
                    "name": "id",
                    "type_source": "int"
                },
                {
                    "default_value_source": "false",
                    "has_default": true,
                    "is_named": true,
                    "name": "preview",
                    "type_source": "bool"
                }
            ],
            "path": "/products/:id",
            "source_path": "/workspace/shop/lib/features/product/routes.dart"
        })
    );

    let guard = json_by_class(
        snapshot.string_set("dust_route.guards.v1").unwrap(),
        "ProductGuard",
    );
    assert_eq!(
        guard,
        json!({
            "class_name": "ProductGuard",
            "has_unnamed_constructor": true,
            "import_uri": "package:route_test/features/product/routes.dart",
            "params": [
                {
                    "has_default": false,
                    "is_named": true,
                    "name": "cache",
                    "type_source": null
                },
                {
                    "has_default": true,
                    "is_named": true,
                    "name": "enabled",
                    "type_source": "bool"
                }
            ],
            "source_path": "/workspace/shop/lib/features/product/routes.dart"
        })
    );
}

#[test]
fn collects_source_path_import_uri_for_non_lib_route_files() {
    let plugin = register_plugin();
    let mut library = library_with_classes(vec![route_page_class(
        "ToolRoutePage",
        "('/tool')",
        Vec::new(),
    )]);
    library.source_path = "tool/routes.dart".to_owned();
    let mut builder = WorkspaceAnalysisBuilder::default();

    plugin.collect_workspace_analysis_ir(&library, &mut builder);

    let snapshot = builder.snapshot();
    let route = json_by_class(
        snapshot.string_set("dust_route.routes.v1").unwrap(),
        "ToolRoutePage",
    );
    assert_eq!(route["import_uri"], "tool/routes.dart");
}

fn import(uri: &str) -> ImportIr {
    ImportIr {
        uri: uri.to_owned(),
        prefix: None,
        show: Vec::new(),
        hide: Vec::new(),
        is_deferred: false,
        span: span(0, 0),
    }
}

fn json_by_class(values: &[String], class_name: &str) -> Value {
    values
        .iter()
        .map(|value| serde_json::from_str::<Value>(value).unwrap())
        .find(|value| value["class_name"] == class_name)
        .unwrap()
}
