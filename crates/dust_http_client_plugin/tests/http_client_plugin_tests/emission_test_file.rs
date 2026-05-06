use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use super::support::{
    config, future_of, http_client_class, library_for, library_for_with_imports, method, param,
};

#[test]
fn generates_map_body_request_fixtures() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![
            config("HttpClient", Some("()")),
            config("GenerateTest", Some("()")),
        ],
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
    assert!(generated.contains("await api.createUser(const <String, dynamic>{'value': 'dust'});"));
    assert!(!generated.contains("skip: 'Dust could not synthesize fixtures"));
}

#[test]
fn skips_model_body_request_fixtures_that_need_user_data() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![
            config("HttpClient", Some("()")),
            config("GenerateTest", Some("()")),
        ],
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

    let generated = &plugin
        .emit(&library, &SymbolPlan::default())
        .auxiliary_outputs[0]
        .source;
    assert!(
        generated
            .contains("test('POST createUser', () {}, skip: 'Dust could not synthesize fixtures")
    );
}

#[test]
fn generates_stream_response_body_fixtures() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![
            config("HttpClient", Some("()")),
            config("GenerateTest", Some("()")),
        ],
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
        vec![
            config("HttpClient", Some("()")),
            config("GenerateTest", Some("()")),
        ],
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
            vec![
                config("HttpClient", Some("()")),
                config("GenerateTest", Some("()")),
            ],
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
