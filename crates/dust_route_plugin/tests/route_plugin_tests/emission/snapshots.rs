use dust_ir::TypeIr;
use std::path::PathBuf;

use crate::support::{
    constructor_param, defaulted_param, defaulted_param_source, guard_class, library_with_classes,
    route_page_class, router_class,
};

use super::support::{assert_route_snapshot, generate_route_output, route_outputs_snapshot};

#[test]
fn emits_standalone_route_and_core_outputs() {
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

    let contribution = generate_route_output(&library);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("standalone_route.dart.snapshot", &output);
}

#[test]
fn emits_route_outputs_under_relative_package_root() {
    let mut library = library_with_classes(vec![
        router_class("(initial: '/', notFound: '/404')"),
        route_page_class("DashboardPage", "('/', name: 'dashboard')", Vec::new()),
    ]);
    library.package_root = "examples/shopping_app".to_owned();

    let contribution = generate_route_output(&library);
    let actual_paths = contribution
        .auxiliary_outputs
        .iter()
        .map(|output| output.output_path.clone())
        .collect::<Vec<_>>();

    assert_eq!(
        actual_paths,
        vec![
            PathBuf::from("examples/shopping_app/lib/route/routes.g.dart"),
            PathBuf::from("examples/shopping_app/lib/route/paths.g.dart"),
            PathBuf::from("examples/shopping_app/lib/route/metadata.g.dart"),
            PathBuf::from("examples/shopping_app/lib/route/navigation.g.dart"),
            PathBuf::from("examples/shopping_app/lib/route/runtime.g.dart"),
        ]
    );
}

#[test]
fn emits_no_transition_builder_only_when_referenced() {
    let library = library_with_classes(vec![
        router_class("(initial: '/search', notFound: '/404')"),
        route_page_class(
            "SearchPage",
            "('/search', name: 'search', transition: _$NoTransitionBuilder())",
            Vec::new(),
        ),
    ]);

    let contribution = generate_route_output(&library);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("no_transition_route.dart.snapshot", &output);
}

#[test]
fn emits_typed_route_result_helpers() {
    let library = library_with_classes(vec![
        router_class("(initial: '/', notFound: '/404')"),
        route_page_class("HomePage", "('/', name: 'home')", Vec::new()),
        route_page_class(
            "PickerPage",
            "('/picker', name: 'picker', result: bool)",
            Vec::new(),
        ),
    ]);

    let contribution = generate_route_output(&library);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("typed_route_result.dart.snapshot", &output);
}

#[test]
fn emits_inherited_shell_without_extra_annotations() {
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

    let contribution = generate_route_output(&library);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("inherited_shell_route.dart.snapshot", &output);
}

#[test]
fn emits_nearest_shell_when_child_route_overrides_parent_shell() {
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

    let contribution = generate_route_output(&library);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("shell_override_route.dart.snapshot", &output);
}

#[test]
fn emits_common_app_route_use_cases() {
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

    let contribution = generate_route_output(&library);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("common_app_route_use_cases.dart.snapshot", &output);
}

#[test]
fn emits_escaped_branch_literals() {
    let library = library_with_classes(vec![
        router_class("(initial: '/dashboard', notFound: '/404')"),
        route_page_class(
            "DashboardPage",
            r#"('/dashboard', name: 'dashboard', branch: r"team's-$main")"#,
            Vec::new(),
        ),
        route_page_class("NotFoundPage", "('/404', name: 'notFound')", Vec::new()),
    ]);

    let contribution = generate_route_output(&library);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("escaped_branch_literals.dart.snapshot", &output);
}

#[test]
fn emits_guard_helpers_with_custom_router_base_name() {
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

    let contribution = generate_route_output(&library);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("custom_router_guard_route.dart.snapshot", &output);
}

#[test]
fn emits_nested_guarded_route_restore_fixture() {
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

    let contribution = generate_route_output(&library);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("nested_guarded_restore_route.dart.snapshot", &output);
}
