use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan, WorkspaceAnalysisBuilder};
use dust_route_plugin::register_plugin;
use serde_json::json;
use std::sync::Arc;

use super::support::{constructor_param, library_with_classes, route_page_class, router_class};

#[test]
fn emits_standalone_route_and_core_outputs() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: DashboardPage, notFound: NotFoundPage)"),
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
        route_page_class(
            "NotFoundPage",
            "('/404/:path', name: 'notFound')",
            vec![constructor_param("path", TypeIr::string())],
        ),
    ]);

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let primary = contribution.primary_source.expect("primary route output");

    assert!(primary.contains("import 'route.dart';"));
    assert!(primary.contains("import 'package:dust_router/dust_router.dart';"));
    assert!(primary.contains("abstract class $AppRouter extends DustRouterBase<AppRoutePath>"));
    assert!(primary.contains("final class ProjectRoute extends AppRoutePath"));
    assert!(primary.contains("/// Defaults to true. Override and return false for public routes"));
    assert!(primary.contains("return '/404?path=${Uri.encodeComponent(path)}';"));
    assert!(primary.contains("segments.length == 1 && segments[0] == '404'"));
    assert!(primary.contains("NotFoundRoute(path: uri.toString())"));
    assert!(primary.contains("_parseBool: unrecognised value"));
    assert!(primary.contains("bool _shellConsistencyCheck()"));
    assert!(primary.contains("transition: FadeUpwardsPageTransitionsBuilder()"));
    assert!(primary.contains("generatedPage("));
    assert!(!primary.contains("pageType:"));
    assert!(!primary.contains("_kDefaultTransition"));
    assert!(!primary.contains("class _NoTransitionBuilder"));
    assert!(primary.contains("'projects',"));
    assert!(primary.contains("name: 'projectSettings'"));
    assert!(primary.contains("ProjectSettingsPage: AppShell"));
    assert!(primary.contains("child: AppShell(child: ProjectSettingsPage"));
    assert!(primary.contains("projectId.toString(),"));
    assert!(!primary.contains("Uri.decodeComponent(segments"));
    assert!(primary.contains("int.tryParse(segments[1])"));
    assert!(primary.contains("if (archived != null)"));
    assert!(primary.contains("query['archived'] = archived!.toString();"));
    assert!(primary.contains("archived: _parseBool(uri.queryParameters['archived'])"));
    assert!(primary.contains("AppRoutesNavigation get routes"));
    assert!(primary.contains("RouteNavigation<AppRoutePath> project"));
    assert!(!primary.contains("part of"));

    assert!(contribution.auxiliary_outputs.is_empty());
}

#[test]
fn emits_no_transition_builder_only_when_referenced() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: SearchPage)"),
        route_page_class(
            "SearchPage",
            "('/search', name: 'search', transition: _NoTransitionBuilder())",
            Vec::new(),
        ),
    ]);

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let primary = contribution.primary_source.expect("primary route output");

    assert!(primary.contains("class _NoTransitionBuilder extends PageTransitionsBuilder"));
    assert!(primary.contains("transition: _NoTransitionBuilder()"));
}

#[test]
fn emits_workspace_page_imports_and_query_defaults() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![router_class("(initial: SearchPage)")]);
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

    assert!(primary.contains("import 'package:route_test/pages/search_page.dart';"));
    assert!(primary.contains("const SearchRoute({this.page = 1});"));
    assert!(primary.contains("if (page != 1) {"));
    assert!(primary.contains("query['page'] = page.toString();"));
    assert!(primary.contains("page: int.tryParse(uri.queryParameters['page'] ?? '') ?? 1"));
}

#[test]
fn emits_shell_import_from_route_page_library_imports() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![router_class("(initial: ProjectPage)")]);
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

    assert!(primary.contains("import 'package:route_test/pages/project_page.dart';"));
    assert!(primary.contains("import 'package:route_test/layout/app_shell.dart';"));
    assert!(primary.contains("ProjectPage: AppShell"));
    assert!(primary.contains("child: AppShell(child: ProjectPage"));
}

#[test]
fn emits_large_route_sets_without_excessive_output_growth() {
    let plugin = register_plugin();
    let mut classes = vec![router_class("(initial: Page0)")];
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

    assert!(primary.contains("final class Route149Route extends AppRoutePath"));
    assert!(primary.contains("RouteNavigation<AppRoutePath> route149"));
    assert!(primary.len() < 300_000);
}

#[test]
fn emits_deep_nested_route_tree_metadata() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: ReportPage)"),
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

    assert!(primary.contains("'/orgs',"));
    assert!(primary.contains("':orgId',"));
    assert!(primary.contains("'projects',"));
    assert!(primary.contains("':projectId',"));
    assert!(primary.contains("'reports',"));
    assert!(primary.contains("':reportId',"));
    assert!(primary.contains("name: 'report'"));
}
