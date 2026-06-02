use std::fs;

use dust_driver::{BuildRequest, run_build};

use super::helpers::write_routing_workspace;
use crate::support::make_workspace;

#[test]
fn build_matches_full_route_output_snapshot() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let source = fs::read_to_string(workspace.path().join("lib/route.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_route_output_matches_contract(&source);
}
fn assert_route_output_matches_contract(source: &str) {
    for expected in [
        "import 'package:dust_router/dust_router.dart';",
        "import 'package:dust_test/pages/dashboard_page.dart';",
        "import 'package:dust_test/pages/not_found_page.dart';",
        "abstract class $AppRouter",
        "sealed class AppRoutePath",
        "final class DashboardRoute extends AppRoutePath",
        "final class NotFoundRoute extends AppRoutePath",
        "GeneratedRoute(",
        "page: DashboardPage",
        "name: 'dashboard'",
        "AppRoutePath parseAppRoute(Uri uri)",
        "Page<void> buildAppRoutePage(AppRoutePath route)",
        "extension DustRouterContext on BuildContext",
        "RouteNavigation<AppRoutePath> dashboard()",
    ] {
        assert!(
            source.contains(expected),
            "generated route output is missing `{expected}`\n{source}"
        );
    }
}
