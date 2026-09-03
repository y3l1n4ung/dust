import 'dart:io';

import 'package:dust_dart/db.dart';
import 'package:dust_server/server.dart';
import 'package:dust_server/testing.dart';
import 'package:server_app/server_app.dart';
import 'package:test/test.dart';

/// The application on a loopback port, over an in-memory database.
final class TestApp {
  TestApp._(this.server, this.database, this.accounts, this.inventory,
      this.client, this._directory);

  /// Opens a fresh database, migrates it, and serves the routes.
  ///
  /// A **file**, not `:memory:`. SQLite ignores `journal_mode = WAL` for an
  /// in-memory database, so the options the application ships with — WAL, a
  /// busy timeout, foreign keys — would not be the ones under test, and the
  /// concurrency tests would prove less than they appear to.
  ///
  /// One directory per test, deleted afterwards, so nothing is shared between
  /// them and nothing survives the run.
  static Future<TestApp> start() async {
    final directory = Directory.systemTemp.createTempSync('server_app_test_');
    final database = AppDatabase.open(
      '${directory.path}/app.db',
      options: appOptions,
    );
    // The same composition the entry point uses. A test that built its own
    // router would pass while `bin/server.dart` was broken.
    final server = await serve(
      buildApp(database),
      InternetAddress.loopbackIPv4,
      0,
    );
    final origin = 'http://${server.address.host}:${server.port}';

    // Reaching past HTTP to seed. These wrap the connection rather than owning
    // it, so building a second set costs nothing and closes nothing.
    return TestApp._(
      server,
      database,
      AccountsRepo(database.connection),
      InventoryRepo(database.connection),
      TestClient.origin(origin),
      directory,
    );
  }

  /// The running server.
  final ServerHandle server;

  /// The open database.
  final AppDatabase database;

  /// The accounts repository, for reaching past HTTP when a test needs to.
  final AccountsRepo accounts;

  /// The inventory repository, for seeding stock.
  final InventoryRepo inventory;

  /// HTTP client pointed at this server.
  final TestClient client;

  final Directory _directory;

  /// Where the database file lives, for a test that wants to look.
  String get databasePath => '${_directory.path}/app.db';

  /// Creates an account with [password] hashed the way sign-in expects.
  Future<int> createAccount(
    String email,
    String password, {
    String scopes = 'orders:write',
  }) async {
    final salt = Passwords.newSalt();
    final result = await accounts.insertAccount(
      email,
      await Passwords.hash(password, salt),
      salt,
      scopes,
    );

    return (result as Ok<ExecResult, SqlxError>).value.lastInsertId!;
  }

  /// Signs in and returns the token, or throws with the status if it failed.
  Future<String> signIn(String email, String password) async {
    final response = await (client.post('/auth/tokens')
            ..json({'email': email, 'password': password}))
        .send();
    if (response.statusCode != 201) {
      throw StateError(
          'sign-in failed: ${response.statusCode} ${response.body}');
    }

    return (response.json as Map<String, dynamic>)['token'] as String;
  }

  /// Stops the server, closes the client, the database, and removes the file.
  ///
  /// The `-wal` and `-shm` files go with it, which is why the whole directory
  /// is removed rather than the one path.
  Future<void> stop() async {
    await client.close();
    await server.close(drain: const Duration(seconds: 1));
    await database.connection.close();
    if (_directory.existsSync()) _directory.deleteSync(recursive: true);
  }
}

/// Starts an application and stops it after the test.
Future<TestApp> testApp() async {
  final app = await TestApp.start();
  addTearDown(app.stop);

  return app;
}
