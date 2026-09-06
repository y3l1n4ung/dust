import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

import 'expect_ok.dart';

/// Runs an administrative query and returns its rows, failing on an error.
///
/// Tests inspecting `PRAGMA` output or the migration table are the escape
/// hatch's real use case, and going through it here keeps them honest about
/// which path they are exercising.
Future<List<Row>> unsafeRows(
  Sqlite3Driver db,
  String sql, [
  List<Object?> parameters = const <Object?>[],
]) async {
  return expectOk(await Sqlite3UnsafeSql(db).fetch(sql, parameters));
}
