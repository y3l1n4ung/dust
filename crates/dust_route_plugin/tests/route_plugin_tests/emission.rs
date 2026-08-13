use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan, WorkspaceAnalysisBuilder};
use dust_route_plugin::register_plugin;
use serde_json::json;
use std::{fs, path::PathBuf, sync::Arc};

use super::support::{
    constructor_param, defaulted_param, defaulted_param_source, guard_class, library_with_classes,
    named_constructor_guard_class, route_page_class, router_class,
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

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
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

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let primary = contribution.primary_source.expect("primary route output");

    assert_snapshot("no_transition_route.dart.snapshot", &primary);
}

#[test]
fn emits_typed_route_result_helpers() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/', notFound: '/404')"),
        route_page_class("HomePage", "('/', name: 'home')", Vec::new()),
        route_page_class(
            "PickerPage",
            "('/picker', name: 'picker', result: bool)",
            Vec::new(),
        ),
    ]);

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let primary = contribution.primary_source.expect("primary route output");

    assert_snapshot("typed_route_result.dart.snapshot", &primary);
}

#[test]
fn emits_inherited_shell_without_extra_annotations() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/dashboard/orders', notFound: '/404')"),
        route_page_class(
            "DashboardPage",
            "('/dashboard', name: 'dashboard', shell: AppShell)",
            Vec::new(),
        ),
        route_page_class(
            "DashboardOrdersPage",
            "('/dashboard/orders', name: 'dashboardOrders')",
            Vec::new(),
        ),
    ]);

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let primary = contribution.primary_source.expect("primary route output");

    assert_snapshot("inherited_shell_route.dart.snapshot", &primary);
}

#[test]
fn emits_nearest_shell_when_child_route_overrides_parent_shell() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/dashboard/orders', notFound: '/404')"),
        route_page_class(
            "DashboardPage",
            "('/dashboard', name: 'dashboard', shell: AppShell)",
            Vec::new(),
        ),
        route_page_class(
            "DashboardOrdersPage",
            "('/dashboard/orders', name: 'dashboardOrders', shell: OrdersShell)",
            Vec::new(),
        ),
    ]);

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let primary = contribution.primary_source.expect("primary route output");

    assert_snapshot("shell_override_route.dart.snapshot", &primary);
}

#[test]
fn emits_common_app_route_use_cases() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/dashboard', notFound: '/404')"),
        route_page_class(
            "DashboardPage",
            "('/dashboard', name: 'dashboard', shell: AppShell, branch: 'mainTabs', guards: [])",
            Vec::new(),
        ),
        route_page_class(
            "DashboardOrdersPage",
            "('/dashboard/orders', name: 'dashboardOrders')",
            Vec::new(),
        ),
        route_page_class(
            "ProductSearchPage",
            "('/products', name: 'productSearch', guards: [])",
            vec![
                constructor_param("query", TypeIr::string().nullable()),
                defaulted_param("page", TypeIr::int()),
                defaulted_param_source("showArchived", TypeIr::bool(), "false"),
            ],
        ),
        route_page_class(
            "ProductPickerPage",
            "('/product-picker', name: 'productPicker', result: int, guards: [], transition: BottomToTopPageTransitionsBuilder(), fullscreenDialog: true)",
            Vec::new(),
        ),
        route_page_class(
            "InvitePage",
            "('/invite/:code', name: 'invite', guards: [])",
            vec![
                constructor_param("code", TypeIr::string()),
                constructor_param("team", TypeIr::string().nullable()),
            ],
        ),
        route_page_class(
            "OrgProjectPage",
            "('/orgs/:orgId/projects/:projectId', name: 'orgProject')",
            vec![
                constructor_param("orgId", TypeIr::string()),
                constructor_param("projectId", TypeIr::int()),
                constructor_param("tab", TypeIr::string().nullable()),
            ],
        ),
        route_page_class(
            "SetupPage",
            "('/setup', name: 'setup', guards: [], shell: SetupShell)",
            Vec::new(),
        ),
        route_page_class(
            "SetupConnectPage",
            "('/setup/connect', name: 'setupConnect')",
            Vec::new(),
        ),
        route_page_class(
            "AdminPage",
            "('/admin', name: 'admin', guards: [AdminGuard])",
            Vec::new(),
        ),
        guard_class("AdminGuard", Vec::new()),
    ]);

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let primary = contribution.primary_source.expect("primary route output");

    assert_snapshot("common_app_route_use_cases.dart.snapshot", &primary);
}

