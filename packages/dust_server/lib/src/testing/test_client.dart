import 'dart:convert';
import 'dart:io';

import 'package:shelf/shelf.dart';

import '../router/router_base.dart';
import '../serving/graceful.dart' as serving;
import 'test_request.dart';
import 'test_response.dart';

/// Drives a [Router] through either an in-process handler or real HTTP.
///
/// ```dart
/// // In-process — no socket, sub-millisecond.
/// final client = TestClient(router);
///
/// // Real HTTP on port 0 — catches wire-level bugs.
/// final client = await TestClient.serve(router);
/// ```
///
/// ```dart
/// final response = await (client.post('/orders')
///     ..bearer('tok')
///     ..json({'item': 'shirt', 'quantity': 2})
///     ..expectSuccess())
///     .send();
/// response
///     ..assertCreated()
///     ..assertJsonContains({'item': 'shirt'});
/// ```
final class TestClient {
  /// No socket — requests go through `router.handler` directly.
  TestClient(Router router, {bool saveCookies = false})
      : _handler = router.handler,
        _server = null,
        _origin = null,
        _httpClient = null,
        _saveCookies = saveCookies;

  TestClient._(this._handler, this._server, this._origin, this._httpClient,
      {bool saveCookies = false})
      : _saveCookies = saveCookies;

  /// Connects to a server already running at [origin].
  ///
  /// Does not start or stop the server — the caller manages its lifecycle.
  TestClient.origin(String origin, {bool saveCookies = false})
      : _handler = null,
        _server = null,
        _origin = origin,
        _httpClient = HttpClient(),
        _saveCookies = saveCookies;

  /// Binds port 0 on loopback and drives with `dart:io` [HttpClient].
  ///
  /// Port 0 lets the OS pick a free port, so tests run in parallel
  /// without collisions.
  static Future<TestClient> serve(Router router,
      {bool saveCookies = false}) async {
    final server = await serving.serve(router, InternetAddress.loopbackIPv4, 0);
    final origin = 'http://${server.address.host}:${server.port}';
    final httpClient = HttpClient();
    return TestClient._(
      null,
      server,
      origin,
      httpClient,
      saveCookies: saveCookies,
    );
  }

  final Handler? _handler;
  final serving.ServerHandle? _server;
  final String? _origin;
  final HttpClient? _httpClient;
  final bool _saveCookies;
  bool? _defaultExpectSuccess;
  final Map<String, String> _cookieJar = {};

  /// The base URL of the running server. Throws in handler mode.
  String get origin {
    final o = _origin;
    if (o == null) {
      throw StateError(
          'origin is only available in serve mode — use TestClient.serve()');
    }
    return o;
  }

  /// GET [path].
  TestRequest get(String path) => _request('GET', path);

  /// POST [path].
  TestRequest post(String path) => _request('POST', path);

  /// PUT [path].
  TestRequest put(String path) => _request('PUT', path);

  /// PATCH [path].
  TestRequest patch(String path) => _request('PATCH', path);

  /// DELETE [path].
  TestRequest delete(String path) => _request('DELETE', path);

  /// HEAD [path].
  TestRequest head(String path) => _request('HEAD', path);

  /// Arbitrary [verb] on [path].
  TestRequest method(String verb, String path) => _request(verb, path);

  /// Every response must be 2xx. Override per-request with
  /// [TestRequest.expectFailure].
  void expectSuccess() {
    _defaultExpectSuccess = true;
  }

  /// Every response must be non-2xx. Override per-request with
  /// [TestRequest.expectSuccess].
  void expectFailure() {
    _defaultExpectSuccess = false;
  }

  /// Stops the server (in serve mode) and closes the HTTP client.
  Future<void> close() async {
    _httpClient?.close(force: true);
    await _server?.close(drain: const Duration(seconds: 1));
  }

  TestRequest _request(String verb, String path) {
    return TestRequest(
      verb,
      path,
      _execute,
      expectSuccess: _defaultExpectSuccess,
      cookies: _saveCookies ? _cookieJar : null,
    );
  }

  Future<TestResponse> _execute(
    String method,
    String path,
    Map<String, String> headers,
    Object? body,
  ) async {
    final TestResponse response;

    if (_origin != null) {
      response = await _executeHttp(method, path, headers, body);
    } else {
      response = await _executeHandler(method, path, headers, body);
    }

    if (_saveCookies) {
      _extractCookies(response.headersAll);
    }

    return response;
  }

  Future<TestResponse> _executeHandler(
    String method,
    String path,
    Map<String, String> headers,
    Object? body,
  ) async {
    final request = buildRequest(method, path, headers: headers, body: body);
    final shelfResponse = await _handler!(request);
    return TestResponse.fromShelf(shelfResponse);
  }

  Future<TestResponse> _executeHttp(
    String method,
    String path,
    Map<String, String> headers,
    Object? body,
  ) async {
    final uri = Uri.parse('$_origin$path');
    final request = await _httpClient!.openUrl(method, uri);
    for (final entry in headers.entries) {
      request.headers.set(entry.key, entry.value);
    }

    if (body != null) {
      final bytes = body is List<int>
          ? body
          : utf8.encode(body is String ? body : body.toString());
      request.contentLength = bytes.length;
      request.add(bytes);
    }

    final ioResponse = await request.close();
    final responseBytes = <int>[];
    await for (final chunk in ioResponse) {
      responseBytes.addAll(chunk);
    }
    final responseHeaders = <String, List<String>>{};
    ioResponse.headers.forEach((name, values) {
      responseHeaders[name] = values;
    });

    return TestResponse(
      ioResponse.statusCode,
      responseHeaders,
      responseBytes,
    );
  }

  /// Reads every `set-cookie` the response sent, not the first.
  ///
  /// A joined header cannot be split back apart: an `Expires` date carries its
  /// own comma, so a response setting two cookies used to contribute one, and
  /// the second was dropped without a word.
  void _extractCookies(Map<String, List<String>> responseHeaders) {
    for (final setCookie in responseHeaders['set-cookie'] ?? const <String>[]) {
      final pair = setCookie.split(';').first;
      final equalsIndex = pair.indexOf('=');
      if (equalsIndex < 0) continue;
      _cookieJar[pair.substring(0, equalsIndex).trim()] =
          pair.substring(equalsIndex + 1).trim();
    }
  }
}
