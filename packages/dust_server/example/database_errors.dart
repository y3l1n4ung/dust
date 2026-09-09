import 'dart:io';

import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:dust_server/server.dart';

/// Turning a database failure into the right status code.
///
/// A query returns `Result<T, SqlxError>` rather than throwing, so the failure
/// arrives as a value with a `category` on it. Mapping that category is what
/// separates "this request was wrong" from "the database is down" — and
/// answering 500 for both is how a duplicate email pages somebody at 3am.
///
/// Three cases worth telling apart:
///
/// * **A row that is not there** is `cardinality`, and a 404. `fetchOne` says
///   so; `fetchOptional` would hand back `null` instead and let you decide.
/// * **A constraint the database refused** is `query`, and usually a 409. The
///   specific constraint is in `cause`, because which constraints exist is the
///   schema's business rather than something a portable category can name.
/// * **Everything else** is a 500, and the message goes to the log rather than
///   to the client.
///
/// ```shell
/// dart run example/database_errors.dart
///
/// curl localhost:8080/users/1          # 200
/// curl localhost:8080/users/404        # 404, not a 500
/// curl -X POST localhost:8080/users -H 'content-type: application/json' \
///   -d '{"email":"ada@example.com"}'   # 409 the second time
/// ```
Future<void> main() async {
  final database = openDatabase();

  final server = await serve(buildApp(database), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await database.close();
  await server.close(drain: const Duration(seconds: 5));
}

/// Opens the database, with one user already registered.
Sqlite3Driver openDatabase() {
  final database = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const <String, String>{
      '0001_create_users.sql': '''
CREATE TABLE users (
  id    INTEGER PRIMARY KEY,
  email TEXT NOT NULL UNIQUE
);
''',
    },
  );
  database.execute(
    r'INSERT INTO users (email) VALUES ($1)',
    const <Object?>['ada@example.com'],
  );
  return database;
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp(Sqlite3Driver database) {
  return Router()
    ..route('/users', post(register, status: 201))
    ..route('/users/{id}', get(readUser))
    ..withState(database);
}

/// `GET /users/{id}` — 404 when there is no such row, not 500.
Future<Result<Map<String, Object?>, Rejection>> readUser(
  Request request,
) async {
  final database = await request.state<Sqlite3Driver>();
  final id = await request.path<int>('id');

  final user = await database.fetchOne<Map<String, Object?>>(
    r'SELECT id, email FROM users WHERE id = $1',
    <Object?>[id],
    (row) => <String, Object?>{
      'id': row.read<int>('id'),
      'email': row.read<String>('email'),
    },
  );

  return switch (user) {
    Ok(:final value) => Ok(value),
    Err(:final error) => Err(_statusFor(error, 'user $id')),
  };
}

/// `POST /users` — 409 when the email is taken.
///
/// Letting the insert fail rather than checking first: a `SELECT` before an
/// `INSERT` is a race, and the unique index has to do the work anyway.
Future<Result<Map<String, Object?>, Rejection>> register(
  Request request,
) async {
  final database = await request.state<Sqlite3Driver>();
  final body = await request.body<Map<String, Object?>>((json) => json);

  final created = await database.fetchOne<Map<String, Object?>>(
    r'INSERT INTO users (email) VALUES ($1) RETURNING id, email',
    <Object?>[body['email']],
    (row) => <String, Object?>{
      'id': row.read<int>('id'),
      'email': row.read<String>('email'),
    },
  );

  return switch (created) {
    Ok(:final value) => Ok(value),
    Err(:final error) => Err(_statusFor(error, 'that email')),
  };
}

/// One database failure as one status code.
Rejection _statusFor(SqlxError error, String subject) {
  // Message text belongs to the driver, so match on what it means. This check
  // is deliberately narrow: anything it does not recognise stays a 500.
  final duplicate = error.category == SqlxErrorCategory.query &&
      '${error.cause}'.contains('UNIQUE constraint failed');

  return switch (error.category) {
    SqlxErrorCategory.cardinality => Rejection.notFound('no such $subject'),
    SqlxErrorCategory.query when duplicate =>
      Rejection.conflict('$subject is already taken'),
    // The client cannot do anything about the rest, and should not be told
    // which of them it was.
    _ => const Rejection.internal(),
  };
}
