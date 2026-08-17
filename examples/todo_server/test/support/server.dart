import 'dart:convert';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:todo_server/todo_server.dart';

export 'package:todo_server/todo_server.dart';

/// The example application, served on a loopback port for one test group.
///
/// Driving it over a real socket is what makes these tests worth having:
/// the generated `deserialize` runs on bytes that crossed a connection, and
/// the generated `serialize` produces the bytes the client reads back.
final class ExampleServer {
  ExampleServer._(this._server, this.records, this.errors);

  final ServerHandle _server;

  /// Every request the access log recorded, in order.
  final List<AccessRecord> records;

  /// Every error that escaped a handler.
  final List<Object> errors;

  /// Where the server is listening.
  String get origin => 'http://${_server.address.host}:${_server.port}';

  /// Requests still being handled.
  int get inFlight => _server.inFlight;

  /// The store the server is running against, so a test can seed it directly.
  late final TodoStore store;

  /// Starts the example with a fresh in-memory store.
  static Future<ExampleServer> start({TodoStore? store}) =>
      _start(store ?? (TodoRepository()..seed()));

  /// Starts the example against a SQLite database.
  ///
  /// `:memory:` gives each test its own, so the same suite runs against real
  /// SQL without a file to clean up or an ordering dependency between tests.
  static Future<ExampleServer> startWithSqlite() async {
    final store = SqliteTodoStore.open();
    await store.add(
      const CreateTodo(title: 'write the example', assignTo: owner),
    );
    return _start(store);
  }

  static Future<ExampleServer> _start(TodoStore store) async {
    final records = <AccessRecord>[];
    final errors = <Object>[];
    final app = buildApp(
      store,
      log: records.add,
      onError: (error, stack) => errors.add(error),
    );
    final server = await serveRouter(app, InternetAddress.loopbackIPv4, 0);
    return ExampleServer._(server, records, errors)..store = store;
  }

  /// Stops the server, draining what is in flight, and closes the store.
  Future<void> stop() async {
    await _server.close(drain: const Duration(seconds: 5));
    await store.close();
  }

  /// A `GET`, authenticated unless [token] is cleared.
  Future<http.Response> get(String path, {String? token = readToken}) =>
      http.get(Uri.parse('$origin$path'), headers: _headers(token));

  /// A `POST` carrying [body] as JSON.
  Future<http.Response> post(
    String path,
    Object? body, {
    String? token = writeToken,
    String contentType = 'application/json',
  }) {
    return http.post(
      Uri.parse('$origin$path'),
      headers: {..._headers(token), 'content-type': contentType},
      body: body is String ? body : jsonEncode(body),
    );
  }

  /// A `PATCH`.
  Future<http.Response> patch(String path, {String? token = writeToken}) =>
      http.patch(Uri.parse('$origin$path'), headers: _headers(token));

  /// A `DELETE`.
  Future<http.Response> delete(String path, {String? token = writeToken}) =>
      http.delete(Uri.parse('$origin$path'), headers: _headers(token));

  Map<String, String> _headers(String? token) =>
      token == null ? const {} : {'authorization': 'Bearer $token'};
}

/// The decoded JSON body of [response].
Object? bodyOf(http.Response response) => jsonDecode(response.body);

/// The decoded JSON body of [response], as an object.
Map<String, Object?> objectOf(http.Response response) =>
    bodyOf(response)! as Map<String, Object?>;

/// The `fields` map of a 422 body.
Map<String, Object?> fieldsOf(http.Response response) =>
    objectOf(response)['fields']! as Map<String, Object?>;

/// The identity the repository is seeded under.
const owner = 'ada@dust.test';

/// Someone else entirely.
const other = 'grace@dust.test';

/// A read-only token for [owner].
const readToken = '$owner|todos:read';

/// A read-write token for [owner].
const writeToken = '$owner|todos:read,todos:write';

/// A read-write token for [other], with no admin scope.
const otherToken = '$other|todos:read,todos:write';

/// A token that may act on anyone's todos.
const adminToken = 'root@dust.test|todos:read,todos:write,todos:admin';

/// A request body that passes every constraint.
Map<String, Object?> validBody({
  String title = 'buy milk',
  String assignTo = owner,
  bool done = false,
}) =>
    <String, Object?>{'title': title, 'assignTo': assignTo, 'done': done};
