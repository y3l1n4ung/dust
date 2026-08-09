use dust_ir::{DartFileIr, ImportIr, TypeIr};
use dust_plugin_api::DustPlugin;
use dust_route_plugin::register_plugin;

use super::support::{
    constructor_param, defaulted_param, library_with_classes, positional_param, route_page_class,
    shell_class, span,
};

#[test]
fn accepts_url_primitive_route_params() {
    let plugin = register_plugin();
    let class = route_page_class(
        "ProjectPage",
        "('/projects/:projectId', name: 'project')",
        vec![
            constructor_param("projectId", TypeIr::int()),
            constructor_param("tab", TypeIr::string().nullable()),
            constructor_param("download", TypeIr::bool().nullable()),
        ],
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![class]));

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn accepts_query_param_defaults_when_default_source_is_preserved() {
    let plugin = register_plugin();
    let class = route_page_class(
        "SearchPage",
        "('/search', name: 'search')",
        vec![defaulted_param("page", TypeIr::int())],
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![class]));

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn accepts_local_route_shell_with_named_widget_child() {
    let plugin = register_plugin();
    let shell = shell_class(
        "AppShell",
        vec![constructor_param("child", TypeIr::named("Widget"))],
    );
    let page = route_page_class(
        "DashboardPage",
        "('/dashboard', name: 'dashboard', shell: AppShell)",
        Vec::new(),
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![shell, page]));

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn accepts_route_shell_from_show_import() {
    let plugin = register_plugin();
    let page = route_page_class(
        "DashboardPage",
        "('/dashboard', name: 'dashboard', shell: AppShell)",
        Vec::new(),
    );
    let mut library = library_with_classes(vec![page]);
    add_import(
        &mut library,
        "package:app/app_shell.dart",
        &["AppShell"],
        &[],
    );

    let diagnostics = plugin.validate(&library);

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn rejects_route_shell_hidden_by_import() {
    let plugin = register_plugin();
    let page = route_page_class(
        "DashboardPage",
        "('/dashboard', name: 'dashboard', shell: AppShell)",
        Vec::new(),
    );
    let mut library = library_with_classes(vec![page]);
    add_import(
        &mut library,
        "package:app/app_shell.dart",
        &[],
        &["AppShell"],
    );

    let diagnostics = plugin.validate(&library);

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "route shell `AppShell` on `DashboardPage` must be declared in the same library or imported"
        ]
    );
}

#[test]
fn rejects_local_route_shell_without_named_widget_child() {
    let plugin = register_plugin();
    let shell = shell_class("AppShell", Vec::new());
    let page = route_page_class(
        "DashboardPage",
        "('/dashboard', name: 'dashboard', shell: AppShell)",
        Vec::new(),
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![shell, page]));

    let messages = diagnostic_messages(&diagnostics);
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("route shell `AppShell` on `DashboardPage`"));
    assert!(messages[0].contains("required named `Widget child` parameter"));
}

#[test]
fn rejects_local_route_shell_with_positional_child() {
    let plugin = register_plugin();
    let shell = shell_class(
        "AppShell",
        vec![positional_param("child", TypeIr::named("Widget"))],
    );
    let page = route_page_class(
        "DashboardPage",
        "('/dashboard', name: 'dashboard', shell: AppShell)",
        Vec::new(),
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![shell, page]));

    let messages = diagnostic_messages(&diagnostics);
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("route shell `AppShell` on `DashboardPage`"));
    assert!(messages[0].contains("required named `Widget child` parameter"));
}

#[test]
fn rejects_local_route_shell_with_nullable_child() {
    let plugin = register_plugin();
    let shell = shell_class(
        "AppShell",
        vec![constructor_param(
            "child",
            TypeIr::named("Widget").nullable(),
        )],
    );
    let page = route_page_class(
        "DashboardPage",
        "('/dashboard', name: 'dashboard', shell: AppShell)",
        Vec::new(),
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![shell, page]));

    let messages = diagnostic_messages(&diagnostics);
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("route shell `AppShell` on `DashboardPage`"));
    assert!(messages[0].contains("required named `Widget child` parameter"));
}

#[test]
fn rejects_local_route_shell_with_defaulted_child() {
    let plugin = register_plugin();
    let shell = shell_class(
        "AppShell",
        vec![defaulted_param("child", TypeIr::named("Widget"))],
    );
    let page = route_page_class(
        "DashboardPage",
        "('/dashboard', name: 'dashboard', shell: AppShell)",
        Vec::new(),
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![shell, page]));

    let messages = diagnostic_messages(&diagnostics);
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("route shell `AppShell` on `DashboardPage`"));
    assert!(messages[0].contains("required named `Widget child` parameter"));
}

