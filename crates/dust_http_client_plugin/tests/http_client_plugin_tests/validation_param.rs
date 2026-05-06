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
