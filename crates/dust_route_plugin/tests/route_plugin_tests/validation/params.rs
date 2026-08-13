use dust_ir::TypeIr;
use dust_plugin_api::DustPlugin;
use dust_route_plugin::register_plugin;

use crate::support::{
    constructor_param, defaulted_param, defaulted_param_source, enum_type, library_with_classes,
    library_with_classes_and_enums, route_page_class,
};

use super::support::diagnostic_messages;

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
fn accepts_typed_query_route_params() {
    let plugin = register_plugin();
    let mut classes = vec![route_page_class(
        "SearchPage",
        "('/search', name: 'search')",
        vec![
            constructor_param("tab", TypeIr::named("SearchTab")),
            constructor_param("from", TypeIr::named("DateTime").nullable()),
            constructor_param("redirect", TypeIr::named("Uri").nullable()),
            defaulted_param_source(
                "tags",
                TypeIr::list_of(TypeIr::string()),
                "const <String>[]",
            ),
            constructor_param("ids", TypeIr::list_of(TypeIr::int()).nullable()),
        ],
    )];

    let diagnostics = plugin.validate(&library_with_classes_and_enums(
        &mut classes,
        vec![enum_type("SearchTab", &["products", "orders"])],
    ));

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
            "route parameter `filters` on `ProjectPage` must be a URL type (`String`, `int`, `double`, `bool`, `DateTime`, `Uri`, enum, `List<String>`, or `List<int>`)"
        ]
    );
}

#[test]
fn rejects_unknown_named_query_param() {
    let plugin = register_plugin();
    let class = route_page_class(
        "ProjectPage",
        "('/projects', name: 'project')",
        vec![constructor_param("filter", TypeIr::named("ProjectFilter"))],
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![class]));

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "route parameter `filter` on `ProjectPage` must be a URL type (`String`, `int`, `double`, `bool`, `DateTime`, `Uri`, enum, `List<String>`, or `List<int>`)"
        ]
    );
}

#[test]
fn rejects_unsupported_repeated_query_param() {
    let plugin = register_plugin();
    let class = route_page_class(
        "ProjectPage",
        "('/projects', name: 'project')",
        vec![constructor_param(
            "prices",
            TypeIr::list_of(TypeIr::double()),
        )],
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![class]));

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "route repeated query parameter `prices` on `ProjectPage` must be `List<String>` or `List<int>`"
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
            "duplicate route path `/same` used by `FirstPage` (`same`) and `SecondPage` (`same`); URL `/same` matches both",
            "duplicate route name `same` used by `FirstPage` (`/same`) and `SecondPage` (`/same`)"
        ]
    );
}

#[test]
fn rejects_duplicate_route_path_with_page_names() {
    let plugin = register_plugin();
    let first = route_page_class("CartPage", "('/cart', name: 'cart')", Vec::new());
    let second = route_page_class("BasketPage", "('/cart', name: 'basket')", Vec::new());

    let diagnostics = plugin.validate(&library_with_classes(vec![first, second]));

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "duplicate route path `/cart` used by `CartPage` (`cart`) and `BasketPage` (`basket`); URL `/cart` matches both"
        ]
    );
}

#[test]
fn rejects_duplicate_route_name_with_page_paths() {
    let plugin = register_plugin();
    let first = route_page_class("CartPage", "('/cart', name: 'cart')", Vec::new());
    let second = route_page_class(
        "CartSummaryPage",
        "('/cart/summary', name: 'cart')",
        Vec::new(),
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![first, second]));

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "duplicate route name `cart` used by `CartPage` (`/cart`) and `CartSummaryPage` (`/cart/summary`)"
        ]
    );
}
