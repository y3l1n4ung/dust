import 'dart:convert';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:test/test.dart';

/// Serves one example on a loopback port for the length of one test.
///
/// The one-question examples are each a `buildApp()` and a `main`. This drives
/// the router the same way a browser would, over a real socket, because an
/// example that works when its handler is called directly and 500s over HTTP is
/// still a broken example.
///
/// Port 0 rather than the number in the example's `main`: a suite that fails
/// because something else holds 8080 has told you nothing.
final class ExampleApp {
  ExampleApp._(this._server, this.records);

  /// Starts [app].
  static Future<ExampleApp> serve(
    Router app, {
    List<AccessRecord>? records,
  }) async {
    final server = await serveRouter(app, InternetAddress.loopbackIPv4, 0);
    return ExampleApp._(server, records ?? []);
  }

  final ServerHandle _server;

  /// Whatever the example's access log recorded, when it has one.
  final List<AccessRecord> records;

  /// Where to point a client.
  String get origin => 'http://${_server.address.host}:${_server.port}';

  /// Requests still being handled.
  int get inFlight => _server.inFlight;

  /// The full URL of [path].
  Uri uri(String path) => Uri.parse('$origin$path');

  /// Sends a request, encoding [body] as JSON when there is one.
  Future<http.Response> send(
    String method,
    String path, {
    Object? body,
    Map<String, String> headers = const {},
  }) {
    final all = {
      if (body != null) 'content-type': 'application/json',
      ...headers,
    };
    final encoded = switch (body) {
      null => null,
      final String text => text,
      _ => jsonEncode(body),
    };

    return switch (method) {
      'GET' => http.get(uri(path), headers: all),
      'HEAD' => http.head(uri(path), headers: all),
      'POST' => http.post(uri(path), headers: all, body: encoded),
      'PUT' => http.put(uri(path), headers: all, body: encoded),
      'PATCH' => http.patch(uri(path), headers: all, body: encoded),
      'DELETE' => http.delete(uri(path), headers: all),
      _ => throw ArgumentError('unsupported method "$method"'),
    };
  }

  /// A `GET`.
  Future<http.Response> get(String path,
          {Map<String, String> headers = const {}}) =>
      send('GET', path, headers: headers);

  /// A `POST` carrying [body].
  Future<http.Response> post(
    String path,
    Object? body, {
    Map<String, String> headers = const {},
  }) =>
      send('POST', path, body: body, headers: headers);

  /// Sends a method the helpers above do not cover, without a body.
  ///
  /// `OPTIONS` in particular: a CORS preflight cannot be sent through
  /// `package:http`'s named methods.
  Future<http.StreamedResponse> raw(
    String method,
    String path, {
    Map<String, String> headers = const {},
  }) {
    return http.Client().send(
      http.Request(method, uri(path))..headers.addAll(headers),
    );
  }

  /// The decoded JSON object body.
  Map<String, Object?> object(http.Response response) =>
      jsonDecode(response.body) as Map<String, Object?>;

  /// The decoded JSON array body.
  List<Object?> array(http.Response response) =>
      jsonDecode(response.body) as List<Object?>;

  /// Stops the server, draining what is in flight.
  Future<void> stop() => _server.close(drain: const Duration(seconds: 2));
}

/// Serves [app] for one test and closes it afterwards.
///
/// ```dart
/// test('answers', () async {
///   final app = await example(buildApp());
///   expect((await app.get('/')).statusCode, 200);
/// });
/// ```
///
/// The teardown is registered here rather than left to the caller, because a
/// suite of thirty examples is thirty chances to leak a socket.
Future<ExampleApp> example(Router app, {List<AccessRecord>? records}) async {
  final served = await ExampleApp.serve(app, records: records);
  addTearDown(served.stop);
  return served;
}
