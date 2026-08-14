use dust_driver::{RouteTableRequest, RouteTableRow, run_route_table};

use super::support::{DustImport, write_dust_file};
use crate::support::make_workspace;

#[test]
fn route_table_returns_effective_route_rows() {
    let workspace = make_workspace();
    write_dust_file(
        &workspace.path().join("lib/route.dart"),
        &[DustImport::Route],
        "import 'pages/dashboard_page.dart';\n\
         import 'pages/orders_page.dart';\n\
         import 'pages/checkout_page.dart';\n\
         import 'pages/login_page.dart';\n\
         import 'pages/not_found_page.dart';\n\
         @AppRouter(initial: '/dashboard', notFound: '/404')\n\
         final class TestRouter extends $TestRouter {}\n",
    );
    write_dust_file(
        &workspace.path().join("lib/pages/dashboard_page.dart"),
        &[DustImport::Route],
        "@AppRoute('/dashboard', name: 'dashboard', shell: AppShell, branch: 'mainTabs')\n\
         final class DashboardPage {\n\
           const DashboardPage();\n\
         }\n\
         final class AppShell {\n\
           const AppShell({required Widget child});\n\
         }\n",
    );
    write_dust_file(
        &workspace.path().join("lib/pages/orders_page.dart"),
        &[DustImport::Route],
        "@AppRoute('/dashboard/orders', name: 'orders')\n\
         final class OrdersPage {\n\
           const OrdersPage();\n\
         }\n",
    );
    write_dust_file(
        &workspace.path().join("lib/pages/checkout_page.dart"),
        &[DustImport::Route],
        "@AppRoute('/checkout', name: 'checkout', guards: [CartGuard], result: bool)\n\
         final class CheckoutPage {\n\
           const CheckoutPage();\n\
         }\n\
         final class CartGuard {\n\
           const CartGuard();\n\
         }\n",
    );
    write_dust_file(
        &workspace.path().join("lib/pages/login_page.dart"),
        &[DustImport::Route],
        "@AppRoute('/login', name: 'login', guards: [])\n\
         final class LoginPage {\n\
           const LoginPage();\n\
         }\n",
    );
    write_dust_file(
        &workspace.path().join("lib/pages/not_found_page.dart"),
        &[DustImport::Route],
        "@AppRoute('/404', name: 'notFound', guards: [])\n\
         final class NotFoundPage {\n\
           const NotFoundPage({this.path = ''});\n\
           final String path;\n\
         }\n",
    );

    let result = run_route_table(RouteTableRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert_eq!(result.diagnostics, []);
    let table = result.route_table.expect("route table report");
    assert_eq!(table.scanned_files, 6);
    assert_eq!(
        table.routes,
        vec![
            RouteTableRow {
                name: "notFound".to_owned(),
                path: "/404".to_owned(),
                page: "NotFoundPage".to_owned(),
                shell: None,
                branch: None,
                guards: Vec::new(),
                requires_auth: false,
                result_type: "void".to_owned(),
            },
            RouteTableRow {
                name: "checkout".to_owned(),
                path: "/checkout".to_owned(),
                page: "CheckoutPage".to_owned(),
                shell: None,
                branch: None,
                guards: vec!["CartGuard".to_owned()],
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
                name: "orders".to_owned(),
                path: "/dashboard/orders".to_owned(),
                page: "OrdersPage".to_owned(),
                shell: Some("AppShell".to_owned()),
                branch: Some("mainTabs".to_owned()),
                guards: Vec::new(),
                requires_auth: true,
                result_type: "void".to_owned(),
            },
            RouteTableRow {
                name: "login".to_owned(),
                path: "/login".to_owned(),
                page: "LoginPage".to_owned(),
                shell: None,
                branch: None,
                guards: Vec::new(),
                requires_auth: false,
                result_type: "void".to_owned(),
            },
        ]
    );
}

#[test]
fn route_table_marks_router_not_found_route_public() {
    let workspace = make_workspace();
    write_dust_file(
        &workspace.path().join("lib/route.dart"),
        &[DustImport::Route],
        "import 'pages/home_page.dart';\n\
         import 'pages/missing_page.dart';\n\
         @AppRouter(initial: '/', notFound: '/404')\n\
         final class TestRouter extends $TestRouter {}\n",
    );
    write_dust_file(
        &workspace.path().join("lib/pages/home_page.dart"),
        &[DustImport::Route],
        "@AppRoute('/', name: 'home')\n\
         final class HomePage {\n\
           const HomePage();\n\
         }\n",
    );
    write_dust_file(
        &workspace.path().join("lib/pages/missing_page.dart"),
        &[DustImport::Route],
        "@AppRoute('/404', name: 'notFound')\n\
         final class MissingPage {\n\
           const MissingPage({this.path = ''});\n\
           final String path;\n\
         }\n",
    );

    let result = run_route_table(RouteTableRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert_eq!(result.diagnostics, []);
    let table = result.route_table.expect("route table report");
    let auth_by_path = table
        .routes
        .iter()
        .map(|route| (route.path.as_str(), route.requires_auth))
        .collect::<Vec<_>>();

    assert_eq!(auth_by_path, vec![("/", true), ("/404", false)]);
}
