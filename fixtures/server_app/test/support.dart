import 'dart:convert';
import 'dart:io';

import 'package:dust_dart/db.dart';
import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:server_app/server_app.dart';
import 'package:test/test.dart';

/// The application on a loopback port, over an in-memory database.
final class TestApp {
  TestApp._(this.server, this.database, this.queries);

  /// Opens a fresh database, migrates it, and serves the routes.
  static Future<TestApp> start() async {
    final database = AppDatabase.open(':memory:', options: appOptions);
    final queries = AppQueries(database.connection);

    final app = Router()
      ..nest('/auth', authRoutes())
      ..nest('/orders', orderRoutes())
      ..nest('/admin', adminRoutes())
      ..withState(queries);

    final server = await serveRouter(app, InternetAddress.loopbackIPv4, 0);
    return TestApp._(server, database, queries);
  }

  /// The running server.
  final ServerHandle server;

  /// The open database.
  final AppDatabase database;

  /// The queries, for reaching past HTTP when a test needs to.
  final AppQueries queries;

  /// Where to point a client.
  String get origin => 'http://${server.address.host}:${server.port}';

  /// Creates an account with [password] hashed the way sign-in expects.
  Future<int> createAccount(
    String email,
    String password, {
    String scopes = 'orders:write',
  }) async {
    final salt = Passwords.newSalt();
    final result = await queries.insertAccount(
      email,
      Passwords.hash(password, salt),
      salt,
      scopes,
    );

    return (result as Ok<ExecResult, SqlxError>).value.lastInsertId!;
  }

  /// Signs in and returns the token, or throws with the status if it failed.
  Future<String> signIn(String email, String password) async {
    final response = await send(
      'POST',
      '/auth/tokens',
      body: {'email': email, 'password': password},
    );
    if (response.statusCode != 201) {
      throw StateError(
          'sign-in failed: ${response.statusCode} ${response.body}');
    }

    return jsonDecode(response.body)['token'] as String;
  }

  /// Sends a request.
  Future<http.Response> send(
    String method,
    String path, {
    Object? body,
    String? token,
  }) {
    final uri = Uri.parse('$origin$path');
    final headers = {
      if (body != null) 'content-type': 'application/json',
      if (token != null) 'authorization': 'Bearer $token',
    };
    final encoded =
        body is String ? body : (body == null ? null : jsonEncode(body));

    return switch (method) {
      'GET' => http.get(uri, headers: headers),
      'POST' => http.post(uri, headers: headers, body: encoded),
      'DELETE' => http.delete(uri, headers: headers),
      _ => throw ArgumentError(method),
    };
  }

  /// Stops the server and closes the database.
  Future<void> stop() async {
    await server.close(drain: const Duration(seconds: 1));
    await database.connection.close();
  }
}

/// Starts an application and stops it after the test.
Future<TestApp> testApp() async {
  final app = await TestApp.start();
  addTearDown(app.stop);

  return app;
}