#[test]
fn rejects_query_param_defaults_when_default_source_is_missing() {
    let plugin = register_plugin();
    let mut page = defaulted_param("page", TypeIr::int());
    page.default_value_source = None;
    let class = route_page_class("SearchPage", "('/search', name: 'search')", vec![page]);

    let diagnostics = plugin.validate(&library_with_classes(vec![class]));

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "route query parameter `page` on `SearchPage` has a constructor default that Dust could not preserve"
        ]
    );
}

#[test]
fn rejects_relative_route_path() {
    let plugin = register_plugin();
    let class = route_page_class("LoginPage", "('login', name: 'login')", Vec::new());

    let diagnostics = plugin.validate(&library_with_classes(vec![class]));

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec!["route `LoginPage` path `login` must be absolute"]
    );
}

#[test]
fn rejects_missing_path_constructor_param() {
    let plugin = register_plugin();
    let class = route_page_class(
        "ProjectPage",
        "('/projects/:projectId', name: 'project')",
        Vec::new(),
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![class]));

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "route path parameter `:projectId` on `ProjectPage` has no matching constructor parameter"
        ]
    );
}

#[test]
fn rejects_nullable_path_param() {
    let plugin = register_plugin();
    let class = route_page_class(
        "ProjectPage",
        "('/projects/:projectId', name: 'project')",
        vec![constructor_param("projectId", TypeIr::int().nullable())],
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![class]));

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec!["route path parameter `projectId` on `ProjectPage` must be required and non-nullable"]
    );
}

#[test]
fn rejects_complex_query_param() {
    let plugin = register_plugin();
    let class = route_page_class(
        "ProjectPage",
        "('/projects/:projectId', name: 'project')",
        vec![
            constructor_param("projectId", TypeIr::int()),
            constructor_param(
                "filters",
                TypeIr::map_of(TypeIr::string(), TypeIr::string()),
            ),
        ],
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![class]));

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "route parameter `filters` on `ProjectPage` must be a URL primitive (`String`, `int`, `double`, or `bool`)",
            "route query parameter `filters` on `ProjectPage` must be nullable or have a default value"
        ]
    );
}

#[test]
fn rejects_duplicate_paths_and_names() {
    let plugin = register_plugin();
    let first = route_page_class("FirstPage", "('/same', name: 'same')", Vec::new());
    let second = route_page_class("SecondPage", "('/same', name: 'same')", Vec::new());

    let diagnostics = plugin.validate(&library_with_classes(vec![first, second]));

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "duplicate route path `/same`",
            "duplicate route name `same`"
        ]
    );
}

#[test]
fn accepts_visible_route_guard_from_show_import() {
    let plugin = register_plugin();
    let class = route_page_class(
        "AdminPage",
        "('/admin', name: 'admin', guards: [AuthGuard])",
        Vec::new(),
    );
    let mut library = library_with_classes(vec![class]);
    add_import(&mut library, "package:app/guards.dart", &["AuthGuard"], &[]);

    let diagnostics = plugin.validate(&library);

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn rejects_route_guard_missing_from_show_import() {
    let plugin = register_plugin();
    let class = route_page_class(
        "AdminPage",
        "('/admin', name: 'admin', guards: [AuthGuard])",
        Vec::new(),
    );
    let mut library = library_with_classes(vec![class]);
    add_import(
        &mut library,
        "package:app/guards.dart",
        &["BillingGuard"],
        &[],
    );

    let diagnostics = plugin.validate(&library);

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "route guard `AuthGuard` on `AdminPage` must be declared in the same library or imported"
        ]
    );
}

#[test]
fn rejects_route_guard_hidden_by_import() {
    let plugin = register_plugin();
    let class = route_page_class(
        "AdminPage",
        "('/admin', name: 'admin', guards: [AuthGuard])",
        Vec::new(),
    );
    let mut library = library_with_classes(vec![class]);
    add_import(&mut library, "package:app/guards.dart", &[], &["AuthGuard"]);

    let diagnostics = plugin.validate(&library);

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "route guard `AuthGuard` on `AdminPage` must be declared in the same library or imported"
        ]
    );
}

fn diagnostic_messages(diagnostics: &[dust_diagnostics::Diagnostic]) -> Vec<&str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}

fn add_import(library: &mut DartFileIr, uri: &str, show: &[&str], hide: &[&str]) {
    library.imports.push(uri.to_owned());
    library.import_directives.push(ImportIr {
        uri: uri.to_owned(),
        prefix: None,
        show: show.iter().map(|name| (*name).to_owned()).collect(),
        hide: hide.iter().map(|name| (*name).to_owned()).collect(),
        is_deferred: false,
        span: span(0, 0),
    });
}
