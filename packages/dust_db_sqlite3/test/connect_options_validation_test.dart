import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:test/test.dart';

/// What `SqliteConnectOptions` refuses, and why each refusal exists.
///
/// These are all reported before a file is touched: a connection that would
/// behave differently from what was asked for is worth stopping at the call
/// rather than discovering later.

void main() {
  group('validation', () {
    test('a path is required unless the database is in memory', () {
      expect(
        () => Sqlite3Driver.connect(const SqliteConnectOptions()),
        throwsA(isA<SqlxError>()),
      );
    });

    test('options carrying another path do not override open()', () {
      // `open(path)` and `SqliteConnectOptions.path(other)` disagreeing is a
      // mistake rather than a precedence question, so neither wins.
      expect(
        () => Sqlite3Driver.open(
          'one.db',
          options: const SqliteConnectOptions.path('another.db'),
        ),
        throwsA(isA<SqlxError>()),
      );
    });

    test('a read-only connection cannot create the database', () {
      expect(
        () => Sqlite3Driver.connect(
          const SqliteConnectOptions.path(
            'missing.db',
            readOnly: true,
            createIfMissing: true,
          ),
        ),
        throwsA(isA<SqlxError>()),
      );
    });

    test('a read-only connection cannot run migrations', () {
      expect(
        () => Sqlite3Driver.connect(
          const SqliteConnectOptions.path('read.db', readOnly: true),
          migrations: const <String, String>{'0001.sql': 'SELECT 1;'},
        ),
        throwsA(isA<SqlxError>()),
      );
    });

    test('a negative busy timeout is refused', () {
      expect(
        () => Sqlite3Driver.connect(
          const SqliteConnectOptions.memory(
            busyTimeout: Duration(milliseconds: -1),
          ),
        ),
        throwsA(isA<SqlxError>()),
      );
    });

    test('a pragma name that is not an identifier is refused', () {
      // A pragma name is interpolated into SQL, so anything that is not a bare
      // identifier is refused rather than escaped.
      expect(
        () => Sqlite3Driver.connect(
          const SqliteConnectOptions.memory(
            pragmas: <String, Object>{'foo; DROP TABLE users': 1},
          ),
        ),
        throwsA(isA<SqlxError>()),
      );
    });

    test('a pragma value that is not a scalar is refused', () {
      expect(
        () => Sqlite3Driver.connect(
          const SqliteConnectOptions.memory(
            pragmas: <String, Object>{
              'cache_size': <int>[1]
            },
          ),
        ),
        throwsA(isA<SqlxError>()),
      );
    });

    test('an infinite pragma value is refused', () {
      expect(
        () => Sqlite3Driver.connect(
          const SqliteConnectOptions.memory(
            pragmas: <String, Object>{'cache_size': double.infinity},
          ),
        ),
        throwsA(isA<SqlxError>()),
      );
    });
  });

  group('applied settings', () {
    test('every journal mode opens', () async {
      // WAL is meaningless for an in-memory database, which is exactly why the
      // application under test uses a file; the modes still have to render.
      for (final mode in SqliteJournalMode.values) {
        final pool = SqlitePool.connect(
          SqliteConnectOptions.memory(journalMode: mode),
        );
        addTearDown(() async {
          await pool.close();
        });
        expect(pool.driver, Driver.sqlite3, reason: '$mode');
      }
    });

    test('every synchronous mode opens', () async {
      for (final mode in SqliteSynchronousMode.values) {
        final pool = SqlitePool.connect(
          SqliteConnectOptions.memory(synchronous: mode),
        );
        addTearDown(() async {
          await pool.close();
        });
        expect(pool.driver, Driver.sqlite3, reason: '$mode');
      }
    });

    test('pragma values of every scalar shape are rendered', () async {
      final pool = SqlitePool.connect(
        const SqliteConnectOptions.memory(
          pragmas: <String, Object>{
            'foreign_keys': true,
            'cache_size': -2000,
            'locking_mode': 'NORMAL',
          },
        ),
      );
      addTearDown(() async {
        await pool.close();
      });

      expect(pool.driver, Driver.sqlite3);
    });
  });
}
