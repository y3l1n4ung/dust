use dust_ir::TypeIr;

use crate::support::{
    constructor_param, defaulted_param_source, enum_type, library_with_classes_and_enums,
    route_page_class, router_class,
};

use super::support::{assert_route_snapshot, generate_route_output, route_outputs_snapshot};

#[test]
fn emits_url_parser_edge_case_matrix() {
    let mut classes = vec![
        router_class("(initial: '/', notFound: '/404')"),
        route_page_class("HomePage", "('/', name: 'home')", Vec::new()),
        route_page_class(
            "ProductDetailPage",
            "('/products/:productId', name: 'productDetail')",
            vec![
                constructor_param("productId", TypeIr::int()),
                constructor_param("ref", TypeIr::string().nullable()),
                defaulted_param_source("preview", TypeIr::bool(), "false"),
            ],
        ),
        route_page_class(
            "SearchPage",
            "('/search', name: 'search')",
            vec![
                constructor_param("query", TypeIr::string().nullable()),
                defaulted_param_source("page", TypeIr::int(), "1"),
                constructor_param("minRating", TypeIr::double().nullable()),
                constructor_param("inStock", TypeIr::bool().nullable()),
            ],
        ),
        route_page_class(
            "FilterPage",
            "('/filters', name: 'filters', guards: [])",
            vec![
                constructor_param("tab", TypeIr::named("RouteTab")),
                constructor_param("from", TypeIr::named("DateTime")),
                constructor_param("redirect", TypeIr::named("Uri").nullable()),
                defaulted_param_source(
                    "tags",
                    TypeIr::list_of(TypeIr::string()),
                    "const <String>[]",
                ),
                constructor_param("ids", TypeIr::list_of(TypeIr::int()).nullable()),
            ],
        ),
        route_page_class(
            "InvitePreviewPage",
            "('/invite/:code/:active', name: 'invitePreview')",
            vec![
                constructor_param("code", TypeIr::string()),
                constructor_param("active", TypeIr::bool()),
                defaulted_param_source("score", TypeIr::double(), "0.0"),
            ],
        ),
    ];
    let library = library_with_classes_and_enums(
        &mut classes,
        vec![enum_type("RouteTab", &["overview", "reviews"])],
    );

    let contribution = generate_route_output(&library);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("url_parser_edge_case_matrix.dart.snapshot", &output);
}
