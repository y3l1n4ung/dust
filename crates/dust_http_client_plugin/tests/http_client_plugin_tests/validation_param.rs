use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::DustPlugin;

use super::support::{config, future_of, http_client_class, library_for, method, param};

#[test]
fn rejects_duplicate_header_keys_across_class_and_parameters() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config(
            "HttpClient",
            Some("(headers: {'x-auth': 'static-token'})"),
        )],
        vec![method(
            "listUsers",
            future_of(TypeIr::generic("List", vec![TypeIr::named("User")])),
            vec![config("GET", Some("('/users')"))],
            vec![param(
                "token",
                TypeIr::string(),
                vec![config("Header", Some("('x-auth')"))],
            )],
        )],
    )));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("duplicate `@Header` key `x-auth`")
    }));
}

#[test]
fn rejects_form_fields_without_form_urlencoded() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "rename",
            future_of(TypeIr::named("User")),
            vec![config("PATCH", Some("('/users/{id}')"))],
            vec![
                param("id", TypeIr::string(), vec![config("Path", Some("('id')"))]),
                param(
                    "title",
                    TypeIr::string(),
                    vec![config("Field", Some("('title')"))],
                ),
            ],
        )],
    )));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("uses `@Field()` without `@FormUrlEncoded()`")
    }));
}

#[test]
fn rejects_parts_without_multipart() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "upload",
            future_of(TypeIr::named("void")),
            vec![config("POST", Some("('/files')"))],
            vec![param(
                "file",
                TypeIr::named("MultipartFile"),
                vec![config("Part", Some("('file')"))],
            )],
        )],
    )));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("uses `@Part()` without `@MultiPart()`")
    }));
}

#[test]
fn rejects_body_modes_on_non_body_verbs() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![
            method(
                "search",
                future_of(TypeIr::named("User")),
                vec![config("GET", Some("('/users/search')"))],
                vec![param(
                    "payload",
                    TypeIr::named("SearchRequest"),
                    vec![config("Body", Some("()"))],
                )],
            ),
            method(
                "headForm",
                future_of(TypeIr::named("void")),
                vec![
                    config("FormUrlEncoded", Some("()")),
                    config("HEAD", Some("('/users')")),
                ],
                vec![param(
                    "name",
                    TypeIr::string(),
                    vec![config("Field", Some("('name')"))],
                )],
            ),
            method(
                "optionsUpload",
                future_of(TypeIr::named("void")),
                vec![
                    config("MultiPart", Some("()")),
                    config("OPTIONS", Some("('/files')")),
                ],
                vec![param(
                    "file",
                    TypeIr::named("MultipartFile"),
                    vec![config("Part", Some("('file')"))],
                )],
            ),
        ],
    )));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("does not allow `@Body()` for `GET` requests")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("does not allow `@FormUrlEncoded()` for `HEAD` requests")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("does not allow `@MultiPart()` for `OPTIONS` requests")
    }));
}

#[test]
fn accepts_delete_request_bodies() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![
            method(
                "deleteBody",
                future_of(TypeIr::named("void")),
                vec![config("DELETE", Some("('/users')"))],
                vec![param(
                    "payload",
                    TypeIr::named("DeleteRequest"),
                    vec![config("Body", Some("()"))],
                )],
            ),
            method(
                "deleteForm",
                future_of(TypeIr::named("void")),
                vec![
                    config("FormUrlEncoded", Some("()")),
                    config("DELETE", Some("('/users/form')")),
                ],
                vec![param(
                    "reason",
                    TypeIr::string(),
                    vec![config("Field", Some("('reason')"))],
                )],
            ),
            method(
                "deleteMultipart",
                future_of(TypeIr::named("void")),
                vec![
                    config("MultiPart", Some("()")),
                    config("DELETE", Some("('/users/multipart')")),
                ],
                vec![param(
                    "file",
                    TypeIr::named("MultipartFile"),
                    vec![config("Part", Some("('file')"))],
                )],
            ),
        ],
    )));

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn rejects_duplicate_prefixed_cancel_tokens() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "fetch",
            future_of(TypeIr::named("User")),
            vec![config("GET", Some("('/users/{id}')"))],
            vec![
                param("first", TypeIr::named("dio.CancelToken"), Vec::new()),
                param("second", TypeIr::named("CancelToken"), Vec::new()),
            ],
        )],
    )));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("can only declare one `CancelToken` parameter")
    }));
}

#[test]
fn rejects_queries_with_non_string_keyed_maps() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "listUsers",
            future_of(TypeIr::generic("List", vec![TypeIr::named("User")])),
            vec![config("GET", Some("('/users')"))],
            vec![param(
                "query",
                TypeIr::generic("Map", vec![TypeIr::int(), TypeIr::dynamic()]),
                vec![config("Queries", Some("()"))],
            )],
        )],
    )));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("uses `@Queries()` but must have type `Map<String, ...>`")
    }));
}

#[test]
fn rejects_header_maps_with_non_string_keyed_maps() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "fetch",
            future_of(TypeIr::named("User")),
            vec![config("GET", Some("('/users/{id}')"))],
            vec![
                param("id", TypeIr::string(), vec![config("Path", Some("('id')"))]),
                param(
                    "headers",
                    TypeIr::generic("Map", vec![TypeIr::int(), TypeIr::string()]),
                    vec![config("HeaderMap", Some("()"))],
                ),
            ],
        )],
    )));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("uses `@HeaderMap()` but must have type `Map<String, ...>`")
    }));
}
