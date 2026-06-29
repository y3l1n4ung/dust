use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan, WorkspaceAnalysisBuilder};
use dust_route_plugin::register_plugin;
use serde_json::json;
use std::{fs, path::PathBuf, sync::Arc};

use super::support::{
    constructor_param, guard_class, library_with_classes, named_constructor_guard_class,
    route_page_class, router_class,
};

#[test]
fn emits_standalone_route_and_core_outputs() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/', notFound: '/404')"),
        route_page_class(
            "DashboardPage",
            "('/', name: 'dashboard', transition: FadeUpwardsPageTransitionsBuilder())",
            Vec::new(),
        ),
        route_page_class(
            "ProjectPage",
            "('/projects/:projectId', name: 'project', shell: AppShell)",
            vec![
                constructor_param("projectId", TypeIr::int()),
                constructor_param("tab", TypeIr::string().nullable()),
                constructor_param("archived", TypeIr::bool().nullable()),
            ],
        ),
        route_page_class(
            "ProjectSettingsPage",
            "('/projects/:projectId/settings', name: 'projectSettings')",
            vec![constructor_param("projectId", TypeIr::int())],
        ),
    ]);

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let primary = contribution.primary_source.expect("primary route output");

    assert_snapshot("standalone_route.dart.snapshot", &primary);
    assert!(contribution.auxiliary_outputs.is_empty());
}

#[test]
fn emits_no_transition_builder_only_when_referenced() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/search', notFound: '/404')"),
        route_page_class(
            "SearchPage",
            "('/search', name: 'search', transition: _NoTransitionBuilder())",
            Vec::new(),
        ),
    ]);

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let primary = contribution.primary_source.expect("primary route output");

    assert_snapshot("no_transition_route.dart.snapshot", &primary);
}

#[test]
fn emits_guard_helpers_with_custom_router_base_name() {
    let plugin = register_plugin();
    let mut router = router_class("(initial: '/', notFound: '/404')");
    router.name = "BenchmarkRouter".to_owned();
    router.superclass_name = Some("$BenchmarkRouter".to_owned());
    let library = library_with_classes(vec![
        router,
        route_page_class(
            "DashboardPage",
            "('/', name: 'dashboard', guards: [BenchmarkGuard])",
            Vec::new(),
        ),
    ]);

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let primary = contribution.primary_source.expect("primary route output");

    assert!(primary.contains("DashboardRoute() => [BenchmarkGuard()]"));
    assert!(!primary.contains("const BenchmarkGuard()"));
    assert_snapshot("custom_router_guard_route.dart.snapshot", &primary);
}

#[test]
fn rejects_guard_without_unnamed_constructor() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/', notFound: '/404')"),
        route_page_class(
            "DashboardPage",
            "('/', name: 'dashboard', guards: [AuthGuard])",
            Vec::new(),
        ),
        named_constructor_guard_class("AuthGuard"),
    ]);

    let contribution = plugin.emit(&library, &SymbolPlan::default());

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
    let plugin = register_plugin();
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

    let contribution = plugin.emit(&library, &SymbolPlan::default());

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "route guard `AuthGuard` constructor parameter `predicate` needs a resolvable type for router injection"
        ]
    );
}

#[test]
fn emits_workspace_page_imports_and_query_defaults() {
    let plugin = register_plugin();
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
            "imports": ["package:route_test/layout/app_shell.dart"],
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

    let contribution = plugin.emit(&library, &plan);
    let primary = contribution.primary_source.expect("primary route output");

    assert_snapshot("workspace_default_route.dart.snapshot", &primary);
}

#[test]
fn emits_shell_import_from_route_page_library_imports() {
    let plugin = register_plugin();
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
                "shell": "AppShell",
                "guards": [],
                "guards_configured": false,
                "transition": null,
                "fullscreen_dialog": false,
                "maintain_state": true
            },
            "import_uri": "package:route_test/pages/project_page.dart",
            "source_path": "lib/pages/project_page.dart",
            "imports": ["package:route_test/layout/app_shell.dart"],
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

    let contribution = plugin.emit(&library, &plan);
    let primary = contribution.primary_source.expect("primary route output");

    assert_snapshot("workspace_shell_route.dart.snapshot", &primary);
}

#[test]
fn emits_large_route_sets_without_excessive_output_growth() {
    let plugin = register_plugin();
    let mut classes = vec![router_class("(initial: '/section/0', notFound: '/404')")];
    for index in 0..150 {
        classes.push(route_page_class(
            &format!("Page{index}"),
            &format!("('/section/{index}', name: 'route{index}')"),
            Vec::new(),
        ));
    }
    let library = library_with_classes(classes);

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let primary = contribution.primary_source.expect("primary route output");

    assert_snapshot("large_route_set.dart.snapshot", &primary);
}

#[test]
fn emits_deep_nested_route_tree_metadata() {
    let plugin = register_plugin();
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

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let primary = contribution.primary_source.expect("primary route output");

    assert_snapshot("deep_nested_route.dart.snapshot", &primary);
}

fn assert_snapshot(name: &str, actual: &str) {
    let path = snapshot_path(name);
    if std::env::var_os("DUST_UPDATE_ROUTE_SNAPSHOTS").is_some() {
        fs::write(&path, actual).unwrap();
    }
    let expected = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("missing route snapshot `{}`: {error}", path.display()));
    assert_eq!(actual, expected, "route snapshot `{name}` changed");
}

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/route_plugin_tests/snapshots")
        .join(name)
}

fn diagnostic_messages(diagnostics: &[dust_diagnostics::Diagnostic]) -> Vec<&str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}
