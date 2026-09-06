import 'dart:convert';
import 'dart:typed_data';

import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Storing bytes.
///
/// A `Uint8List` binds as `bytea` and reads back as one. Postgres transfers it
/// in the binary protocol, so there is no hex encoding to undo on either side.
///
/// Size is the thing to think about: a `bytea` is read whole, into the server's
/// memory and then into yours, with no way to stream a slice. Rows of a few
/// kilobytes are fine; a file is better in object storage with its key in the
/// column.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/bytea.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_blobs', const []);
    await db.unsafe.execute(
      '''
CREATE TABLE example_blobs (
  id      BIGSERIAL PRIMARY KEY,
  name    TEXT NOT NULL,
  content BYTEA NOT NULL
)''',
      const [],
    );

    final content = Uint8List.fromList(utf8.encode('hello, bytea'));
    await db.execute(
      r'INSERT INTO example_blobs (name, content) VALUES ($1, $2)',
      <Object?>['greeting.txt', content],
    );

    final stored = await db.fetchOne<Uint8List>(
      r'SELECT content FROM example_blobs WHERE name = $1',
      const <Object?>['greeting.txt'],
      (row) => row.read<Uint8List>('content'),
    );
    final bytes = stored.unwrapOrElse((error) => throw error);
    print('bytes: ${bytes.length}');
    print('decoded: ${utf8.decode(bytes)}');

    // `octet_length` measures it without transferring it, which is what a
    // listing wants.
    final size = await db.fetchScalar<int>(
      r'SELECT octet_length(content) FROM example_blobs WHERE name = $1',
      const <Object?>['greeting.txt'],
    );
    print('size without reading: ${size.unwrapOrElse((_) => -1)}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_blobs', const []);
    await db.close();
  }
}
