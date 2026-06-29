use dust_driver::{BuildRequest, run_build};

use super::helpers::write_routing_workspace;
use crate::support::{make_workspace, write_file};

#[test]
fn multiple_router_roots_are_rejected() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");
    write_file(
        &workspace.path().join("lib/admin_route.dart"),
        "import 'pages/admin_page.dart';\n\
         import 'admin_route.g.dart';\n\
         \n\
         @AppRouter(initial: '/admin', notFound: '/404')\n\
         final class AdminRouter extends $AdminRouter {\n\
           const AdminRouter();\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/pages/admin_page.dart"),
        "@AppRoute('/admin', name: 'admin')\n\
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

    assert!(result.has_errors());
    assert_eq!(
        result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>(),
        vec!["exactly one `@AppRouter` is allowed in a Dust route workspace"]
    );
}
