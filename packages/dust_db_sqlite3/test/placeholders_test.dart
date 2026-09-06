import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:dust_db_sqlite3/src/placeholders.dart';
import 'package:test/test.dart';

import 'support/expect_ok.dart';
import 'support/unsafe_rows.dart';

/// The scanner ported from `dust_db_plugin`'s `sql.rs`, case for case.
///
/// Build time and run time must agree about which `$n` is a parameter: the
/// plugin counts them to check arity, and the driver rewrites them to bind. A
/// disagreement means a query that validates and then binds the wrong argument,
/// so these mirror the Rust tests deliberately.

SqlitePool _pool() {
  return SqlitePool.open(
    ':memory:',
    migrations: const <String, String>{
      '0001.sql': '''
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  org_id INTEGER NOT NULL,
  name TEXT NOT NULL
);
''',
    },
  );
}

void main() {
  late SqlitePool pool;

  setUp(() async {
    pool = _pool();
    for (final row in const <List<Object?>>[
      [1, 10, 'Ada'],
      [2, 10, 'Grace'],
      [3, 20, 'Kay'],
    ]) {
      expectOk(
        await queryExecute(
          r'INSERT INTO users (id, org_id, name) VALUES ($1, $2, $3)',
          row,
        ).execute(pool),
      );
    }
  });

  tearDown(() async {
    await pool.close();
  });

  /// Reads the `name` column of every row a statement returns.
  Future<List<String>> names(String sql, List<Object?> parameters) async {
    final rows = await unsafeRows(pool, sql, parameters);
    return rows.map((row) => row.read<String>('name')).toList();
  }

  test('a repeated placeholder binds its argument twice', () async {
    expect(
      await names(
        r"SELECT '$1' AS label, name FROM users "
        r'WHERE id = $1 OR org_id = $1 ORDER BY id',
        [10],
      ),
      // Only org 10 matches, and the literal `'$1'` is text, not a parameter.
      <String>['Ada', 'Grace'],
    );
  });

  test('placeholders bind in the order the statement reads them', () async {
    expect(
      await names(
        r'SELECT name FROM users WHERE org_id = $2 AND id = $1',
        [1, 10],
      ),
      <String>['Ada'],
    );
  });

  test('a line comment holds no placeholders', () async {
    expect(
      await names(
        '-- filter by \$9 owner\n'
        r'SELECT name FROM users WHERE id = $1',
        [2],
      ),
      <String>['Grace'],
    );
  });

  test('a block comment holds no placeholders', () async {
    expect(
      await names(
        r'SELECT name FROM users WHERE id = $1 /* owner is $2 */ AND org_id = $2',
        [1, 10],
      ),
      <String>['Ada'],
    );
  });

  test('block comments do not nest', () async {
    // SQLite ends the comment at the first `*/`, so a `$1` after it is a real
    // parameter however many `/*` came before.
    expect(
      await names(r'SELECT /* a /* b $9 */ name FROM users WHERE id = $1', [3]),
      <String>['Kay'],
    );
  });

  test('comment openers inside string literals are text', () async {
    expect(
      await names(
        r"SELECT '-- not a comment' AS a, '/*' AS b, name "
        r'FROM users WHERE id = $1',
        [1],
      ),
      <String>['Ada'],
    );
  });

  test('doubled quotes escape rather than close', () async {
    expect(
      await names(
        'SELECT \'it\'\'s \$9\' AS a, "od""d \$9" AS b, name '
        r'FROM users WHERE id = $1',
        [3],
      ),
      <String>['Kay'],
    );
  });

  test('native ? placeholders still work untouched', () async {
    expect(
      await names('SELECT name FROM users WHERE id = ?', [2]),
      <String>['Grace'],
    );
  });

  // Dollar quoting is Postgres syntax that SQLite rejects outright, so these
  // two cannot be proven by running a statement. They still have to hold: a
  // migration carrying a `CREATE FUNCTION` body reaches this scanner, and the
  // count it produces has to match what the plugin counted at build time.
  test('a dollar-quoted body holds no placeholders', () {
    final rewrite = rewritePlaceholders(
      r'SELECT $$ raw $1 body $$, $tag$ also $2 $tag$ FROM t WHERE id = $1',
    );

    expect(
      rewrite.sql,
      r'SELECT $$ raw $1 body $$, $tag$ also $2 $tag$ FROM t WHERE id = ?',
    );
    expect(rewrite.parameterOrder, <int>[1]);
  });

  test('an unclosed dollar tag does not swallow the statement', () {
    final rewrite =
        rewritePlaceholders(r'SELECT $tag$ id FROM t WHERE id = $1');

    expect(rewrite.sql, r'SELECT $tag$ id FROM t WHERE id = ?');
    expect(rewrite.parameterOrder, <int>[1]);
  });

  test('a zero placeholder is not one', () {
    // `$0` is not a placeholder any dialect binds; the plugin rejects it at
    // build time and the driver must not read past the front of the list.
    final rewrite = rewritePlaceholders(r'SELECT id FROM t WHERE id = $0');

    expect(rewrite.sql, r'SELECT id FROM t WHERE id = $0');
    expect(rewrite.parameterOrder, isEmpty);
  });

  test('a placeholder past the end of the arguments reports itself', () async {
    final result = await queryExecute(
      r'DELETE FROM users WHERE id = $3',
      [1],
    ).execute(pool);

    expect(result.isErr, isTrue);
    expect(
      result.match(ok: (_) => '', err: (error) => error.message),
      contains(r'binds $3 but only 1 argument was supplied'),
    );
  });
}
