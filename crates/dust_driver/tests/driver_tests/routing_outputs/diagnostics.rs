use dust_driver::{BuildRequest, run_build};

use super::helpers::write_routing_workspace;
use crate::support::{make_workspace, write_file};

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
