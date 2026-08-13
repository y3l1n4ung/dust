use dust_driver::{RouteGraphNode, RouteGraphRequest, run_route_graph};

use super::support::{DustImport, write_dust_file};
use crate::support::make_workspace;

#[test]
fn route_graph_returns_parent_route_nodes() {
    let workspace = make_workspace();
    write_dust_file(
        &workspace.path().join("lib/route.dart"),
        &[DustImport::Route],
        "import 'pages/dashboard_page.dart';\n\
         import 'pages/orders_page.dart';\n\
         import 'pages/checkout_page.dart';\n\
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
        &workspace.path().join("lib/pages/not_found_page.dart"),
        &[DustImport::Route],
        "@AppRoute('/404', name: 'notFound', guards: [])\n\
         final class NotFoundPage {\n\
           const NotFoundPage({this.path = ''});\n\
           final String path;\n\
         }\n",
    );

    let result = run_route_graph(RouteGraphRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert_eq!(result.diagnostics, []);
    let graph = result.route_graph.expect("route graph report");
    assert_eq!(graph.scanned_files, 5);
    assert_eq!(
        graph.nodes,
        vec![
            RouteGraphNode {
                name: "notFound".to_owned(),
                path: "/404".to_owned(),
                parent_path: None,
                page: "NotFoundPage".to_owned(),
                shell: None,
                branch: None,
                guards: Vec::new(),
            },
            RouteGraphNode {
                name: "checkout".to_owned(),
                path: "/checkout".to_owned(),
                parent_path: None,
                page: "CheckoutPage".to_owned(),
                shell: None,
                branch: None,
                guards: vec!["CartGuard".to_owned()],
            },
            RouteGraphNode {
                name: "dashboard".to_owned(),
                path: "/dashboard".to_owned(),
                parent_path: None,
                page: "DashboardPage".to_owned(),
                shell: Some("AppShell".to_owned()),
                branch: Some("mainTabs".to_owned()),
                guards: Vec::new(),
            },
            RouteGraphNode {
                name: "orders".to_owned(),
                path: "/dashboard/orders".to_owned(),
                parent_path: Some("/dashboard".to_owned()),
                page: "OrdersPage".to_owned(),
                shell: Some("AppShell".to_owned()),
                branch: Some("mainTabs".to_owned()),
                guards: Vec::new(),
            },
        ]
    );
}
