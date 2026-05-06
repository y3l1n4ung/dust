use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use super::support::{config, future_of, http_client_class, library_for, library_for_with_imports, method};

#[test]
fn emits_raw_response_wrappers_for_prefixed_response_types() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "fetchRaw",
            future_of(TypeIr::generic("dio.Response", vec![TypeIr::named("User")])),
            vec![config("GET", Some("('/users/raw')"))],
            Vec::new(),
        )],
    ));

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let emitted = contribution.support_types.join("\n");
    let helpers = contribution.shared_helpers.join("\n");

    assert!(emitted.contains("final _result = await _dio.fetch<Map<String, dynamic>>("));
    assert!(emitted.contains("return _dustBuildResponse<User>(_result, _value);"));
    assert!(
        helpers.contains("Response<T> _dustBuildResponse<T>(Response<dynamic> response, T data)")
    );
}

#[test]
fn omits_result_binding_for_void_methods() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "deleteUser",
            future_of(TypeIr::named("void")),
            vec![config("DELETE", Some("('/users/me')"))],
            Vec::new(),
        )],
    ));

    let emitted = plugin
        .emit(&library, &SymbolPlan::default())
        .support_types
        .join("\n");
    assert!(emitted.contains("await _dio.fetch<void>("));
    assert!(!emitted.contains("final _result = await _dio.fetch<void>("));
    assert!(emitted.contains("return;"));
}

#[test]
fn emits_direct_map_casts_for_map_payload_responses() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "createPost",
            future_of(TypeIr::generic(
                "Map",
                vec![TypeIr::string(), TypeIr::dynamic()],
            )),
            vec![config("POST", Some("('/posts')"))],
            Vec::new(),
        )],
    ));

    let emitted = plugin
        .emit(&library, &SymbolPlan::default())
        .support_types
        .join("\n");
    assert!(emitted.contains("return _result.data as Map<String, dynamic>;"));
    assert!(!emitted.contains("Map<String, dynamic>.fromJson"));
}

#[test]
fn emits_stream_fetches_for_response_body_payloads() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "streamPosts",
            future_of(TypeIr::named("ResponseBody")),
            vec![config("GET", Some("('/posts/raw')"))],
            Vec::new(),
        )],
    ));

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let emitted = contribution.support_types.join("\n");
    let helpers = contribution.shared_helpers.join("\n");

    assert!(emitted.contains("final _result = await _dio.fetch<ResponseBody>("));
    assert!(emitted.contains("_setStreamType<ResponseBody>("));
    assert!(emitted.contains("return _result.data!;"));
    assert!(helpers.contains("if (T == ResponseBody) {"));
    assert!(helpers.contains("requestOptions.responseType = ResponseType.stream;"));
}

#[test]
fn returns_raw_stream_response_without_rewrapping() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "streamEnvelope",
            future_of(TypeIr::generic(
                "Response",
                vec![TypeIr::named("ResponseBody")],
            )),
            vec![config("GET", Some("('/posts/raw-response')"))],
            Vec::new(),
        )],
    ));

    let emitted = plugin
        .emit(&library, &SymbolPlan::default())
        .support_types
        .join("\n");

    assert!(emitted.contains("final _result = await _dio.fetch<ResponseBody>("));
    assert!(emitted.contains("return _result;"));
    assert!(!emitted.contains("_dustBuildResponse<ResponseBody>"));
}

#[test]
fn emits_direct_byte_stream_methods() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "streamBytes",
            TypeIr::generic("Stream", vec![TypeIr::list_of(TypeIr::int())]),
            vec![config("GET", Some("('/posts/bytes')"))],
            Vec::new(),
        )],
    ));

    let emitted = plugin
        .emit(&library, &SymbolPlan::default())
        .support_types
        .join("\n");

    assert!(emitted.contains("Stream<List<int>> streamBytes() async* {"));
    assert!(emitted.contains("final _result = await _dio.fetch<ResponseBody>("));
    assert!(emitted.contains("_setStreamType<ResponseBody>("));
    assert!(emitted.contains("yield* _body.stream;"));
}

#[test]
fn emits_direct_text_stream_methods() {
    let plugin = register_plugin();
    let library = library_for_with_imports(
        http_client_class(
            vec![config("HttpClient", Some("()"))],
            vec![method(
                "streamText",
                TypeIr::generic("Stream", vec![TypeIr::string()]),
                vec![config("GET", Some("('/posts/text')"))],
                Vec::new(),
            )],
        ),
        vec!["dart:convert"],
    );

    let emitted = plugin
        .emit(&library, &SymbolPlan::default())
        .support_types
        .join("\n");

    assert!(emitted.contains("Stream<String> streamText() async* {"));
    assert!(emitted.contains("final _result = await _dio.fetch<ResponseBody>("));
    assert!(emitted.contains("yield* utf8.decoder.bind(_body.stream);"));
}
