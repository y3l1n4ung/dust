use std::{fs, path::PathBuf};

use crate::support::{DustImport, write_dust_file};

pub(crate) fn write_routing_workspace(root: &std::path::Path, dashboard_name: &str) {
    write_dust_file(
        &root.join("lib/route.dart"),
        &[DustImport::Route],
        "import 'pages/dashboard_page.dart';\n\
         import 'pages/not_found_page.dart';\n\
         import 'route/routes.g.dart';\n\
         export 'route/routes.g.dart';\n\
         \n\
         @AppRouter(initial: '/', notFound: '/404')\n\
         final class TestRouter extends $TestRouter {\n\
           const TestRouter();\n\
         }\n",
    );
    write_dashboard_page(root, dashboard_name);
    write_dust_file(
        &root.join("lib/pages/not_found_page.dart"),
        &[DustImport::Route],
        "@AppRoute('/404', name: 'notFound', guards: [])\n\
         final class NotFoundPage {\n\
           const NotFoundPage({this.path = ''});\n\
           final String path;\n\
         }\n",
    );
}

pub(crate) fn write_dashboard_page(root: &std::path::Path, name: &str) {
    write_dust_file(
        &root.join("lib/pages/dashboard_page.dart"),
        &[DustImport::Route],
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

pub(crate) fn read_route_outputs(root: &std::path::Path) -> String {
    [
        "routes.g.dart",
        "paths.g.dart",
        "metadata.g.dart",
        "navigation.g.dart",
        "runtime.g.dart",
    ]
    .into_iter()
    .map(|name| {
        let source = fs::read_to_string(root.join("lib/route").join(name)).unwrap();
        format!("// file: lib/route/{name}\n{source}")
    })
    .collect::<Vec<_>>()
    .join("\n")
}

pub(crate) fn route_output_paths(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    [
        "routes.g.dart",
        "paths.g.dart",
        "metadata.g.dart",
        "navigation.g.dart",
        "runtime.g.dart",
    ]
    .into_iter()
    .map(|name| root.join("lib/route").join(name))
    .collect()
}

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/driver_tests/routing_outputs/snapshots")
        .join(name)
}
