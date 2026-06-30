use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use super::support::{
    config, future_of, http_client_class, library_for, method, named_param, param,
    required_named_param,
};

#[test]
fn emits_form_urlencoded_request_bodies() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "rename",
            future_of(TypeIr::named("User")),
            vec![
                config("FormUrlEncoded", Some("()")),
                config("PATCH", Some("('/users/{id}')")),
            ],
            vec![
                param("id", TypeIr::string(), vec![config("Path", Some("('id')"))]),
                param(
                    "title",
                    TypeIr::string(),
                    vec![config("Field", Some("('title')"))],
                ),
            ],
        )],
    ));

    let emitted = plugin
        .emit(&library, &SymbolPlan::default())
        .support_types
        .join("\n");
    assert!(emitted.contains("final _data = <String, dynamic>{"));
    assert!(emitted.contains("'title': title"));
    assert!(emitted.contains("contentType: 'application/x-www-form-urlencoded'"));
}

#[test]
fn emits_multipart_requests_with_prefixed_options_types() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "upload",
            future_of(TypeIr::named("void")),
            vec![
                config("MultiPart", Some("()")),
                config("POST", Some("('/files/{id}')")),
            ],
            vec![
                param("id", TypeIr::string(), vec![config("Path", Some("('id')"))]),
                named_param(
                    "file",
                    TypeIr::named("MultipartFile").nullable(),
                    vec![config("Part", Some("('file')"))],
                ),
                named_param(
                    "options",
                    TypeIr::named("dio.Options").nullable(),
                    Vec::new(),
                ),
            ],
        )],
    ));

    let emitted = plugin
        .emit(&library, &SymbolPlan::default())
        .support_types
        .join("\n");
    assert!(emitted.contains("final _data = FormData.fromMap(<String, dynamic>{"));
    assert!(emitted.contains("if (file != null) 'file': file"));
    assert!(emitted.contains(
        "options?.copyWith(
          method: 'POST',"
    ));
    assert!(emitted.contains("contentType: 'multipart/form-data'"));
}

#[test]
fn emits_model_request_bodies_null_safely() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![
            method(
                "createUser",
                future_of(TypeIr::named("User")),
                vec![config("POST", Some("('/users')"))],
                vec![param(
                    "payload",
                    TypeIr::named("UserCreate"),
                    vec![config("Body", Some("()"))],
                )],
            ),
            method(
                "updateUser",
                future_of(TypeIr::named("User")),
                vec![config("PATCH", Some("('/users')"))],
                vec![param(
                    "payload",
                    TypeIr::named("UserUpdate").nullable(),
                    vec![config("Body", Some("()"))],
                )],
            ),
            method(
                "putMetadata",
                future_of(TypeIr::named("User")),
                vec![config("PUT", Some("('/users/meta')"))],
                vec![param(
                    "payload",
                    TypeIr::generic("Map", vec![TypeIr::string(), TypeIr::dynamic()]).nullable(),
                    vec![config("Body", Some("()"))],
                )],
            ),
        ],
    ));

    let emitted = plugin
        .emit(&library, &SymbolPlan::default())
        .support_types
        .join("\n");
    assert!(emitted.contains("final Object? _data = payload.toJson();"));
    assert!(emitted.contains("final Object? _data = payload?.toJson();"));
    assert!(emitted.contains("final Object? _data = payload;"));
}

#[test]
fn emits_documented_request_encoding_policy() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config(
            "HttpClient",
            Some("(headers: {'accept': 'application/json'})"),
        )],
        vec![method(
            "search",
            future_of(TypeIr::named("void")),
            vec![
                config("Headers", Some("({'x-method': 'search'})")),
                config("GET", Some("('/users/{slug}')")),
            ],
            vec![
                param(
                    "slug",
                    TypeIr::string(),
                    vec![config("Path", Some("('slug')"))],
                ),
                named_param(
                    "tags",
                    TypeIr::list_of(TypeIr::string()).nullable(),
                    vec![config("Query", Some("('tags')"))],
                ),
                required_named_param(
                    "filters",
                    TypeIr::map_of(TypeIr::string(), TypeIr::dynamic()),
                    vec![config("Queries", Some("()"))],
                ),
                named_param(
                    "page",
                    TypeIr::int(),
                    vec![config("Header", Some("('x-page')"))],
                ),
                required_named_param(
                    "headers",
                    TypeIr::map_of(TypeIr::string(), TypeIr::string()),
                    vec![config("HeaderMap", Some("()"))],
                ),
            ],
        )],
    ));

    let emitted = plugin
        .emit(&library, &SymbolPlan::default())
        .support_types
        .join("\n");
    assert!(emitted.contains("'/users/' + Uri.encodeComponent(slug.toString())"));
    assert!(emitted.contains("if (tags != null) _queryParameters['tags'] = tags;"));
    assert!(emitted.contains("_queryParameters.addAll(filters);"));
    assert!(emitted.contains("_headers['accept'] = 'application/json';"));
    assert!(emitted.contains("_headers['x-method'] = 'search';"));
    assert!(emitted.contains("_headers['x-page'] = page.toString();"));
    assert!(emitted.contains("_headers.addAll(headers);"));
    let method_header = emitted
        .find("_headers['x-method'] = 'search';")
        .expect("method header assignment is emitted");
    let header_map = emitted
        .find("_headers.addAll(headers);")
        .expect("header map merge is emitted");
    assert!(method_header < header_map);
}
