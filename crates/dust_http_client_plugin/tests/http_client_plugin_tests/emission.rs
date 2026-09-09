use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use super::support::{
    config, future_of, http_client_class, library_for, library_for_with_imports, method,
    named_param, named_param_with_default, param,
};

/// Literal escaping, parameter defaults and the generated test file.
#[path = "emission/literals.rs"]
mod literals;

#[test]
fn emits_dio_client_with_inherited_isolate_decode() {
    let plugin = register_plugin();
    let library = library_for_with_imports(
        http_client_class(
            vec![config(
                "HttpClient",
                Some("(baseUrl: 'https://api.example.com', parseThread: HttpParseThread.isolate)"),
            )],
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
        vec!["dart:isolate"],
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
    let emitted = contribution.support_types.join("\n");
    let helpers = contribution.top_level_functions.join("\n");

    assert!(emitted.contains("final class _$Api implements Api"));
    assert!(emitted.contains(
        "final _options = Options(
      method: 'GET',"
    ));
    assert!(emitted.contains("_dio.fetch<Map<String, dynamic>>"));
    assert!(emitted.contains("Uri.encodeComponent(id.toString())"));
    assert!(emitted.contains(
        "_combineBaseUrls(
                _dio.options.baseUrl,
                _baseUrl ?? 'https://api.example.com',
              )"
    ));
    assert!(emitted.contains("await Isolate.run(() => _$Api_getUser_Decode(_result.data!))"));
    assert!(helpers.contains("User _$Api_getUser_Decode(dynamic json)"));
    assert!(helpers.contains("User.fromJson(json as Map<String, dynamic>)"));
}

#[test]
fn emits_flutter_target_isolate_decode_with_compute() {
    let plugin = register_plugin();
    let library = library_for_with_imports(
        http_client_class(
            vec![config(
                "HttpClient",
                Some("(target: HttpTarget.flutter, parseThread: HttpParseThread.isolate)"),
            )],
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
        vec!["package:flutter/foundation.dart"],
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
    let emitted = contribution.support_types.join("\n");
    let helpers = contribution.top_level_functions.join("\n");

    assert!(emitted.contains("await compute(_$Api_getUser_Decode, _result.data!)"));
    assert!(!emitted.contains("Isolate.run"));
    assert!(helpers.contains("User _$Api_getUser_Decode(dynamic json)"));
}

#[test]
fn method_parse_override_can_opt_in_one_endpoint() {
    let plugin = register_plugin();
    let library = library_for_with_imports(
        http_client_class(
            vec![config("HttpClient", Some("(target: HttpTarget.flutter)"))],
            vec![
                method(
                    "getUser",
                    future_of(TypeIr::named("User")),
                    vec![
                        config("GET", Some("('/users/{id}')")),
                        config("HttpParse", Some("(thread: HttpParseThread.isolate)")),
                    ],
                    vec![param(
                        "id",
                        TypeIr::string(),
                        vec![config("Path", Some("('id')"))],
                    )],
                ),
                method(
                    "getStatus",
                    future_of(TypeIr::named("Status")),
                    vec![config("GET", Some("('/status')"))],
                    Vec::new(),
                ),
            ],
        ),
        vec!["package:flutter/foundation.dart"],
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
    let emitted = contribution.support_types.join("\n");
    let helpers = contribution.top_level_functions.join("\n");

    assert!(emitted.contains("await compute(_$Api_getUser_Decode, _result.data!)"));
    assert!(emitted.contains("return Status.fromJson(_result.data as Map<String, dynamic>);"));
    assert!(helpers.contains("_$Api_getUser_Decode"));
    assert!(!helpers.contains("_$Api_getStatus_Decode"));
}
