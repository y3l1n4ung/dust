use crate::support::write_file;

pub(crate) fn write_routing_workspace(root: &std::path::Path, dashboard_name: &str) {
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

pub(crate) fn write_dashboard_page(root: &std::path::Path, name: &str) {
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
