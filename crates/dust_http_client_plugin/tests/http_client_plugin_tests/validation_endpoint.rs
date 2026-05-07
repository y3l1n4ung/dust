use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::DustPlugin;

use super::support::{
    config, future_of, http_client_class, library_for, library_for_with_imports, method,
};

#[test]
fn rejects_methods_without_http_verbs() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "getUser",
            future_of(TypeIr::named("User")),
            Vec::new(),
            Vec::new(),
        )],
    )));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("must declare exactly one HTTP verb annotation")
    }));
}

#[test]
fn rejects_methods_with_multiple_http_verbs() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "saveUser",
            future_of(TypeIr::named("User")),
            vec![
                config("GET", Some("('/users/{id}')")),
                config("POST", Some("('/users')")),
            ],
            Vec::new(),
        )],
    )));

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("more than one HTTP verb"))
    );
}

#[test]
fn rejects_unsupported_synchronous_return_types() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "getUser",
            TypeIr::named("User"),
            vec![config("GET", Some("('/users/{id}')"))],
            Vec::new(),
        )],
    )));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("must return `Future<T>`, `Future<Response<T>>`, or a supported stream shape")
    }));
}

#[test]
fn accepts_prefixed_response_return_types() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "fetchRaw",
            future_of(TypeIr::generic("dio.Response", vec![TypeIr::named("User")])),
            vec![config("GET", Some("('/users/raw')"))],
            Vec::new(),
        )],
    )));

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn accepts_raw_stream_body_return_types() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![
            method(
                "streamBody",
                future_of(TypeIr::named("ResponseBody")),
                vec![config("GET", Some("('/events/raw')"))],
                Vec::new(),
            ),
            method(
                "streamEnvelope",
                future_of(TypeIr::generic(
                    "Response",
                    vec![TypeIr::named("ResponseBody")],
                )),
                vec![config("GET", Some("('/events/raw-response')"))],
                Vec::new(),
            ),
        ],
    )));

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn rejects_non_string_keyed_map_return_types() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "loadStats",
            future_of(TypeIr::generic(
                "Map",
                vec![TypeIr::int(), TypeIr::dynamic()],
            )),
            vec![config("GET", Some("('/stats')"))],
            Vec::new(),
        )],
    )));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("must return `Future<T>`, `Future<Response<T>>`, or a supported stream shape")
    }));
}

#[test]
fn rejects_stream_return_types() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![
            method(
                "watchUsers",
                future_of(TypeIr::generic("Stream", vec![TypeIr::named("User")])),
                vec![config("GET", Some("('/users/stream')"))],
                Vec::new(),
            ),
            method(
                "watchUsersRaw",
                future_of(TypeIr::generic(
                    "Response",
                    vec![TypeIr::generic("Stream", vec![TypeIr::named("User")])],
                )),
                vec![config("GET", Some("('/users/raw-stream')"))],
                Vec::new(),
            ),
        ],
    )));

    assert_eq!(diagnostics.len(), 2, "{diagnostics:?}");
    assert!(diagnostics.iter().all(|diagnostic| {
        diagnostic
            .message
            .contains("must return `Future<T>`, `Future<Response<T>>`, or a supported stream shape")
    }));
}

#[test]
fn accepts_direct_byte_stream_return_types() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "streamBytes",
            TypeIr::generic("Stream", vec![TypeIr::list_of(TypeIr::int())]),
            vec![config("GET", Some("('/events/bytes')"))],
            Vec::new(),
        )],
    )));

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn accepts_direct_text_stream_return_types_when_dart_convert_is_imported() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for_with_imports(
        http_client_class(
            vec![config("HttpClient", Some("()"))],
            vec![method(
                "streamText",
                TypeIr::generic("Stream", vec![TypeIr::string()]),
                vec![config("GET", Some("('/events/text')"))],
                Vec::new(),
            )],
        ),
        vec!["dart:convert"],
    ));

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn rejects_text_stream_return_types_without_dart_convert_import() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "streamText",
            TypeIr::generic("Stream", vec![TypeIr::string()]),
            vec![config("GET", Some("('/events/text')"))],
            Vec::new(),
        )],
    )));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("requires `import 'dart:convert';`")
    }));
}
