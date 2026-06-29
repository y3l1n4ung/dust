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

#[test]
fn build_reports_duplicate_route_path_params() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");
    write_file(
        &workspace.path().join("lib/pages/post_page.dart"),
        "@AppRoute('/users/:id/posts/:id', name: 'post')\n\
         final class PostPage {\n\
           const PostPage({required this.id});\n\
           final int id;\n\
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
            "route `PostPage` path `/users/:id/posts/:id` declares duplicate path parameter `:id`"
        ]
    );
}

#[test]
fn build_reports_static_and_dynamic_route_siblings() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");
    write_file(
        &workspace.path().join("lib/pages/user_page.dart"),
        "@AppRoute('/users/:id', name: 'user')\n\
         final class UserPage {\n\
           const UserPage({required this.id});\n\
           final int id;\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/pages/user_settings_page.dart"),
        "@AppRoute('/users/settings', name: 'userSettings')\n\
         final class UserSettingsPage {\n\
           const UserSettingsPage();\n\
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
            "route path `/users/settings` conflicts with sibling `/users/:id`; static and dynamic segments under `/users` are ambiguous"
        ]
    );
}

fn diagnostic_messages(diagnostics: &[dust_diagnostics::Diagnostic]) -> Vec<&str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}
