use dust_cli::run_cli;

use super::helpers::{DustImport, make_workspace, write_dust_file};

#[test]
fn cli_route_table_prints_effective_route_rows() {
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

    let run = run_cli([
        "route",
        "table",
        "--root",
        workspace.path().to_str().unwrap(),
    ]);

    assert_eq!(run.exit_code, 0, "{}", run.stderr);
    assert_eq!(run.stderr, "");
    assert_eq!(
        route_table_body(&run.stdout),
        "route table  scanned: 5  routes: 4  time: <ms>\n\
         name | path | page | shell | branch | guards | result\n\
         --- | --- | --- | --- | --- | --- | ---\n\
         notFound | /404 | NotFoundPage | - | - | - | void\n\
         checkout | /checkout | CheckoutPage | - | - | CartGuard | bool\n\
         dashboard | /dashboard | DashboardPage | AppShell | mainTabs | - | void\n\
         orders | /dashboard/orders | OrdersPage | AppShell | mainTabs | - | void\n"
    );
}

fn route_table_body(output: &str) -> String {
    let body = output.split_once("\n\n").map_or(output, |(_, body)| body);
    body.lines()
        .map(|line| {
            line.find("  time: ").map_or_else(
                || line.to_owned(),
                |index| format!("{}  time: <ms>", &line[..index]),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
