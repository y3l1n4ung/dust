use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use super::support::{
    config, future_of, http_client_class, library_for, library_for_with_imports, method, param,
};

#[test]
fn emits_dio_client_with_inherited_isolate_decode() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config(
            "HttpClient",
            Some("(baseUrl: 'https://api.example.com', parseThread: DustParseThread.isolate)"),
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
    ));

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let emitted = contribution.support_types.join("\n");
    let helpers = contribution.top_level_functions.join("\n");

    assert!(emitted.contains("final class _$Api implements Api"));
    assert!(emitted.contains("Options(method: 'GET'"));
    assert!(emitted.contains("_dio.fetch<Map<String, dynamic>>"));
    assert!(emitted.contains("Uri.encodeComponent(id.toString())"));
    assert!(
        emitted.contains(
            "_combineBaseUrls(_dio.options.baseUrl, _baseUrl ?? 'https://api.example.com')"
        )
    );
    assert!(emitted.contains("await Isolate.run(() => _$Api_getUser_Decode(_result.data!))"));
    assert!(helpers.contains("User _$Api_getUser_Decode(dynamic json)"));
    assert!(helpers.contains("User.fromJson(json as Map<String, dynamic>)"));
}

#[test]
fn escapes_generated_single_quoted_literals() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config(
            "HttpClient",
            Some(r#"(baseUrl: 'https://api.example.com/$tenant', headers: {"x-'$": "tok'$"})"#),
        )],
        vec![method(
            "ping",
            future_of(TypeIr::named("void")),
            vec![config("GET", Some("('/ping/$health')"))],
            Vec::new(),
        )],
    ));

    let emitted = plugin
        .emit(&library, &SymbolPlan::default())
        .support_types
        .join("\n");
    assert_eq!(
        emitted,
        r#"final class _$Api implements Api {
  _$Api(this._dio, {String? baseUrl}) : _baseUrl = baseUrl;

  final Dio _dio;
  final String? _baseUrl;

  @override
  Future<void> ping() async {
    final _queryParameters = <String, dynamic>{};
    final _headers = <String, dynamic>{};
    final _extra = <String, dynamic>{};
    _headers['x-\'\$'] = 'tok\'\$';
    final Object? _data = null;
    final _options = Options(method: 'GET', headers: _headers, extra: _extra, contentType: null);
    await _dio.fetch<void>(
      _setStreamType<void>(
        _options
            .compose(
              _dio.options,
              '/ping/\$health',
              queryParameters: _queryParameters,
              data: _data,
              cancelToken: null,
              onSendProgress: null,
              onReceiveProgress: null,
            )
            .copyWith(
              baseUrl: _combineBaseUrls(_dio.options.baseUrl, _baseUrl ?? 'https://api.example.com/\$tenant'),
            ),
      ),
    );
    return;
  }
}
"#
    );
}

#[test]
fn emits_generate_test_auxiliary_file() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![
            config("HttpClient", Some("()")),
            config("GenerateTest", Some("()")),
        ],
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

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    assert_eq!(contribution.auxiliary_outputs.len(), 1);
    let generated = &contribution.auxiliary_outputs[0];

    assert!(
        generated
            .output_path
            .ends_with("test/generated/api_test.dart")
    );
    assert!(generated.source.contains("void main() {"));
    assert!(generated.source.contains("group('Api request mapping'"));
    assert!(generated.source.contains("await api.getUser('dust-id');"));
    assert!(
        generated
            .source
            .contains("import 'package:dust_test/api.dart';")
    );
}

#[test]
fn rewrites_relative_source_imports_for_generated_tests() {
    let plugin = register_plugin();
    let library = library_for_with_imports(
        http_client_class(
            vec![
                config("HttpClient", Some("()")),
                config("GenerateTest", Some("()")),
            ],
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

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let generated = &contribution.auxiliary_outputs[0];

    assert!(
        generated
            .source
            .contains("import 'package:dust_test/models/user.dart';")
    );
}
