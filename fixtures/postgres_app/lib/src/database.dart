import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';

part 'database.g.dart';

/// The fixture's database.
///
/// `connect(String url)` rather than SQLite's `open(String path)`: the facade
/// signature follows the database, and PostgreSQL is reached over a network.
@SqlxDatabase(type: SqlxDatabaseType.postgres)
abstract class AppDatabase implements DatabaseClient {
  /// Connects to [url] and prepares the generated queries.
  factory AppDatabase.connect(String url, {PgConnectOptions? options}) =
      _$AppDatabase.connect;

  @override
  Connection get connection;
}
