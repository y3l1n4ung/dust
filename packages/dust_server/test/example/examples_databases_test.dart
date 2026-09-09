import 'dart:convert';
import 'dart:io';
import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:test/test.dart';
import '../../example/database_errors.dart' as database_errors;
import '../../example/database_pagination.dart' as database_pagination;
import '../../example/database_transactions.dart' as database_transactions;
import '../../example/sqlite_database.dart' as sqlite_database;
import 'package:dust_db_postgres/dust_db_postgres.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import '../../example/postgres_database.dart' as postgres_database;
import 'serve.dart';

/// The database examples, against SQLite and PostgreSQL.
///
/// Split out of one suite per example so each file stays readable; they
/// share the harness in `support.dart`.
void main() {
  group('sqlite_database', () {
    test('serves rows and writes one back', () async {
      final directory = Directory.systemTemp.createTempSync('dust_notes_test');
      final database = sqlite_database.openDatabase(
        '${directory.path}/notes.db',
      );
      addTearDown(() async {
        await database.close();
        directory.deleteSync(recursive: true);
      });
      final app = await ExampleApp.start(sqlite_database.buildApp(database));

      expect(jsonDecode((await app.get('/notes')).body), isEmpty);

      final written = await app.post('/notes', const {'body': 'over HTTP'});
      expect(written.statusCode, 201);
      expect(app.object(written)['body'], 'over HTTP');

      final listed =
          jsonDecode((await app.get('/notes')).body) as List<Object?>;
      expect((listed.single! as Map<String, Object?>)['body'], 'over HTTP');
    });
  });

  group('database_transactions', () {
    late Sqlite3Driver database;
    late ExampleApp app;

    setUp(() async {
      database = database_transactions.openDatabase();
      addTearDown(database.close);
      app = await ExampleApp.start(database_transactions.buildApp(database));
    });

    test('a checkout that fits commits both writes', () async {
      final placed = await app.post(
        '/checkout',
        const {'item': 'shirt', 'quantity': 2},
      );

      expect(placed.statusCode, 201);
      expect(app.object(placed)['quantity'], 2);

      final stock = jsonDecode((await app.get('/stock')).body) as List<Object?>;
      expect((stock.single! as Map<String, Object?>)['onHand'], 3);
    });

    test('a checkout that does not fit reserves nothing', () async {
      final refused = await app.post(
        '/checkout',
        const {'item': 'shirt', 'quantity': 99},
      );

      // 409, because somebody buying the last one is an ordinary outcome.
      expect(refused.statusCode, 409);

      // The whole point: the reservation above it was rolled back.
      final stock = jsonDecode((await app.get('/stock')).body) as List<Object?>;
      expect((stock.single! as Map<String, Object?>)['onHand'], 5);
    });
  });

  group('database_errors', () {
    late ExampleApp app;

    setUp(() async {
      final database = database_errors.openDatabase();
      addTearDown(database.close);
      app = await ExampleApp.start(database_errors.buildApp(database));
    });

    test('a row that is there is a 200', () async {
      final response = await app.get('/users/1');

      expect(response.statusCode, 200);
      expect(app.object(response)['email'], 'ada@example.com');
    });

    test('a row that is not there is a 404, not a 500', () async {
      expect((await app.get('/users/404')).statusCode, 404);
    });

    test('a duplicate is a 409, not a 500', () async {
      expect(
        (await app.post('/users', const {'email': 'grace@example.com'}))
            .statusCode,
        201,
      );
      expect(
        (await app.post('/users', const {'email': 'grace@example.com'}))
            .statusCode,
        409,
      );
    });
  });

  group('database_pagination', () {
    late ExampleApp app;

    setUp(() async {
      final database = database_pagination.openDatabase();
      addTearDown(database.close);
      app = await ExampleApp.start(database_pagination.buildApp(database));
    });

    test('limit and offset page through the rows', () async {
      final first =
          jsonDecode((await app.get('/notes?limit=2')).body) as List<Object?>;
      final second = jsonDecode((await app.get('/notes?limit=2&offset=2')).body)
          as List<Object?>;

      expect(first.length, 2);
      expect(second.length, 2);
      expect(first.first, isNot(second.first));
    });

    test('sorting picks a constant statement, not a concatenated one',
        () async {
      final byBody =
          jsonDecode((await app.get('/notes?sort=body')).body) as List<Object?>;

      expect(
        [for (final row in byBody) (row! as Map<String, Object?>)['body']],
        <String>['alpha', 'bravo', 'charlie', 'delta'],
      );
    });

    test('a sort column nothing offers is a 400', () async {
      // The reason the switch exists: this reaches no SQL at all.
      expect(
        (await app.get('/notes?sort=;%20DROP%20TABLE%20notes')).statusCode,
        400,
      );
    });

    test('an oversized limit is clamped rather than trusted', () async {
      final all = jsonDecode((await app.get('/notes?limit=1000000')).body)
          as List<Object?>;

      expect(all.length, 4);
    });
  });

  group('postgres_database', () {
    // The only example that needs something the suite cannot bring: there is
    // no in-memory PostgreSQL. Skipped, and reported as skipped, rather than
    // failing a checkout that has no server.
    final url = Platform.environment['DUST_DATABASE_URL'];

    test('serves rows out of PostgreSQL and writes one back', () async {
      final database = PostgresDriver.connect(
        url!,
        migrations: const <String, String>{
          '0001_create_notes.sql': 'CREATE TABLE IF NOT EXISTS example_notes ('
              'id BIGSERIAL PRIMARY KEY, body TEXT NOT NULL);',
        },
      );
      addTearDown(() async {
        await database.unsafe.execute(
          'DROP TABLE IF EXISTS example_notes',
          const [],
        );
        await database.unsafe.execute(
          r'DELETE FROM __dust_schema_migrations WHERE name = $1',
          const <Object?>['0001_create_notes.sql'],
        );
        await database.close();
      });
      expect((await database.migrate()).isOk, isTrue);
      await database.unsafe.execute('DELETE FROM example_notes', const []);

      final app = await ExampleApp.start(
        postgres_database.buildApp(database),
      );

      expect(jsonDecode((await app.get('/notes')).body), isEmpty);

      final written = await app.post('/notes', const {'body': 'over HTTP'});
      expect(written.statusCode, 201);
      expect(app.object(written)['body'], 'over HTTP');

      final listed =
          jsonDecode((await app.get('/notes')).body) as List<Object?>;
      expect(listed.length, 1);
      expect((listed.single! as Map<String, Object?>)['body'], 'over HTTP');
    }, skip: url == null ? 'set DUST_DATABASE_URL to run' : null);
  });
}

/// The `seen` count out of a `/whoami` body.
int app0(http.Response response) =>
    jsonDecode(response.body)['seen'] as int? ?? -1;

/// Keeps every span so a test can look at one.
final class CollectingExporter implements SpanExporter {
  /// What has been exported.
  final spans = <Span>[];

  @override
  void export(Span span) => spans.add(span);
}
