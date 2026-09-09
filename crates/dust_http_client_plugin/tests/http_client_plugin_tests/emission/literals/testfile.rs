//! The auxiliary test file a generated client can emit.

use super::*;

#[test]
fn emits_generate_test_auxiliary_file() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("(generateTest: true)"))],
        vec![method(
            "getUser",
            future_of(TypeIr::named("User")),
            vec![config("GET", Some("('/users/{id}')"))],
            vec![param(
                "id",
                TypeIr::string(),
                vec![config("Path", Some("('id')"))],
            )],
        )],
    ));

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    assert_eq!(contribution.auxiliary_outputs.len(), 1);
    let generated = &contribution.auxiliary_outputs[0];

    assert!(
        generated
            .output_path
            .ends_with("test/generated/api_test.dart")
    );
    assert!(generated.source.contains("void main() {"));
    assert!(generated.source.contains("group('Api request mapping'"));
    assert!(
        generated
            .source
            .contains("await expectLater(api.getUser('dust-id'), throwsA(anything));")
    );
    assert!(!generated.source.contains("catch (_)"));
    assert!(
        generated
            .source
            .contains("import 'package:dust_test/api.dart';")
    );
}

#[test]
fn omits_generate_test_auxiliary_file_by_default() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "getUser",
            future_of(TypeIr::named("User")),
            vec![config("GET", Some("('/users/{id}')"))],
            vec![param(
                "id",
                TypeIr::string(),
                vec![config("Path", Some("('id')"))],
            )],
        )],
    ));

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert!(contribution.auxiliary_outputs.is_empty());
}

#[test]
fn rewrites_relative_source_imports_for_generated_tests() {
    let plugin = register_plugin();
    let library = library_for_with_imports(
        http_client_class(
            vec![config("HttpClient", Some("(generateTest: true)"))],
            vec![method(
                "getUser",
                future_of(TypeIr::named("User")),
                vec![config("GET", Some("('/users/{id}')"))],
                vec![param(
                    "id",
                    TypeIr::string(),
                    vec![config("Path", Some("('id')"))],
                )],
            )],
        ),
        vec!["models/user.dart"],
    );

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let generated = &contribution.auxiliary_outputs[0];

    assert!(
        generated
            .source
            .contains("import 'package:dust_test/models/user.dart';")
    );
}
