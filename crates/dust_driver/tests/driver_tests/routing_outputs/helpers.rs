use std::{fs, path::PathBuf};

use crate::support::write_file;

pub(crate) fn write_routing_workspace(root: &std::path::Path, dashboard_name: &str) {
    write_file(
        &root.join("lib/route.dart"),
        "import 'pages/dashboard_page.dart';\n\
         import 'pages/not_found_page.dart';\n\
         import 'route.g.dart';\n\
         \n\
         @AppRouter(initial: '/', notFound: '/404')\n\
         final class TestRouter extends $TestRouter {\n\
           const TestRouter();\n\
         }\n",
    );
    write_dashboard_page(root, dashboard_name);
    write_file(
        &root.join("lib/pages/not_found_page.dart"),
        "@AppRoute('/404', name: 'notFound', guards: [])\n\
         final class NotFoundPage {\n\
           const NotFoundPage({this.path = ''});\n\
           final String path;\n\
         }\n",
    );
}

pub(crate) fn write_dashboard_page(root: &std::path::Path, name: &str) {
    write_file(
        &root.join("lib/pages/dashboard_page.dart"),
        &format!(
            "@AppRoute('/', name: '{name}')\n\
             final class DashboardPage {{\n\
               const DashboardPage();\n\
             }}\n"
        ),
    );
}

pub(crate) fn assert_route_snapshot(name: &str, actual: &str) {
    let path = snapshot_path(name);
    if std::env::var_os("DUST_UPDATE_DRIVER_ROUTE_SNAPSHOTS").is_some() {
        fs::write(&path, actual).unwrap();
    }
    let expected = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("missing route snapshot `{}`: {error}", path.display()));
    assert_eq!(actual, expected, "route snapshot `{name}` changed");
}

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/driver_tests/routing_outputs/snapshots")
        .join(name)
}
