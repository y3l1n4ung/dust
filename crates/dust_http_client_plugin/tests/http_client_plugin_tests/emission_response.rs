use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use super::support::{
    config, future_of, http_client_class, library_for, library_for_with_imports, method,
};

const SHARED_HELPERS: &str = r#"RequestOptions _setStreamType<T>(RequestOptions requestOptions) {
  if (T != dynamic &&
      requestOptions.responseType != ResponseType.bytes &&
      requestOptions.responseType != ResponseType.stream) {
    if (T == ResponseBody) {
      requestOptions.responseType = ResponseType.stream;
    } else if (T == String) {
      requestOptions.responseType = ResponseType.plain;
    } else {
      requestOptions.responseType = ResponseType.json;
    }
  }
  return requestOptions;
}
String _combineBaseUrls(String dioBaseUrl, String? baseUrl) {
  if (baseUrl == null || baseUrl.trim().isEmpty) {
    return dioBaseUrl;
  }
  final url = Uri.parse(baseUrl);
  if (url.isAbsolute) {
    return url.toString();
  }
  return Uri.parse(dioBaseUrl).resolveUri(url).toString();
}
Response<T> _buildResponse<T>(Response<dynamic> response, T data) {
  return Response<T>(
    data: data,
    headers: response.headers,
    isRedirect: response.isRedirect,
    redirects: response.redirects,
    requestOptions: response.requestOptions,
    statusCode: response.statusCode,
    statusMessage: response.statusMessage,
    extra: response.extra,
  );
}"#;

#[test]
fn emits_raw_response_wrappers_for_prefixed_response_types() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "fetchRaw",
            future_of(TypeIr::generic("dio.Response", vec![TypeIr::named("User")])),
            vec![config("GET", Some("('/users/raw')"))],
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
  Future<dio.Response<User>> fetchRaw() async {
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
    final _result = await _dio.fetch<Map<String, dynamic>>(
      _setStreamType<User>(
        _options
            .compose(
              _dio.options,
              '/users/raw',
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
    final _value = User.fromJson(_result.data as Map<String, dynamic>);
    return _buildResponse<User>(_result, _value);
  }
}
"#
    );
    assert_eq!(helpers, SHARED_HELPERS);
}

#[test]
fn omits_result_binding_for_void_methods() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "deleteUser",
            future_of(TypeIr::named("void")),
            vec![config("DELETE", Some("('/users/me')"))],
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
  Future<void> deleteUser() async {
    final _queryParameters = <String, dynamic>{};
    final _headers = <String, dynamic>{};
    final _extra = <String, dynamic>{};
    final Object? _data = null;
    final _options = Options(
      method: 'DELETE',
      headers: _headers,
      extra: _extra,
      contentType: null,
    );
    await _dio.fetch<void>(
      _setStreamType<void>(
        _options
            .compose(
              _dio.options,
              '/users/me',
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
    return;
  }
}
"#
    );
}

#[test]
fn emits_direct_map_casts_for_map_payload_responses() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "createPost",
            future_of(TypeIr::generic(
                "Map",
                vec![TypeIr::string(), TypeIr::dynamic()],
            )),
            vec![config("POST", Some("('/posts')"))],
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
  Future<Map<String, dynamic>> createPost() async {
    final _queryParameters = <String, dynamic>{};
    final _headers = <String, dynamic>{};
    final _extra = <String, dynamic>{};
    final Object? _data = null;
    final _options = Options(
      method: 'POST',
      headers: _headers,
      extra: _extra,
      contentType: null,
    );
    final _result = await _dio.fetch<Map<String, dynamic>>(
      _setStreamType<Map<String, dynamic>>(
        _options
            .compose(
              _dio.options,
              '/posts',
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
    return _result.data as Map<String, dynamic>;
  }
}
"#
    );
}

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
