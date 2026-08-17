import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

part 'database.g.dart';

/// The database: opening it, migrating it, closing it.
///
/// Separate from the queries because the two have different owners. The
/// database is opened once by `main`; the queries are what a handler gets, and
/// a handler has no business closing a connection.
@SqlxDatabase(type: SqlxDatabaseType.sqlite, migrations: './migrations')
abstract class NotesDatabase implements DatabaseClient {
  /// Opens the database at [path], migrating it.
  factory NotesDatabase.open(String path, {SqliteConnectOptions? options}) =
      _$NotesDatabase.open;

  /// The open connection.
  @override
  DatabaseConnection get connection;
}
