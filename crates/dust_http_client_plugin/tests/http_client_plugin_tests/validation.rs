use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::DustPlugin;

use super::support::{config, future_of, http_client_class, library_for, method, param};

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
