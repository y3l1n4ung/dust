use dust_ir::TypeIr;
use dust_plugin_api::DustPlugin;
use dust_route_plugin::register_plugin;

use super::support::{constructor_param, defaulted_param, library_with_classes, route_page_class};

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

fn diagnostic_messages(diagnostics: &[dust_diagnostics::Diagnostic]) -> Vec<&str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}
