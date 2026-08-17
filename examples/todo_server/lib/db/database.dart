import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

part 'database.g.dart';

/// The database itself: opening it, migrating it, closing it.
///
/// Deliberately separate from the queries in `todo_queries.dart`, because the
/// two have different lifetimes and different owners.
///
/// * A **database** is opened once by `main`, migrated on the way up, and
///   closed on the way down. Nothing but the composition site touches it.
/// * **Queries** are made per executor. `TodoDao(database.connection)` runs
///   against the connection; `TodoDao(tx)` inside `transaction` runs against
///   that transaction instead — the same queries, a different executor.
///
/// Folding them into one type would make that impossible to say: there would
/// be no way to hand a handler the queries without also handing it the power
/// to close the connection.
@SqlxDatabase(type: SqlxDatabaseType.sqlite, migrations: './migrations')
abstract class TodoDatabase implements DatabaseClient {
  /// Opens the database at [path], applying any unapplied migrations.
  ///
  /// `:memory:` gives each test its own with no file to clean up; a deployment
  /// passes a path and runs the same code.
  factory TodoDatabase.open(String path, {SqliteConnectOptions? options}) =
      _$TodoDatabase.open;

  /// The open connection every query runs against.
  @override
  DatabaseConnection get connection;
}
