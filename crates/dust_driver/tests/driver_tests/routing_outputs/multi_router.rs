use std::fs;

use dust_driver::{BuildRequest, run_build};

use super::helpers::write_routing_workspace;
use crate::support::{make_workspace, write_file};

#[test]
fn multiple_router_roots_emit_separate_outputs() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");
    write_file(
        &workspace.path().join("lib/admin_route.dart"),
        "import 'pages/admin_page.dart';\n\
         import 'pages/not_found_page.dart';\n\
         import 'admin_route.g.dart';\n\
         \n\
         @Router(initial: AdminPage, notFound: NotFoundPage)\n\
         final class AdminRouter extends $AdminRouter {\n\
           const AdminRouter();\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/pages/admin_page.dart"),
        "@Route('/admin', name: 'admin')\n\
         final class AdminPage {\n\
           const AdminPage();\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let app_source = fs::read_to_string(workspace.path().join("lib/route.g.dart")).unwrap();
    let admin_source = fs::read_to_string(workspace.path().join("lib/admin_route.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(app_source.contains("abstract class $AppRouter"));
    assert!(app_source.contains("initialRoute: const DashboardRoute()"));
    assert!(admin_source.contains("abstract class $AdminRouter"));
    assert!(admin_source.contains("initialRoute: const AdminRoute()"));
    assert!(admin_source.contains("import 'package:dust_test/pages/admin_page.dart';"));
}