#[test]
fn emits_escaped_branch_literals() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/dashboard', notFound: '/404')"),
        route_page_class(
            "DashboardPage",
            r#"('/dashboard', name: 'dashboard', branch: r"team's-$main")"#,
            Vec::new(),
        ),
        route_page_class("NotFoundPage", "('/404', name: 'notFound')", Vec::new()),
    ]);

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let primary = contribution.primary_source.expect("primary route output");

    assert_snapshot("escaped_branch_literals.dart.snapshot", &primary);
}

#[test]
fn rejects_generated_route_class_name_collisions() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/orders/detail', notFound: '/404')"),
        route_page_class(
            "OrderDetailPage",
            "('/orders/detail', name: 'orderDetail')",
            Vec::new(),
        ),
        route_page_class(
            "OrderDetailSlugPage",
            "('/order-details/:id', name: 'order_detail')",
            vec![constructor_param("id", TypeIr::string())],
        ),
    ]);

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec!["generated route class `OrderDetailRoute` is emitted by more than one route name"]
    );
}

#[test]
fn rejects_reserved_route_helper_names() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/switch', notFound: '/404')"),
        route_page_class("SwitchPage", "('/switch', name: 'switch')", Vec::new()),
    ]);

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec!["route name `switch` must be a valid non-reserved Dart identifier"]
    );
}

#[test]
fn rejects_invalid_route_helper_identifiers() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/orders/detail', notFound: '/404')"),
        route_page_class(
            "OrderDetailPage",
            "('/orders/detail', name: 'order-detail')",
            Vec::new(),
        ),
    ]);

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec!["route name `order-detail` must be a valid non-reserved Dart identifier"]
    );
}

#[test]
fn rejects_route_helper_name_that_conflicts_with_navigator_pop() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/pop', notFound: '/404')"),
        route_page_class("PopPage", "('/pop', name: 'pop')", Vec::new()),
    ]);

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec!["route name `pop` conflicts with the generated navigator `pop` helper"]
    );
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

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let primary = contribution.primary_source.expect("primary route output");

    assert_snapshot("custom_router_guard_route.dart.snapshot", &primary);
}

#[test]
fn emits_nested_guarded_route_restore_fixture() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/', notFound: '/404')"),
        route_page_class("HomePage", "('/', name: 'home', guards: [])", Vec::new()),
        route_page_class(
            "WorkspacePage",
            "('/workspace', name: 'workspace', guards: [AuthGuard])",
            Vec::new(),
        ),
        route_page_class(
            "WorkspaceDetailsPage",
            "('/workspace/details', name: 'workspaceDetails')",
            Vec::new(),
        ),
        guard_class("AuthGuard", Vec::new()),
    ]);

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let primary = contribution.primary_source.expect("primary route output");

    assert_snapshot("nested_guarded_restore_route.dart.snapshot", &primary);
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

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

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

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "route guard `AuthGuard` constructor parameter `predicate` needs a resolvable type for router injection"
        ]
    );
}

#[test]
fn rejects_duplicate_path_params_before_emitting_parser() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/users/:id/posts/:id', notFound: '/404')"),
        route_page_class(
            "PostPage",
            "('/users/:id/posts/:id', name: 'post')",
            vec![constructor_param("id", TypeIr::int())],
        ),
    ]);

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "route `PostPage` path `/users/:id/posts/:id` declares duplicate path parameter `:id`"
        ]
    );
}

#[test]
fn rejects_static_and_dynamic_route_siblings_before_emitting_parser() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/users/settings', notFound: '/404')"),
        route_page_class(
            "UserPage",
            "('/users/:id', name: 'user')",
            vec![constructor_param("id", TypeIr::int())],
        ),
        route_page_class(
            "UserSettingsPage",
            "('/users/settings', name: 'userSettings')",
            Vec::new(),
        ),
    ]);

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "route path `/users/settings` conflicts with sibling `/users/:id`; static and dynamic segments under `/users` are ambiguous"
        ]
    );
}

#[test]
fn allows_deeper_static_route_beside_shorter_dynamic_route() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/users/settings/profile', notFound: '/404')"),
        route_page_class(
            "UserPage",
            "('/users/:id', name: 'user')",
            vec![constructor_param("id", TypeIr::int())],
        ),
        route_page_class(
            "UserSettingsProfilePage",
            "('/users/settings/profile', name: 'userSettingsProfile')",
            Vec::new(),
        ),
    ]);

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert!(
        contribution.diagnostics.is_empty(),
        "{:?}",
        contribution.diagnostics
    );
    assert!(contribution.primary_source.is_some());
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

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext { symbol_plan: &plan },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
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

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext { symbol_plan: &plan },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
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

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
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

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
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
