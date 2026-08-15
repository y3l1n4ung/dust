use std::sync::Arc;

use dust_ir::{ImportIr, TypeIr};
use dust_plugin_api::{SymbolPlan, WorkspaceAnalysisBuilder};
use serde_json::json;

use crate::support::{
    constructor_param, guard_class, library_with_classes, named_constructor_guard_class,
    route_page_class, router_class, router_field,
};

use super::support::{diagnostic_messages, generate_route_output, generate_route_output_with_plan};

#[test]
fn rejects_guard_without_unnamed_constructor() {
    let library = library_with_classes(vec![
        router_class("(initial: '/', notFound: '/404')"),
        route_page_class(
            "DashboardPage",
            "('/', name: 'dashboard', guards: [AuthGuard])",
            Vec::new(),
        ),
        named_constructor_guard_class("AuthGuard"),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "route guard `AuthGuard` needs an unnamed generative constructor for generated route guard lookup"
        ]
    );
}

#[test]
fn rejects_guard_required_dependency_with_unresolvable_type() {
    let library = library_with_classes(vec![
        router_class("(initial: '/', notFound: '/404')"),
        route_page_class(
            "DashboardPage",
            "('/', name: 'dashboard', guards: [AuthGuard])",
            Vec::new(),
        ),
        guard_class(
            "AuthGuard",
            vec![constructor_param(
                "predicate",
                TypeIr::function("bool Function(String value)"),
            )],
        ),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "route guard `AuthGuard` constructor parameter `predicate` needs a resolvable type for router injection"
        ]
    );
}

#[test]
fn rejects_guard_dependency_without_matching_router_field() {
    let library = library_with_classes(vec![
        router_class("(initial: '/', notFound: '/404')"),
        route_page_class(
            "DashboardPage",
            "('/', name: 'dashboard', guards: [AuthGuard])",
            Vec::new(),
        ),
        guard_class(
            "AuthGuard",
            vec![constructor_param("auth", TypeIr::named("AuthService"))],
        ),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec!["route guard `AuthGuard` needs `auth` but router has no field of type `AuthService`"]
    );
}

#[test]
fn rejects_guard_dependency_with_ambiguous_router_fields() {
    let mut router = router_class("(initial: '/', notFound: '/404')");
    router.fields = vec![
        router_field("primaryAuth", "AuthService"),
        router_field("secondaryAuth", "AuthService"),
    ];
    let library = library_with_classes(vec![
        router,
        route_page_class(
            "DashboardPage",
            "('/', name: 'dashboard', guards: [AuthGuard])",
            Vec::new(),
        ),
        guard_class(
            "AuthGuard",
            vec![constructor_param("auth", TypeIr::named("AuthService"))],
        ),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "route guard `AuthGuard` dependency `auth` has multiple router fields of type `AuthService`"
        ]
    );
}

#[test]
fn rejects_workspace_guard_dependency_without_matching_router_field() {
    let mut library = library_with_classes(vec![
        router_class("(initial: '/', notFound: '/404')"),
        route_page_class(
            "DashboardPage",
            "('/', name: 'dashboard', guards: [AuthGuard])",
            Vec::new(),
        ),
    ]);
    library
        .import_directives
        .push(import("package:app/auth_guard.dart"));

    let mut analysis = WorkspaceAnalysisBuilder::default();
    analysis.add_string_set_value(
        "dust_route.guards.v1",
        json!({
            "class_name": "AuthGuard",
            "has_unnamed_constructor": true,
            "import_uri": "package:app/auth_guard.dart",
            "source_path": "lib/auth_guard.dart",
            "params": [
                {
                    "name": "auth",
                    "type_source": "AuthService",
                    "is_named": true,
                    "has_default": false
                }
            ]
        })
        .to_string(),
    );
    let mut plan = SymbolPlan::default();
    plan.set_workspace_analysis(Arc::new(analysis.build()));

    let contribution = generate_route_output_with_plan(&library, &plan);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec!["route guard `AuthGuard` needs `auth` but router has no field of type `AuthService`"]
    );
}

fn import(uri: &str) -> ImportIr {
    ImportIr {
        uri: uri.to_owned(),
        prefix: None,
        show: Vec::new(),
        hide: Vec::new(),
        is_deferred: false,
        span: crate::support::span(0, 0),
    }
}
