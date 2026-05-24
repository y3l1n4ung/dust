use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use super::support::{
    config, future_of, http_client_class, library_for, method, named_param, param,
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
