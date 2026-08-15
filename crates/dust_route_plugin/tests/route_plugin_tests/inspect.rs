use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, WorkspaceAnalysisBuilder};
use dust_route_plugin::{
    inspect::{RouteTableRow, route_table_rows},
    register_plugin,
};
use serde_json::json;

use super::support::{constructor_param, library_with_classes, route_page_class, router_class};

const ROUTES_KEY: &str = "dust_route.routes.v1";
const ROUTERS_KEY: &str = "dust_route.routers.v1";

#[test]
fn route_table_rows_cover_canonical_route_analysis() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        router_class("(initial: '/dashboard', notFound: '/404')"),
        route_page_class(
            "DashboardPage",
            "('/dashboard', name: 'dashboard', shell: AppShell, branch: 'mainTabs')",
            Vec::new(),
        ),
        route_page_class("DashboardOrdersPage", "('/dashboard/orders')", Vec::new()),
        route_page_class("AccountScreen", "('/account', guards: [])", Vec::new()),
        route_page_class(
            "CheckoutView",
            "('/checkout', guards: [CheckoutGuard], result: bool)",
            vec![constructor_param("cartId", TypeIr::string())],
        ),
    ]);
    let mut builder = WorkspaceAnalysisBuilder::default();
    plugin.collect_workspace_analysis_ir(&library, &mut builder);

    let rows = route_table_rows(&[builder.snapshot()]);

    assert_eq!(
        rows,
        vec![
            row("notFound", "/404", "NotFoundPage"),
            RouteTableRow {
                name: "account".to_owned(),
                path: "/account".to_owned(),
                page: "AccountScreen".to_owned(),
                shell: None,
                branch: None,
                guards: Vec::new(),
                requires_auth: false,
                result_type: "void".to_owned(),
            },
            RouteTableRow {
                name: "checkout".to_owned(),
                path: "/checkout".to_owned(),
                page: "CheckoutView".to_owned(),
                shell: None,
                branch: None,
                guards: vec!["CheckoutGuard".to_owned()],
                requires_auth: true,
                result_type: "bool".to_owned(),
            },
            RouteTableRow {
                name: "dashboard".to_owned(),
                path: "/dashboard".to_owned(),
                page: "DashboardPage".to_owned(),
                shell: Some("AppShell".to_owned()),
                branch: Some("mainTabs".to_owned()),
                guards: Vec::new(),
                requires_auth: true,
                result_type: "void".to_owned(),
            },
            RouteTableRow {
                name: "dashboardOrders".to_owned(),
                path: "/dashboard/orders".to_owned(),
                page: "DashboardOrdersPage".to_owned(),
                shell: Some("AppShell".to_owned()),
                branch: Some("mainTabs".to_owned()),
                guards: Vec::new(),
                requires_auth: true,
                result_type: "void".to_owned(),
            },
        ]
    );
}

#[test]
fn route_table_rows_ignore_bad_facts_and_use_nearest_inherited_metadata() {
    let root = route_fact(
        "SettingsPage",
        "/settings",
        None,
        Some("SettingsShell"),
        Some("settingsTabs"),
    );
    let account = route_fact(
        "AccountView",
        "/settings/account",
        None,
        Some("AccountShell"),
        None,
    );
    let security = route_fact(
        "SecurityScreen",
        "/settings/account/security",
        Some("security"),
        None,
        None,
    );
    let empty = route_fact("", "/empty", None, None, None);
    let legacy = route_fact("legacy_admin", "/legacy", None, None, None);
    let not_found = route_fact("MissingPage", "/missing", None, None, None);

    let rows = route_table_rows(&[
        snapshot([
            (ROUTES_KEY, "not json".to_owned()),
            (ROUTERS_KEY, r#"{"not_found":42}"#.to_owned()),
            (ROUTES_KEY, root),
            (ROUTES_KEY, security),
        ]),
        snapshot([
            (ROUTES_KEY, account),
            (ROUTES_KEY, empty),
            (ROUTES_KEY, legacy),
            (ROUTES_KEY, not_found),
            (
                ROUTERS_KEY,
                json!({
                    "class_name": "ShopRouter",
                    "initial": "/settings",
                    "not_found": "/missing",
                    "source_path": "lib/route.dart"
                })
                .to_string(),
            ),
        ]),
    ]);

    assert_eq!(
        rows,
        vec![
            RouteTableRow {
                name: String::new(),
                path: "/empty".to_owned(),
                page: String::new(),
                shell: None,
                branch: None,
                guards: Vec::new(),
                requires_auth: true,
                result_type: "void".to_owned(),
            },
            RouteTableRow {
                name: "legacyAdmin".to_owned(),
                path: "/legacy".to_owned(),
                page: "legacy_admin".to_owned(),
                shell: None,
                branch: None,
                guards: Vec::new(),
                requires_auth: true,
                result_type: "void".to_owned(),
            },
            RouteTableRow {
                name: "missing".to_owned(),
                path: "/missing".to_owned(),
                page: "MissingPage".to_owned(),
                shell: None,
                branch: None,
                guards: Vec::new(),
                requires_auth: false,
                result_type: "void".to_owned(),
            },
            RouteTableRow {
                name: "settings".to_owned(),
                path: "/settings".to_owned(),
                page: "SettingsPage".to_owned(),
                shell: Some("SettingsShell".to_owned()),
                branch: Some("settingsTabs".to_owned()),
                guards: Vec::new(),
                requires_auth: true,
                result_type: "void".to_owned(),
            },
            RouteTableRow {
                name: "account".to_owned(),
                path: "/settings/account".to_owned(),
                page: "AccountView".to_owned(),
                shell: Some("AccountShell".to_owned()),
                branch: Some("settingsTabs".to_owned()),
                guards: Vec::new(),
                requires_auth: true,
                result_type: "void".to_owned(),
            },
            RouteTableRow {
                name: "security".to_owned(),
                path: "/settings/account/security".to_owned(),
                page: "SecurityScreen".to_owned(),
                shell: Some("AccountShell".to_owned()),
                branch: Some("settingsTabs".to_owned()),
                guards: Vec::new(),
                requires_auth: true,
                result_type: "void".to_owned(),
            },
        ]
    );
}

fn row(name: &str, path: &str, page: &str) -> RouteTableRow {
    RouteTableRow {
        name: name.to_owned(),
        path: path.to_owned(),
        page: page.to_owned(),
        shell: None,
        branch: None,
        guards: Vec::new(),
        requires_auth: false,
        result_type: "void".to_owned(),
    }
}

fn route_fact(
    class_name: &str,
    path: &str,
    name: Option<&str>,
    shell: Option<&str>,
    branch: Option<&str>,
) -> String {
    json!({
        "class_name": class_name,
        "path": path,
        "name": name,
        "annotation": {
            "path": path,
            "name": name,
            "result_type": null,
            "shell": shell,
            "branch": branch,
            "guards": [],
            "guards_configured": false,
            "transition": null,
            "fullscreen_dialog": false,
            "maintain_state": true
        },
        "import_uri": "package:route_test/route.dart",
        "source_path": "lib/route.dart",
        "imports": [],
        "params": []
    })
    .to_string()
}

fn snapshot<const N: usize>(
    values: [(&str, String); N],
) -> dust_plugin_api::LibraryAnalysisSnapshot {
    let mut builder = WorkspaceAnalysisBuilder::default();
    for (key, value) in values {
        builder.add_string_set_value(key, value);
    }
    builder.snapshot()
}
