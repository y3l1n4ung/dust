use std::fs;

use dust_driver::{
    BuildRequest, CheckRequest, CleanRequest, WatchRequest, run_build, run_check, run_clean,
    run_watch,
};

use super::support::{make_pub_workspace_member, make_workspace, write_file};

#[test]
fn build_writes_route_output_only_from_router_root() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });

    let route_output = workspace.path().join("lib/route.g.dart");
    let dashboard_output = workspace.path().join("lib/pages/dashboard_page.g.dart");
    let not_found_output = workspace.path().join("lib/pages/not_found_page.g.dart");
    let source = fs::read_to_string(&route_output).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(route_output.exists());
    assert!(!dashboard_output.exists());
    assert!(!not_found_output.exists());
    assert!(result.build_artifacts.iter().any(|artifact| {
        artifact.source_path.ends_with("dashboard_page.dart")
            && artifact.routed
            && !artifact.written
    }));
    assert!(result.build_artifacts.iter().any(|artifact| {
        artifact.source_path.ends_with("not_found_page.dart")
            && artifact.routed
            && !artifact.written
    }));
    assert!(source.contains("import 'package:dust_router/dust_router.dart';"));
    assert!(source.contains("import 'package:dust_test/pages/dashboard_page.dart';"));
    assert!(source.contains("final class DashboardRoute extends AppRoutePath"));
    assert!(source.contains("page: DashboardPage"));
    assert!(source.contains("name: 'dashboard'"));
}

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

#[test]
fn build_refreshes_router_output_when_annotated_page_changes() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");

    let first = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let second = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let second_source = fs::read_to_string(workspace.path().join("lib/route.g.dart")).unwrap();
    write_dashboard_page(workspace.path(), "home");
    let third = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });

    let source = fs::read_to_string(workspace.path().join("lib/route.g.dart")).unwrap();

    assert!(!first.has_errors(), "{:?}", first.diagnostics);
    assert!(!second.has_errors(), "{:?}", second.diagnostics);
    assert!(!third.has_errors(), "{:?}", third.diagnostics);
    assert_eq!(second.cache.as_ref().unwrap().misses, 0);
    assert!(second_source.contains("page: DashboardPage"));
    assert!(second_source.contains("name: 'dashboard'"));
    assert!(source.contains("page: DashboardPage"));
    assert!(source.contains("name: 'home'"));
    assert!(source.contains("RouteNavigation<AppRoutePath> home()"));
    assert!(!source.contains("RouteNavigation<AppRoutePath> dashboard()"));
}

#[test]
fn check_reports_stale_route_output_before_build_and_fresh_after_build() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");

    let before = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let build = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let after = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });

    assert!(!before.has_errors(), "{:?}", before.diagnostics);
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    assert!(!after.has_errors(), "{:?}", after.diagnostics);
    assert!(
        before
            .checked_libraries
            .iter()
            .any(|library| library.output_path.ends_with("route.g.dart") && library.stale)
    );
    assert!(
        after
            .checked_libraries
            .iter()
            .any(|library| library.output_path.ends_with("route.g.dart") && !library.stale)
    );
}

#[test]
fn clean_removes_route_output_only_from_router_root() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");
    let build = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    assert!(workspace.path().join("lib/route.g.dart").exists());

    let clean = run_clean(CleanRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(!clean.has_errors(), "{:?}", clean.diagnostics);
    assert!(!workspace.path().join("lib/route.g.dart").exists());
    assert!(
        !workspace
            .path()
            .join("lib/pages/dashboard_page.g.dart")
            .exists()
    );
    assert!(clean.clean.unwrap().removed_files >= 1);
}

#[test]
fn watch_rebuilds_route_output_when_annotated_page_changes() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");

    let root = workspace.path().to_path_buf();
    let modifier = std::thread::spawn({
        let root = root.clone();
        move || {
            std::thread::sleep(std::time::Duration::from_millis(1_000));
            write_dashboard_page(&root, "home");
        }
    });
    let result = run_watch(WatchRequest {
        cwd: root.clone(),
        fail_fast: true,
        jobs: None,
        poll_interval_ms: 50,
        max_cycles: Some(30),
    });
    modifier.join().unwrap();

    let source = fs::read_to_string(root.join("lib/route.g.dart")).unwrap();
    let watch = result.watch.as_ref().unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_eq!(watch.rebuild_batches, 1);
    assert!(source.contains("page: DashboardPage"));
    assert!(source.contains("name: 'home'"));
}

#[test]
fn route_generation_works_from_pub_workspace_member() {
    let (_workspace, package_root) = make_pub_workspace_member();
    write_routing_workspace(&package_root, "dashboard");

    let result = run_build(BuildRequest {
        cwd: package_root.clone(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let source = fs::read_to_string(package_root.join("lib/route.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(source.contains("import 'package:product_showcase/pages/dashboard_page.dart';"));
    assert!(source.contains("final class DashboardRoute extends AppRoutePath"));
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

#[test]
fn build_reports_missing_initial_route_page() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");
    write_file(
        &workspace.path().join("lib/route.dart"),
        "import 'pages/dashboard_page.dart';\n\
         import 'pages/not_found_page.dart';\n\
         import 'route.g.dart';\n\
         \n\
         @Router(initial: MissingPage, notFound: NotFoundPage)\n\
         final class AppRouter extends $AppRouter {\n\
           const AppRouter();\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });

    assert!(result.has_errors());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("initial page `MissingPage` does not match")
    }));
}

#[test]
fn build_reports_missing_not_found_route_page() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");
    write_file(
        &workspace.path().join("lib/route.dart"),
        "import 'pages/dashboard_page.dart';\n\
         import 'route.g.dart';\n\
         \n\
         @Router(initial: DashboardPage, notFound: MissingPage)\n\
         final class AppRouter extends $AppRouter {\n\
           const AppRouter();\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });

    assert!(result.has_errors());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("notFound page `MissingPage` does not match")
    }));
}

#[test]
fn build_reports_route_shell_or_guard_without_visible_import() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");
    write_file(
        &workspace.path().join("lib/pages/project_page.dart"),
        "@Route('/projects/:projectId', name: 'project', shell: AppShell, guards: [ProjectGuard])\n\
         final class ProjectPage {\n\
           const ProjectPage({required this.projectId});\n\
           final int projectId;\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });

    assert!(result.has_errors());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("route shell `AppShell` on `ProjectPage`")
    }));
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("route guard `ProjectGuard` on `ProjectPage`")
    }));
}

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

fn write_routing_workspace(root: &std::path::Path, dashboard_name: &str) {
    write_file(
        &root.join("lib/route.dart"),
        "import 'pages/dashboard_page.dart';\n\
         import 'pages/not_found_page.dart';\n\
         import 'route.g.dart';\n\
         \n\
         @Router(initial: DashboardPage, notFound: NotFoundPage)\n\
         final class AppRouter extends $AppRouter {\n\
           const AppRouter();\n\
         }\n",
    );
    write_dashboard_page(root, dashboard_name);
    write_file(
        &root.join("lib/pages/not_found_page.dart"),
        "@Route('/404/:path', name: 'notFound', guards: [])\n\
         final class NotFoundPage {\n\
           const NotFoundPage({required this.path});\n\
           final String path;\n\
         }\n",
    );
}

fn write_dashboard_page(root: &std::path::Path, name: &str) {
    write_file(
        &root.join("lib/pages/dashboard_page.dart"),
        &format!(
            "@Route('/', name: '{name}')\n\
             final class DashboardPage {{\n\
               const DashboardPage();\n\
             }}\n"
        ),
    );
}
