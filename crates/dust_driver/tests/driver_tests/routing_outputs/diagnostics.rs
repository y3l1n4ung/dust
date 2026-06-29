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
         @AppRouter(initial: '/missing', notFound: '/404')\n\
         final class TestRouter extends $TestRouter {\n\
           const TestRouter();\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });

    assert!(result.has_errors());
    assert_eq!(
        diagnostic_messages(&result.diagnostics),
        vec![
            "router `TestRouter` initial path `/missing` does not match any discovered `@AppRoute` path"
        ]
    );
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
         @AppRouter(initial: '/', notFound: '/missing')\n\
         final class TestRouter extends $TestRouter {\n\
           const TestRouter();\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });

    assert!(result.has_errors());
    assert_eq!(
        diagnostic_messages(&result.diagnostics),
        vec![
            "router `TestRouter` notFound path `/missing` does not match any discovered `@AppRoute` path"
        ]
    );
}

#[test]
fn build_reports_route_shell_or_guard_without_visible_import() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");
    write_file(
        &workspace.path().join("lib/pages/project_page.dart"),
        "@AppRoute('/projects/:projectId', name: 'project', shell: AppShell, guards: [ProjectGuard])\n\
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
    assert_eq!(
        diagnostic_messages(&result.diagnostics),
        vec![
            "route shell `AppShell` on `ProjectPage` must be declared in the same library or imported",
            "route guard `ProjectGuard` on `ProjectPage` must be declared in the same library or imported",
        ]
    );
}

fn diagnostic_messages(diagnostics: &[dust_diagnostics::Diagnostic]) -> Vec<&str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}
