//! Byte and text stream methods returned without rewrapping.

use super::*;

#[test]
fn emits_direct_byte_stream_methods() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "streamBytes",
            TypeIr::generic("Stream", vec![TypeIr::list_of(TypeIr::int())]),
            vec![config("GET", Some("('/posts/bytes')"))],
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
  Stream<List<int>> streamBytes() async* {
    final _queryParameters = <String, dynamic>{};
    final _headers = <String, dynamic>{};
    final _extra = <String, dynamic>{};
    final Object? _data = null;
    final _options = Options(
      method: 'GET',
      headers: _headers,
      extra: _extra,
      contentType: null,
    );
    final _result = await _dio.fetch<ResponseBody>(
      _setStreamType<ResponseBody>(
        _options
            .compose(
              _dio.options,
              '/posts/bytes',
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
    final _body = _result.data;
    if (_body == null) return;
    yield* _body.stream;
  }
}
"#
    );
}

#[test]
fn emits_direct_text_stream_methods() {
    let plugin = register_plugin();
    let library = library_for_with_imports(
        http_client_class(
            vec![config("HttpClient", Some("()"))],
            vec![method(
                "streamText",
                TypeIr::generic("Stream", vec![TypeIr::string()]),
                vec![config("GET", Some("('/posts/text')"))],
                Vec::new(),
            )],
        ),
        vec!["dart:convert"],
    );

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
  Stream<String> streamText() async* {
    final _queryParameters = <String, dynamic>{};
    final _headers = <String, dynamic>{};
    final _extra = <String, dynamic>{};
    final Object? _data = null;
    final _options = Options(
      method: 'GET',
      headers: _headers,
      extra: _extra,
      contentType: null,
    );
    final _result = await _dio.fetch<ResponseBody>(
      _setStreamType<ResponseBody>(
        _options
            .compose(
              _dio.options,
              '/posts/text',
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
    final _body = _result.data;
    if (_body == null) return;
    yield* utf8.decoder.bind(_body.stream);
  }
}
"#
    );
}
