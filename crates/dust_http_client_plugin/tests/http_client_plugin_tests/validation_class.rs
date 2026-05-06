use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::DustPlugin;

use super::support::{config, future_of, http_client_class, library_for, method};

#[test]
fn rejects_generate_test_without_http_client() {
    let plugin = register_plugin();
    let class = http_client_class(vec![config("GenerateTest", Some("()"))], Vec::new());
    let diagnostics = plugin.validate(&library_for(class));

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("requires `@HttpClient()`"));
}

#[test]
fn rejects_factory_with_wrong_redirect_target() {
    let plugin = register_plugin();
    let mut class = http_client_class(vec![config("HttpClient", Some("()"))], Vec::new());
    class.constructors[0].redirected_target_name = Some("_$OtherApi".to_owned());
    let diagnostics = plugin.validate(&library_for(class));

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("must redirect to `_$Api`"))
    );
}

#[test]
fn accepts_prefixed_dio_factory_parameter_types() {
    let plugin = register_plugin();
    let mut class = http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "healthcheck",
            future_of(TypeIr::named("void")),
            vec![config("GET", Some("('/health')"))],
            Vec::new(),
        )],
    );
    class.constructors[0].params[0].ty = TypeIr::named("dio.Dio");

    let diagnostics = plugin.validate(&library_for(class));
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn rejects_non_nullable_factory_base_url_parameter() {
    let plugin = register_plugin();
    let mut class = http_client_class(vec![config("HttpClient", Some("()"))], Vec::new());
    class.constructors[0].params[1].ty = TypeIr::string();
    let diagnostics = plugin.validate(&library_for(class));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("only supports optional named `String? baseUrl` without a default")
    }));
}
