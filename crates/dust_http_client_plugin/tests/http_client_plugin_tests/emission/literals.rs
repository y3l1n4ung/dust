//! Literal escaping, parameter defaults and the generated test file.

use super::*;

/// The auxiliary test file a generated client can emit.
#[path = "literals/testfile.rs"]
mod testfile;

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
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution")
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
    final _options = Options(
      method: 'GET',
      headers: _headers,
      extra: _extra,
      contentType: null,
    );
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
              baseUrl: _combineBaseUrls(
                _dio.options.baseUrl,
                _baseUrl ?? 'https://api.example.com/\$tenant',
              ),
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
fn preserves_named_parameter_defaults_in_generated_client_methods() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "fetchTemplates",
            future_of(TypeIr::generic(
                "List",
                vec![TypeIr::named("HabitTemplate")],
            )),
            vec![config("GET", Some("('/api/v1/habits/templates')"))],
            vec![
                named_param_with_default(
                    "locale",
                    TypeIr::string(),
                    "'en'",
                    vec![config("Query", Some("('locale')"))],
                ),
                named_param_with_default(
                    "isActive",
                    TypeIr::bool(),
                    "true",
                    vec![config("Query", Some("('is_active')"))],
                ),
                named_param(
                    "category",
                    TypeIr::string().nullable(),
                    vec![config("Query", Some("('category')"))],
                ),
            ],
        )],
    ));

    let emitted = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution")
        .support_types
        .join("\n");

    assert_eq!(
        emitted,
        r#"final class _$Api implements Api {
  _$Api(this._dio, {String? baseUrl}) : _baseUrl = baseUrl;

  final Dio _dio;
  final String? _baseUrl;

  @override
  Future<List<HabitTemplate>> fetchTemplates({
    String locale = 'en',
    bool isActive = true,
    String? category,
  }) async {
    final _queryParameters = <String, dynamic>{};
    final _headers = <String, dynamic>{};
    final _extra = <String, dynamic>{};
    _queryParameters['locale'] = locale;
    _queryParameters['is_active'] = isActive;
    if (category != null) _queryParameters['category'] = category;
    final Object? _data = null;
    final _options = Options(
      method: 'GET',
      headers: _headers,
      extra: _extra,
      contentType: null,
    );
    final _result = await _dio.fetch<List<dynamic>>(
      _setStreamType<List<HabitTemplate>>(
        _options
            .compose(
              _dio.options,
              '/api/v1/habits/templates',
              queryParameters: _queryParameters,
              data: _data,
              cancelToken: null,
              onSendProgress: null,
              onReceiveProgress: null,
            )
            .copyWith(
              baseUrl: _combineBaseUrls(
                _dio.options.baseUrl,
                _baseUrl,
              ),
            ),
      ),
    );
    return (_result.data as List<dynamic>)
    .map((item) => HabitTemplate.fromJson(item as Map<String, dynamic>))
    .toList();
  }
}
"#
    );
}
