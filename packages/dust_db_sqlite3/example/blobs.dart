import 'dart:convert';
import 'dart:typed_data';

import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Storing bytes.
///
/// A `Uint8List` binds as a BLOB and reads back as one. It is the single
/// exception to `List` binding as a JSON set — a `List<int>` is ambiguous, a
/// `Uint8List` is not, which is why the byte case has its own type rather than
/// a flag.
///
/// ```shell
/// dart run example/blobs.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': '''
CREATE TABLE attachments (
  id      INTEGER PRIMARY KEY,
  name    TEXT NOT NULL,
  content BLOB NOT NULL
);
''',
    },
  );

  try {
    final content = Uint8List.fromList(utf8.encode('hello, blob'));
    await db.execute(
      r'INSERT INTO attachments (name, content) VALUES ($1, $2)',
      <Object?>['greeting.txt', content],
    );

    final stored = await db.fetchOne<Uint8List>(
      r'SELECT content FROM attachments WHERE name = $1',
      const <Object?>['greeting.txt'],
      (row) => row.read<Uint8List>('content'),
    );
    final bytes = stored.unwrapOrElse((error) => throw error);
    print('bytes: ${bytes.length}');
    print('decoded: ${utf8.decode(bytes)}');
  } finally {
    await db.close();
  }
}
