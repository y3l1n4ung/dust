use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use super::helpers::SHARED_HELPERS;
use crate::http_client_plugin_tests::support::{
    config, future_of, http_client_class, library_for, library_for_with_imports, method,
};

#[test]
fn emits_stream_fetches_for_response_body_payloads() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "streamPosts",
            future_of(TypeIr::named("ResponseBody")),
            vec![config("GET", Some("('/posts/raw')"))],
            Vec::new(),
        )],
    ));

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let emitted = contribution.support_types.join("\n");
    let helpers = contribution.shared_helpers.join("\n");

    assert_eq!(
        emitted,
        r#"final class _$Api implements Api {
  _$Api(this._dio, {String? baseUrl}) : _baseUrl = baseUrl;

  final Dio _dio;
  final String? _baseUrl;

  @override
  Future<ResponseBody> streamPosts() async {
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
              '/posts/raw',
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
    return _result.data!;
  }
}
"#
    );
    assert_eq!(helpers, SHARED_HELPERS);
}

#[test]
fn returns_raw_stream_response_without_rewrapping() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "streamEnvelope",
            future_of(TypeIr::generic(
                "Response",
                vec![TypeIr::named("ResponseBody")],
            )),
            vec![config("GET", Some("('/posts/raw-response')"))],
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
  Future<Response<ResponseBody>> streamEnvelope() async {
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
              '/posts/raw-response',
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
    return _result;
  }
}
"#
    );
}

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
