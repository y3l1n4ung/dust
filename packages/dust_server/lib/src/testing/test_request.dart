import 'dart:convert';

import 'package:meta/meta.dart';
import 'package:shelf/shelf.dart';

import '../request/request_parts.dart' show pathParametersKey;
import 'test_response.dart';

/// A request being built for a `TestClient`.
///
/// ```dart
/// await (client.post('/orders')
///     ..bearer('tok_abc')
///     ..json({'item': 'shirt', 'quantity': 2})
///     ..expectSuccess())
///     .send();
/// ```
final class TestRequest {
  /// @nodoc
  @internal
  TestRequest(this._method, this._path, this._execute,
      {bool? expectSuccess, Map<String, String>? cookies})
      : _expectSuccess = expectSuccess,
        _cookies = cookies != null ? Map.of(cookies) : {};

  final String _method;
  final String _path;
  final Future<TestResponse> Function(
          String method, String path, Map<String, String> headers, Object? body)
      _execute;

  final Map<String, String> _headers = {};
  final Map<String, String> _queryParams = {};
  final Map<String, String> _cookies;
  Object? _body;
  bool? _expectSuccess;

  /// JSON-encodes [body] and sets content-type.
  void json(Object body) {
    _headers['content-type'] = 'application/json';
    _body = jsonEncode(body);
  }

  /// Plain text [body].
  void text(String body) {
    _headers['content-type'] = 'text/plain; charset=utf-8';
    _body = body;
  }

  /// Form-urlencoded [fields].
  void form(Map<String, String> fields) {
    _headers['content-type'] = 'application/x-www-form-urlencoded';
    _body = Uri(queryParameters: fields).query;
  }

  /// Raw bytes with an explicit [contentType].
  void bytes(
    List<int> data, {
    String contentType = 'application/octet-stream',
  }) {
    _headers['content-type'] = contentType;
    _body = data;
  }

  /// Sets content-type without setting a body.
  void contentType(String mediaType) {
    _headers['content-type'] = mediaType;
  }

  /// One header.
  void header(String name, String value) {
    _headers[name] = value;
  }

  /// `Authorization: Bearer [token]`.
  void bearer(String token) {
    _headers['authorization'] = 'Bearer $token';
  }

  /// `Authorization: Basic base64(username:password)`.
  void basic(String username, String password) {
    final encoded = base64Encode(utf8.encode('$username:$password'));
    _headers['authorization'] = 'Basic $encoded';
  }

  /// Appends a cookie.
  void cookie(String name, String value) {
    _cookies[name] = value;
  }

  /// Appends a query parameter.
  void queryParam(String name, String value) {
    _queryParams[name] = value;
  }

  /// Assert 2xx after send.
  void expectSuccess() {
    _expectSuccess = true;
  }

  /// Assert non-2xx after send.
  void expectFailure() {
    _expectSuccess = false;
  }

  /// Executes the request.
  ///
  /// When an expectation was set, the assertion runs before the response is
  /// returned — a forgotten status check shows up at the call site, not three
  /// assertions later.
  Future<TestResponse> send() async {
    final allHeaders = Map<String, String>.of(_headers);
    if (_cookies.isNotEmpty) {
      allHeaders['cookie'] =
          _cookies.entries.map((e) => '${e.key}=${e.value}').join('; ');
    }

    var path = _path;
    if (_queryParams.isNotEmpty) {
      final query = Uri(queryParameters: _queryParams).query;
      path = path.contains('?') ? '$path&$query' : '$path?$query';
    }

    final response = await _execute(_method, path, allHeaders, _body);

    switch (_expectSuccess) {
      case true:
        response.assertSuccess();
      case false:
        response.assertFailure();
      case null:
        break;
    }

    return response;
  }
}

/// Builds a shelf [Request] for testing extractors without a router.
Request buildRequest(
  String method,
  String path, {
  Map<String, String> headers = const {},
  Object? body,
  Map<String, String> pathParameters = const {},
  Map<String, Object> context = const {},
}) {
  return Request(
    method,
    Uri.parse('http://localhost$path'),
    headers: headers,
    body: body,
    context: {
      if (pathParameters.isNotEmpty) pathParametersKey: pathParameters,
      ...context,
    },
  );
}
