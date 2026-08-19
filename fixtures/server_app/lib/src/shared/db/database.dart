import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

part 'database.g.dart';

/// The application's database: opening it, migrating it, closing it.
///
/// Separate from the queries because the two have different owners. This is
/// opened once by `main`; a DAO is what a handler gets, and a handler has no
/// business closing a connection.
@SqlxDatabase(type: SqlxDatabaseType.sqlite, migrations: './migrations')
abstract class AppDatabase implements DatabaseClient {
  /// Opens the database at [path], applying any unapplied migrations.
  factory AppDatabase.open(String path, {SqliteConnectOptions? options}) =
      _$AppDatabase.open;

  /// The open connection.
  @override
  DatabaseConnection get connection;
}

/// What a file-backed database wants when more than one isolate has it open.
///
/// WAL lets readers run while a writer holds the file, which is the difference
/// between a shared SQLite database that serves requests and one that returns
/// `SQLITE_BUSY` under load. Foreign keys are on because the schema declares
/// them and SQLite ignores them unless asked.
SqliteConnectOptions get appOptions => const SqliteConnectOptions(
      journalMode: SqliteJournalMode.wal,
      busyTimeout: Duration(seconds: 5),
      foreignKeys: true,
    );
