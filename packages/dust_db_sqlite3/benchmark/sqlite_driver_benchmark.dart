import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

final class BenchUser {
  const BenchUser(this.id, this.name);

  final int id;
  final String name;

  static BenchUser fromRow(Row row) {
    return BenchUser(row.read<int>('id'), row.read<String>('name'));
  }
}

Future<void> main() async {
  const rows = 1000;
  const reads = 200;
  final db = Sqlite3Driver.open(
    ':memory:',
    migrations: const {
      '0001.sql':
          'CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);',
    },
  );
  for (var i = 0; i < rows; i += 1) {
    await db.execute(r'INSERT INTO users (id, name) VALUES (?, ?)', [
      i,
      'user-$i',
    ]);
  }

  final watch = Stopwatch()..start();
  var decoded = 0;
  for (var i = 0; i < reads; i += 1) {
    final result = await db.fetchAll<BenchUser>(
      'SELECT id, name FROM users ORDER BY id',
      const [],
      BenchUser.fromRow,
    );
    decoded += result.match(ok: (rows) => rows.length, err: (_) => 0);
  }
  watch.stop();
  await db.close();
  print(
    'sqlite_fetch_all reads=$reads decoded=$decoded elapsed_us=${watch.elapsedMicroseconds}',
  );
}
