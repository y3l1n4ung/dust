use dust_ir::TypeIr;

use crate::support::{
    constructor_param, defaulted_param_source, enum_type, library_with_classes_and_enums,
    route_page_class, router_class,
};

use super::support::{assert_route_snapshot, generate_route_output, route_outputs_snapshot};

#[test]
fn emits_query_decode_and_not_found_fallback_matrix() {
    let mut classes = vec![
        router_class("(initial: '/filters', notFound: '/missing/:code/:ratio/:active')"),
        route_page_class(
            "FilterPage",
            "('/filters', name: 'filters')",
            vec![
                constructor_param("requiredText", TypeIr::string()),
                defaulted_param_source("defaultText", TypeIr::string(), "'all'"),
                constructor_param("optionalPage", TypeIr::int().nullable()),
                defaulted_param_source("requiredLimit", TypeIr::int(), "25"),
                defaulted_param_source("requiredRatio", TypeIr::double(), "1.5"),
                defaulted_param_source("requiredFlag", TypeIr::bool(), "true"),
                defaulted_param_source(
                    "requiredDate",
                    TypeIr::named("DateTime"),
                    "DateTime.utc(2026)",
                ),
                defaulted_param_source("requiredUri", TypeIr::named("Uri"), "Uri.parse('/')"),
                constructor_param("optionalTags", TypeIr::list_of(TypeIr::string()).nullable()),
                constructor_param("requiredIds", TypeIr::list_of(TypeIr::int())),
                constructor_param("optionalTab", TypeIr::named("RouteTab").nullable()),
                defaulted_param_source(
                    "requiredTab",
                    TypeIr::named("RouteTab"),
                    "RouteTab.overview",
                ),
            ],
        ),
        route_page_class(
            "NotFoundPage",
            "('/missing/:code/:ratio/:active', name: 'notFound', guards: [])",
            vec![
                constructor_param("code", TypeIr::int()),
                constructor_param("ratio", TypeIr::double()),
                constructor_param("active", TypeIr::bool()),
                constructor_param("message", TypeIr::string()),
                constructor_param("from", TypeIr::named("Uri").nullable()),
                defaulted_param_source("source", TypeIr::string(), "'generated'"),
            ],
        ),
    ];
    let library = library_with_classes_and_enums(
        &mut classes,
        vec![enum_type("RouteTab", &["overview", "reviews"])],
    );

    let contribution = generate_route_output(&library);
    assert_eq!(contribution.diagnostics, []);
    let output = route_outputs_snapshot(&contribution);

    assert_route_snapshot("parser_decode_matrix.dart.snapshot", &output);
}
