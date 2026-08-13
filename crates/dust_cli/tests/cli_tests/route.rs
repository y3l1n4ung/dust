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

#[test]
fn cli_route_graph_prints_parent_route_nodes() {
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
        "graph",
        "--root",
        workspace.path().to_str().unwrap(),
    ]);

    assert_eq!(run.exit_code, 0, "{}", run.stderr);
    assert_eq!(run.stderr, "");
    assert_eq!(
        route_body(&run.stdout),
        "route graph  scanned: 5  routes: 4  time: <ms>\n\
         path | parent | name | page | shell | branch | guards\n\
         --- | --- | --- | --- | --- | --- | ---\n\
         /404 | - | notFound | NotFoundPage | - | - | -\n\
         /checkout | - | checkout | CheckoutPage | - | - | CartGuard\n\
         /dashboard | - | dashboard | DashboardPage | AppShell | mainTabs | -\n\
         /dashboard/orders | /dashboard | orders | OrdersPage | AppShell | mainTabs | -\n"
    );
}

#[test]
fn cli_route_fixtures_prints_valid_and_invalid_deep_links() {
    let workspace = make_workspace();
    write_dust_file(
        &workspace.path().join("lib/route.dart"),
        &[DustImport::Route],
        "import 'pages/not_found_page.dart';\n\
         import 'pages/product_page.dart';\n\
         @AppRouter(initial: '/products/:productId', notFound: '/404')\n\
         final class TestRouter extends $TestRouter {}\n",
    );
    write_dust_file(
        &workspace.path().join("lib/pages/not_found_page.dart"),
        &[DustImport::Route],
        "@AppRoute('/404', name: 'notFound')\n\
         final class NotFoundPage {\n\
           const NotFoundPage({this.path = ''});\n\
           final String path;\n\
         }\n",
    );
    write_dust_file(
        &workspace.path().join("lib/pages/product_page.dart"),
        &[DustImport::Route],
        "@AppRoute('/products/:productId', name: 'product')\n\
         final class ProductPage {\n\
           const ProductPage({\n\
             required this.productId,\n\
             required this.from,\n\
             this.tab,\n\
           });\n\
           final int productId;\n\
           final DateTime from;\n\
           final String? tab;\n\
         }\n",
    );

    let run = run_cli([
        "route",
        "fixtures",
        "--root",
        workspace.path().to_str().unwrap(),
    ]);

    assert_eq!(run.exit_code, 0, "{}", run.stderr);
    assert_eq!(run.stderr, "");
    assert_eq!(
        route_body(&run.stdout),
        "route fixtures  scanned: 3  fixtures: 15  time: <ms>\n\
         route | case | valid | shape | uri | expected\n\
         --- | --- | --- | --- | --- | ---\n\
         notFound | path | true | path | /404?path=path-sample | typed-route\n\
         notFound | web-url | true | web-url | https://shop.example/404?path=path-sample | normalize-then-typed-route\n\
         notFound | app-link | true | app-link | https://shop.example/app/404?path=path-sample | normalize-prefix-then-typed-route\n\
         notFound | custom-scheme | true | custom-scheme | shopping:///404?path=path-sample | normalize-scheme-then-typed-route\n\
         notFound | fragment-preserved | true | path | /404?path=path-sample#details | typed-route-preserve-fragment\n\
         product | path | true | path | /products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews | typed-route\n\
         product | web-url | true | web-url | https://shop.example/products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews | normalize-then-typed-route\n\
         product | app-link | true | app-link | https://shop.example/app/products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews | normalize-prefix-then-typed-route\n\
         product | custom-scheme | true | custom-scheme | shopping:///products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews | normalize-scheme-then-typed-route\n\
         product | fragment-preserved | true | path | /products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews#details | typed-route-preserve-fragment\n\
         product | invalid-path-param | false | path | /products/not-an-int?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews | not-found-route\n\
         product | missing-query-from | false | path | /products/42?tab=reviews | not-found-route\n\
         product | invalid-query-from | false | path | /products/42?from=not-a-date&tab=reviews | not-found-route\n\
         - | not-found | false | path | /__dust_missing_route__ | not-found-route\n\
         - | malformed-fragment-escape | false | path | /#%ZZ | uri-parse-error-before-router\n"
    );
}

fn route_table_body(output: &str) -> String {
    route_body(output)
}

fn route_body(output: &str) -> String {
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
