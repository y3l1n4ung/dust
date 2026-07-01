use dust_http_client_plugin::register_plugin;
use dust_ir::{SymbolId, TraitApplicationIr, TypeIr};
use dust_plugin_api::DustPlugin;

use super::support::{
    config, future_of, http_client_class, library_for, library_with_classes, method, param,
    serde_model_class, span,
};

#[test]
fn rejects_non_interface_targets() {
    let plugin = register_plugin();
    let mut class = http_client_class(vec![config("HttpClient", Some("()"))], Vec::new());
    class.is_interface = false;
    let diagnostics = plugin.validate(&library_for(class));

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("abstract interface class"));
}

#[test]
fn rejects_missing_path_bindings() {
    let plugin = register_plugin();
    let endpoint = method(
        "getUser",
        future_of(TypeIr::named("User")),
        vec![config("GET", Some("('/users/{id}')"))],
        vec![param("id", TypeIr::string(), Vec::new())],
    );
    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![endpoint],
    )));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("is missing `@Path('id')` for route placeholder `{id}`")
    }));
}

#[test]
fn rejects_methods_with_bodies() {
    let plugin = register_plugin();
    let mut endpoint = method(
        "saveUser",
        future_of(TypeIr::named("void")),
        vec![config("POST", Some("('/users')"))],
        vec![param(
            "user",
            TypeIr::named("User"),
            vec![config("Body", Some("()"))],
        )],
    );
    endpoint.has_body = true;

    let diagnostics = plugin.validate(&library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![endpoint],
    )));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("must not declare an implementation body")
    }));
}

#[test]
fn rejects_unsupported_body_model_serialization() {
    let plugin = register_plugin();
    let api = http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "saveUser",
            future_of(TypeIr::named("void")),
            vec![config("POST", Some("('/users')"))],
            vec![param(
                "user",
                TypeIr::named("User"),
                vec![config("Body", Some("()"))],
            )],
        )],
    );
    let user = serde_model_class("User", Vec::new());

    let diagnostics = plugin.validate(&library_with_classes(vec![api, user]));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains(
            "parameter `user` on `Api.saveUser` uses `@Body()` with `User` but generated HTTP serialization requires `User.toJson()`",
        )
    }));
}

#[test]
fn rejects_unsupported_response_model_deserialization() {
    let plugin = register_plugin();
    let api = http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "getUser",
            future_of(TypeIr::named("User")),
            vec![config("GET", Some("('/users')"))],
            Vec::new(),
        )],
    );
    let mut user = serde_model_class("User", Vec::new());
    user.constructors.clear();

    let diagnostics = plugin.validate(&library_with_classes(vec![api, user]));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains(
            "method `getUser` on `Api` returns `User` but generated HTTP deserialization requires `User.fromJson(Map<String, Object?>)`",
        )
    }));
}

#[test]
fn accepts_json_capable_body_and_response_models() {
    let plugin = register_plugin();
    let api = http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "saveUser",
            future_of(TypeIr::named("User")),
            vec![config("POST", Some("('/users')"))],
            vec![param(
                "user",
                TypeIr::named("User"),
                vec![config("Body", Some("()"))],
            )],
        )],
    );
    let mut user = serde_model_class("User", Vec::new());
    user.traits.push(TraitApplicationIr {
        symbol: SymbolId::new("dust_dart::Serialize"),
        span: span(1, 2),
    });

    let diagnostics = plugin.validate(&library_with_classes(vec![api, user]));

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}
