use dust_driver::{RouteFixtureRow, RouteFixturesRequest, run_route_fixtures};

use super::support::{DustImport, write_dust_file};
use crate::support::make_workspace;

#[test]
fn route_fixtures_return_valid_and_invalid_deep_links() {
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
        "enum ProductMode { active, archived }\n\
         @AppRoute('/products/:productId', name: 'product')\n\
         final class ProductPage {\n\
           const ProductPage({\n\
             required this.productId,\n\
             required this.from,\n\
             this.tab,\n\
             this.preview = false,\n\
             this.tags = const <String>[],\n\
             this.quantities = const <int>[],\n\
             required this.receipt,\n\
             this.mode = ProductMode.active,\n\
           });\n\
           final int productId;\n\
           final DateTime from;\n\
           final String? tab;\n\
           final bool preview;\n\
           final List<String> tags;\n\
           final List<int> quantities;\n\
           final Uri receipt;\n\
           final ProductMode mode;\n\
         }\n",
    );

    let result = run_route_fixtures(RouteFixturesRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert_eq!(result.diagnostics, []);
    let report = result.route_fixtures.expect("route fixtures report");
    assert_eq!(report.scanned_files, 3);
    assert_eq!(
        report.fixtures,
        vec![
            fixture(
                "notFound",
                "path",
                true,
                "path",
                "/404?path=path-sample",
                "typed-route",
            ),
            fixture(
                "notFound",
                "web-url",
                true,
                "web-url",
                "https://shop.example/404?path=path-sample",
                "normalize-then-typed-route",
            ),
            fixture(
                "notFound",
                "app-link",
                true,
                "app-link",
                "https://shop.example/app/404?path=path-sample",
                "normalize-prefix-then-typed-route",
            ),
            fixture(
                "notFound",
                "custom-scheme",
                true,
                "custom-scheme",
                "shopping:///404?path=path-sample",
                "normalize-scheme-then-typed-route",
            ),
            fixture(
                "notFound",
                "fragment-preserved",
                true,
                "path",
                "/404?path=path-sample#details",
                "typed-route-preserve-fragment",
            ),
            fixture(
                "product",
                "path",
                true,
                "path",
                "/products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews&preview=true&tags=sale&tags=new&quantities=1&quantities=2&receipt=https%3A%2F%2Fexample.com%2Freceipt&mode=active",
                "typed-route",
            ),
            fixture(
                "product",
                "web-url",
                true,
                "web-url",
                "https://shop.example/products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews&preview=true&tags=sale&tags=new&quantities=1&quantities=2&receipt=https%3A%2F%2Fexample.com%2Freceipt&mode=active",
                "normalize-then-typed-route",
            ),
            fixture(
                "product",
                "app-link",
                true,
                "app-link",
                "https://shop.example/app/products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews&preview=true&tags=sale&tags=new&quantities=1&quantities=2&receipt=https%3A%2F%2Fexample.com%2Freceipt&mode=active",
                "normalize-prefix-then-typed-route",
            ),
            fixture(
                "product",
                "custom-scheme",
                true,
                "custom-scheme",
                "shopping:///products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews&preview=true&tags=sale&tags=new&quantities=1&quantities=2&receipt=https%3A%2F%2Fexample.com%2Freceipt&mode=active",
                "normalize-scheme-then-typed-route",
            ),
            fixture(
                "product",
                "fragment-preserved",
                true,
                "path",
                "/products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews&preview=true&tags=sale&tags=new&quantities=1&quantities=2&receipt=https%3A%2F%2Fexample.com%2Freceipt&mode=active#details",
                "typed-route-preserve-fragment",
            ),
            fixture(
                "product",
                "invalid-path-param",
                false,
                "path",
                "/products/not-an-int?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews&preview=true&tags=sale&tags=new&quantities=1&quantities=2&receipt=https%3A%2F%2Fexample.com%2Freceipt&mode=active",
                "not-found-route",
            ),
            fixture(
                "product",
                "missing-query-from",
                false,
                "path",
                "/products/42?tab=reviews&preview=true&tags=sale&tags=new&quantities=1&quantities=2&receipt=https%3A%2F%2Fexample.com%2Freceipt&mode=active",
                "not-found-route",
            ),
            fixture(
                "product",
                "invalid-query-from",
                false,
                "path",
                "/products/42?from=not-a-date&tab=reviews&preview=true&tags=sale&tags=new&quantities=1&quantities=2&receipt=https%3A%2F%2Fexample.com%2Freceipt&mode=active",
                "not-found-route",
            ),
            fixture(
                "product",
                "invalid-query-preview",
                false,
                "path",
                "/products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews&preview=not-a-bool&tags=sale&tags=new&quantities=1&quantities=2&receipt=https%3A%2F%2Fexample.com%2Freceipt&mode=active",
                "not-found-route",
            ),
            fixture(
                "product",
                "invalid-query-quantities",
                false,
                "path",
                "/products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews&preview=true&tags=sale&tags=new&quantities=not-an-int&receipt=https%3A%2F%2Fexample.com%2Freceipt&mode=active",
                "not-found-route",
            ),
            fixture(
                "product",
                "missing-query-receipt",
                false,
                "path",
                "/products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews&preview=true&tags=sale&tags=new&quantities=1&quantities=2&mode=active",
                "not-found-route",
            ),
            fixture(
                "product",
                "invalid-query-receipt",
                false,
                "path",
                "/products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews&preview=true&tags=sale&tags=new&quantities=1&quantities=2&receipt=%ZZ&mode=active",
                "not-found-route",
            ),
            fixture(
                "product",
                "invalid-query-mode",
                false,
                "path",
                "/products/42?from=2026-08-10T09%3A30%3A00.000Z&tab=reviews&preview=true&tags=sale&tags=new&quantities=1&quantities=2&receipt=https%3A%2F%2Fexample.com%2Freceipt&mode=not-a-valid-value",
                "not-found-route",
            ),
            fixture(
                "-",
                "not-found",
                false,
                "path",
                "/__dust_missing_route__",
                "not-found-route",
            ),
        ]
    );
}

fn fixture(
    route: &str,
    case_name: &str,
    valid: bool,
    shape: &str,
    uri: &str,
    expected: &str,
) -> RouteFixtureRow {
    RouteFixtureRow {
        route: route.to_owned(),
        case_name: case_name.to_owned(),
        valid,
        shape: shape.to_owned(),
        uri: uri.to_owned(),
        expected: expected.to_owned(),
    }
}
