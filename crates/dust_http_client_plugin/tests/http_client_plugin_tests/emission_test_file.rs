use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use super::support::{
    config, field, future_of, http_client_class, library_for, library_for_with_imports,
    library_with_classes, method, param, serde_model_class,
};

#[test]
fn generates_map_body_request_fixtures() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("(generateTest: true)"))],
        vec![method(
            "createUser",
            future_of(TypeIr::generic(
                "Map",
                vec![TypeIr::string(), TypeIr::dynamic()],
            )),
            vec![config("POST", Some("('/users')"))],
            vec![param(
                "payload",
                TypeIr::generic("Map", vec![TypeIr::string(), TypeIr::dynamic()]),
                vec![config("Body", Some("()"))],
            )],
        )],
    ));

    let generated = &plugin
        .emit(&library, &SymbolPlan::default())
        .auxiliary_outputs[0]
        .source;
    assert!(generated.contains("await api.createUser({'value': 'dust'});"));
    assert!(!generated.contains("skip: 'Dust could not synthesize fixtures"));
}

#[test]
fn generates_model_body_request_fixtures_from_local_serde_models() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        http_client_class(
            vec![config("HttpClient", Some("(generateTest: true)"))],
            vec![method(
                "createUser",
                future_of(TypeIr::named("User")),
                vec![config("POST", Some("('/users')"))],
                vec![param(
                    "payload",
                    TypeIr::named("UserCreate"),
                    vec![config("Body", Some("()"))],
                )],
            )],
        ),
        serde_model_class(
            "UserCreate",
            vec![
                field("name", TypeIr::string()),
                field("isAdmin", TypeIr::bool()),
            ],
        ),
    ]);

    let generated = &plugin
        .emit(&library, &SymbolPlan::default())
        .auxiliary_outputs[0]
        .source;
    assert!(
        generated.contains("await api.createUser(UserCreate.fromJson(<String, Object?>{'name': 'dust', 'isAdmin': true}));")
    );
    assert!(
        generated.contains(
            "expect(request.data, equals(UserCreate.fromJson(<String, Object?>{'name': 'dust', 'isAdmin': true}).toJson()));"
        )
    );
}

#[test]
fn omits_model_body_request_fixtures_without_local_model_ir() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("(generateTest: true)"))],
        vec![method(
            "createUser",
            future_of(TypeIr::named("User")),
            vec![config("POST", Some("('/users')"))],
            vec![param(
                "payload",
                TypeIr::named("UserCreate"),
                vec![config("Body", Some("()"))],
            )],
        )],
    ));

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    assert!(contribution.auxiliary_outputs.is_empty());
}

#[test]
fn generates_stream_response_body_fixtures() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("(generateTest: true)"))],
        vec![method(
            "streamUsers",
            future_of(TypeIr::named("ResponseBody")),
            vec![config("GET", Some("('/users/raw')"))],
            Vec::new(),
        )],
    ));

    let generated = &plugin
        .emit(&library, &SymbolPlan::default())
        .auxiliary_outputs[0]
        .source;
    assert!(generated.contains("ResponseBody.fromString('{}', 200)"));
}

#[test]
fn drains_generated_byte_stream_invocations() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("(generateTest: true)"))],
        vec![method(
            "streamUsers",
            TypeIr::generic("Stream", vec![TypeIr::list_of(TypeIr::int())]),
            vec![config("GET", Some("('/users/bytes')"))],
            Vec::new(),
        )],
    ));

    let generated = &plugin
        .emit(&library, &SymbolPlan::default())
        .auxiliary_outputs[0]
        .source;
    assert!(generated.contains("await api.streamUsers().drain<void>();"));
}

#[test]
fn drains_generated_text_stream_invocations() {
    let plugin = register_plugin();
    let library = library_for_with_imports(
        http_client_class(
            vec![config("HttpClient", Some("(generateTest: true)"))],
            vec![method(
                "streamUsers",
                TypeIr::generic("Stream", vec![TypeIr::string()]),
                vec![config("GET", Some("('/users/text')"))],
                Vec::new(),
            )],
        ),
        vec!["dart:convert"],
    );

    let generated = &plugin
        .emit(&library, &SymbolPlan::default())
        .auxiliary_outputs[0]
        .source;
    assert!(generated.contains("await api.streamUsers().drain<void>();"));
}
