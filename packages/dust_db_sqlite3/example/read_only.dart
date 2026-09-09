import 'dart:io';

import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// A connection that cannot write, by construction.
///
/// [SqliteConnectOptions.readOnly] opens the file read-only at the operating
/// system level, so a write fails in SQLite rather than depending on the
/// application never issuing one. It also refuses migrations up front: a
/// read-only connection cannot apply them, and silently skipping them would
/// leave the caller believing the schema is current.
///
/// ```shell
/// dart run example/read_only.dart
/// ```
Future<void> main() async {
  final directory = Directory.systemTemp.createTempSync('dust_sqlite');
  final path = '${directory.path}/app.db';

  final writer = Sqlite3Driver.open(
    path,
    migrations: const {
      '0001_create_tenants.sql': 'CREATE TABLE tenants (name TEXT NOT NULL);',
    },
  );
  await writer.execute(
    r'INSERT INTO tenants (name) VALUES ($1)',
    const <Object?>['acme'],
  );
  await writer.close();

  final reader = Sqlite3Driver.connect(SqliteConnectOptions.readOnly(path));
  try {
    final names = await reader.fetchAll<String>(
      'SELECT name FROM tenants',
      const [],
      (row) => row.read<String>('name'),
    );
    print('reads: ${names.unwrapOrElse((_) => const <String>[])}');

    final refused = await reader.execute(
      r'INSERT INTO tenants (name) VALUES ($1)',
      const <Object?>['nope'],
    );
    print('write refused: ${refused.isErr}');
  } finally {
    await reader.close();
    directory.deleteSync(recursive: true);
  }
}
