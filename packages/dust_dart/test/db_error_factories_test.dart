import 'package:dust_dart/db.dart';
import 'package:test/test.dart';

/// The `SqlxError` factories, which drivers construct and callers match on.
///
/// Each one exists so a failure arrives with a category rather than as a bare
/// message, and every driver reaches for a different subset — so the ones this
/// package's own tests do not happen to use still have to hold.

void main() {
  test('each factory carries its category, driver and operation', () {
    final errors = <SqlxErrorCategory, SqlxError>{
      SqlxErrorCategory.driver: SqlxError.driver(
        'boom',
        driver: Driver.postgres,
        operation: 'op',
      ),
      SqlxErrorCategory.connection: SqlxError.connection(
        'boom',
        driver: Driver.postgres,
        operation: 'op',
      ),
      SqlxErrorCategory.migration: SqlxError.migration(
        'boom',
        driver: Driver.postgres,
        operation: 'op',
      ),
      SqlxErrorCategory.query: SqlxError.query(
        'boom',
        driver: Driver.postgres,
        operation: 'op',
      ),
      SqlxErrorCategory.transaction: SqlxError.transaction(
        'boom',
        driver: Driver.postgres,
        operation: 'op',
      ),
      SqlxErrorCategory.decode: SqlxError.decode(
        'boom',
        driver: Driver.postgres,
        operation: 'op',
      ),
    };

    for (final entry in errors.entries) {
      expect(entry.value.category, entry.key, reason: '${entry.key}');
      expect(entry.value.driver, Driver.postgres, reason: '${entry.key}');
      expect(entry.value.operation, 'op', reason: '${entry.key}');
      expect(entry.value.message, contains('boom'), reason: '${entry.key}');
    }
  });

  test('a cause is carried through', () {
    final cause = StateError('underlying');
    final error = SqlxError.query('boom', cause: cause);

    expect(error.cause, same(cause));
  });

  group('cardinality', () {
    test('no rows reads as none where one was wanted', () {
      final error = SqlxError.noRows('SELECT 1', driver: Driver.sqlite3);

      expect(error.category, SqlxErrorCategory.cardinality);
      expect(error.cause, isNull);
      expect(error.toString(), contains('SELECT 1'));
    });

    test('too many rows names what came back', () {
      final error = SqlxError.tooManyRows(
        expected: 1,
        actual: 3,
        query: 'SELECT 1',
      );

      expect(error.category, SqlxErrorCategory.cardinality);
      expect(error.message, contains('3'));
    });

    test('an empty query name still reads', () {
      // A generated call has a name; an inline one may not, and the message has
      // to make sense either way.
      final error = SqlxError.tooManyRows(expected: 1, actual: 2, query: '');

      expect(error.toString(), startsWith('SQL query expected'));
    });
  });

  test('a null column names the column', () {
    final error = SqlxError.nullColumn('email', driver: Driver.sqlite3);

    expect(error.category, SqlxErrorCategory.decode);
    expect(error.message, contains('email'));
  });
}
