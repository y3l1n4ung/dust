use dust_ir::TypeIr;

use crate::support::{
    constructor_param, guard_class, library_with_classes, named_constructor_guard_class,
    route_page_class, router_class,
};

use super::support::{diagnostic_messages, generate_route_output, route_outputs_snapshot};

#[test]
fn rejects_generated_route_class_name_collisions() {
    let library = library_with_classes(vec![
        router_class("(initial: '/orders/detail', notFound: '/404')"),
        route_page_class(
            "OrderDetailPage",
            "('/orders/detail', name: 'orderDetail')",
            Vec::new(),
        ),
        route_page_class(
            "OrderDetailSlugPage",
            "('/order-details/:id', name: 'order_detail')",
            vec![constructor_param("id", TypeIr::string())],
        ),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec!["generated route class `OrderDetailRoute` is emitted by more than one route name"]
    );
}

#[test]
fn rejects_generated_route_class_that_conflicts_with_existing_class() {
    let library = library_with_classes(vec![
        router_class("(initial: '/login', notFound: '/404')"),
        guard_class("LoginRoute", Vec::new()),
        route_page_class("LoginPage", "('/login', name: 'login')", Vec::new()),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "generated route class `LoginRoute` conflicts with an existing Dart class; rename the route or page"
        ]
    );
}

#[test]
fn rejects_generated_route_class_that_conflicts_with_router_support_class() {
    let library = library_with_classes(vec![
        router_class("(initial: '/test', notFound: '/404')"),
        route_page_class("TestPage", "('/test', name: 'test')", Vec::new()),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "generated route class `TestRoute` conflicts with a generated router support declaration"
        ]
    );
}

#[test]
fn rejects_reserved_route_helper_names() {
    let library = library_with_classes(vec![
        router_class("(initial: '/switch', notFound: '/404')"),
        route_page_class("SwitchPage", "('/switch', name: 'switch')", Vec::new()),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec!["route name `switch` must be a valid non-reserved Dart identifier"]
    );
}

#[test]
fn rejects_invalid_route_helper_identifiers() {
    let library = library_with_classes(vec![
        router_class("(initial: '/orders/detail', notFound: '/404')"),
        route_page_class(
            "OrderDetailPage",
            "('/orders/detail', name: 'order-detail')",
            Vec::new(),
        ),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec!["route name `order-detail` must be a valid non-reserved Dart identifier"]
    );
}

#[test]
fn rejects_route_helper_name_that_conflicts_with_navigator_pop() {
    let library = library_with_classes(vec![
        router_class("(initial: '/pop', notFound: '/404')"),
        route_page_class("PopPage", "('/pop', name: 'pop')", Vec::new()),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec!["route name `pop` conflicts with the generated navigator `pop` helper"]
    );
}

#[test]
fn rejects_duplicate_route_paths_with_page_names_before_emitting_parser() {
    let library = library_with_classes(vec![
        router_class("(initial: '/cart', notFound: '/404')"),
        route_page_class("CartPage", "('/cart', name: 'cart')", Vec::new()),
        route_page_class("BasketPage", "('/cart', name: 'basket')", Vec::new()),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "duplicate route path `/cart` used by `BasketPage` (`basket`) and `CartPage` (`cart`); URL `/cart` matches both"
        ]
    );
}

#[test]
fn rejects_duplicate_route_names_with_page_paths_before_emitting_parser() {
    let library = library_with_classes(vec![
        router_class("(initial: '/cart', notFound: '/404')"),
        route_page_class("CartPage", "('/cart', name: 'cart')", Vec::new()),
        route_page_class(
            "CartSummaryPage",
            "('/cart/summary', name: 'cart')",
            Vec::new(),
        ),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "duplicate route name `cart` used by `CartPage` (`/cart`) and `CartSummaryPage` (`/cart/summary`)",
            "generated route helper `cart` is emitted more than once",
            "generated route class `CartRoute` is emitted by more than one route name"
        ]
    );
}

#[test]
fn rejects_guard_without_unnamed_constructor() {
    let library = library_with_classes(vec![
        router_class("(initial: '/', notFound: '/404')"),
        route_page_class(
            "DashboardPage",
            "('/', name: 'dashboard', guards: [AuthGuard])",
            Vec::new(),
        ),
        named_constructor_guard_class("AuthGuard"),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "route guard `AuthGuard` needs an unnamed generative constructor for generated route guard lookup"
        ]
    );
}

#[test]
fn rejects_guard_required_dependency_with_unresolvable_type() {
    let library = library_with_classes(vec![
        router_class("(initial: '/', notFound: '/404')"),
        route_page_class(
            "DashboardPage",
            "('/', name: 'dashboard', guards: [AuthGuard])",
            Vec::new(),
        ),
        guard_class(
            "AuthGuard",
            vec![constructor_param(
                "predicate",
                TypeIr::function("bool Function(String value)"),
            )],
        ),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "route guard `AuthGuard` constructor parameter `predicate` needs a resolvable type for router injection"
        ]
    );
}

#[test]
fn rejects_duplicate_path_params_before_emitting_parser() {
    let library = library_with_classes(vec![
        router_class("(initial: '/users/:id/posts/:id', notFound: '/404')"),
        route_page_class(
            "PostPage",
            "('/users/:id/posts/:id', name: 'post')",
            vec![constructor_param("id", TypeIr::int())],
        ),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "route `PostPage` path `/users/:id/posts/:id` declares duplicate path parameter `:id`"
        ]
    );
}

#[test]
fn rejects_static_and_dynamic_route_siblings_before_emitting_parser() {
    let library = library_with_classes(vec![
        router_class("(initial: '/users/settings', notFound: '/404')"),
        route_page_class(
            "UserPage",
            "('/users/:id', name: 'user')",
            vec![constructor_param("id", TypeIr::int())],
        ),
        route_page_class(
            "UserSettingsPage",
            "('/users/settings', name: 'userSettings')",
            Vec::new(),
        ),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "route `UserSettingsPage` (`userSettings`, `/users/settings`) conflicts with `UserPage` (`user`, `/users/:id`); URL `/users/settings` can match both because static and dynamic segments under `/users` are ambiguous"
        ]
    );
}

#[test]
fn rejects_deeper_static_and_dynamic_route_siblings_with_example_url() {
    let library = library_with_classes(vec![
        router_class("(initial: '/shops/acme/products/featured', notFound: '/404')"),
        route_page_class(
            "ShopProductPage",
            "('/shops/:shopId/products/featured', name: 'shopProduct')",
            vec![constructor_param("shopId", TypeIr::string())],
        ),
        route_page_class(
            "FeaturedProductPage",
            "('/shops/acme/products/:slug', name: 'featuredProduct')",
            vec![constructor_param("slug", TypeIr::string())],
        ),
    ]);

    let contribution = generate_route_output(&library);

    assert!(contribution.primary_source.is_none());
    assert_eq!(
        diagnostic_messages(&contribution.diagnostics),
        vec![
            "route `FeaturedProductPage` (`featuredProduct`, `/shops/acme/products/:slug`) conflicts with `ShopProductPage` (`shopProduct`, `/shops/:shopId/products/featured`); URL `/shops/acme/products/featured` can match both because static and dynamic segments under `/shops` are ambiguous"
        ]
    );
}

#[test]
fn allows_deeper_static_route_beside_shorter_dynamic_route() {
    let library = library_with_classes(vec![
        router_class("(initial: '/users/settings/profile', notFound: '/404')"),
        route_page_class(
            "UserPage",
            "('/users/:id', name: 'user')",
            vec![constructor_param("id", TypeIr::int())],
        ),
        route_page_class(
            "UserSettingsProfilePage",
            "('/users/settings/profile', name: 'userSettingsProfile')",
            Vec::new(),
        ),
    ]);

    let contribution = generate_route_output(&library);

    assert!(
        contribution.diagnostics.is_empty(),
        "{:?}",
        contribution.diagnostics
    );
    assert!(!route_outputs_snapshot(&contribution).is_empty());
}
