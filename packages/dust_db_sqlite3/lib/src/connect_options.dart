part of 'sqlite_pool.dart';

final _sqlitePragmaName = RegExp(r'^[A-Za-z_][A-Za-z0-9_]*$');

/// SQLite journal mode applied with `PRAGMA journal_mode`.
enum SqliteJournalMode {
  /// Default rollback journal mode.
  delete,

  /// Truncate the rollback journal instead of deleting it.
  truncate,

  /// Keep the rollback journal file after transactions.
  persist,

  /// Store the rollback journal in memory.
  memory,

  /// Use write-ahead logging.
  wal,

  /// Disable rollback journal files.
  off,
}

/// SQLite synchronous mode applied with `PRAGMA synchronous`.
enum SqliteSynchronousMode {
  /// Disable extra sync operations.
  off,

  /// Sync at normal SQLite durability level.
  normal,

  /// Sync at full SQLite durability level.
  full,

  /// Sync with SQLite extra durability checks.
  extra,
}

/// Production SQLite connection options for Dust databases.
final class SqliteConnectOptions {
  /// Creates reusable options for [Sqlite3Driver.open].
  const SqliteConnectOptions({
    this.createIfMissing = true,
    this.readOnly = false,
    this.busyTimeout,
    this.foreignKeys,
    this.journalMode,
    this.synchronous,
    this.pragmas = const <String, Object>{},
  })  : path = null,
        inMemory = false;

  /// Opens a database file at [path].
  const SqliteConnectOptions.path(
    this.path, {
    this.createIfMissing = true,
    this.readOnly = false,
    this.busyTimeout,
    this.foreignKeys,
    this.journalMode,
    this.synchronous,
    this.pragmas = const <String, Object>{},
  }) : inMemory = false;

  /// Opens an existing database file at [path] in read-only mode.
  const SqliteConnectOptions.readOnly(
    this.path, {
    this.busyTimeout,
    this.foreignKeys,
    this.journalMode,
    this.synchronous,
    this.pragmas = const <String, Object>{},
  })  : createIfMissing = false,
        readOnly = true,
        inMemory = false;

  /// Opens an in-memory SQLite database.
  const SqliteConnectOptions.memory({
    this.busyTimeout,
    this.foreignKeys,
    this.journalMode,
    this.synchronous,
    this.pragmas = const <String, Object>{},
  })  : path = null,
        createIfMissing = true,
        readOnly = false,
        inMemory = true;

  /// Database path when this is a path-based connection.
  final String? path;

  /// Whether this connection uses SQLite's in-memory database.
  final bool inMemory;

  /// Creates a missing path-based database file.
  final bool createIfMissing;

  /// Opens a path-based database in read-only mode.
  final bool readOnly;

  /// SQLite busy timeout for lock waits.
  final Duration? busyTimeout;

  /// Enables or disables SQLite foreign key enforcement.
  final bool? foreignKeys;

  /// SQLite rollback journal or WAL mode.
  final SqliteJournalMode? journalMode;

  /// SQLite disk sync behavior.
  final SqliteSynchronousMode? synchronous;

  /// Extra SQLite pragmas applied after the built-in options.
  final Map<String, Object> pragmas;

  SqliteConnectOptions _withPath(String fallbackPath) {
    if (inMemory) return this;
    final currentPath = path;
    if (currentPath != null) {
      if (currentPath != fallbackPath) {
        throw SqlxError.driver(
          'SQLite connect options path `$currentPath` does not match '
          'open path `$fallbackPath`.',
        );
      }
      return this;
    }
    return SqliteConnectOptions.path(
      fallbackPath,
      createIfMissing: createIfMissing,
      readOnly: readOnly,
      busyTimeout: busyTimeout,
      foreignKeys: foreignKeys,
      journalMode: journalMode,
      synchronous: synchronous,
      pragmas: pragmas,
    );
  }

  void _validate({required bool hasMigrations}) {
    if (!inMemory && path == null) {
      throw SqlxError.driver(
        'SQLite path is required. Use Sqlite3Driver.open(path) or '
        'SqliteConnectOptions.path(path).',
      );
    }
    if (readOnly && createIfMissing) {
      throw SqlxError.driver(
        'SQLite read-only connections cannot create missing databases.',
      );
    }
    if (readOnly && hasMigrations) {
      throw SqlxError.driver(
        'SQLite read-only connections cannot apply Dust migrations.',
      );
    }
    final timeout = busyTimeout;
    if (timeout != null && timeout.inMilliseconds < 0) {
      throw SqlxError.driver('SQLite busyTimeout must not be negative.');
    }
    for (final entry in pragmas.entries) {
      _validatePragma(entry.key, entry.value);
    }
  }

  sqlite.OpenMode get _openMode {
    if (readOnly) return sqlite.OpenMode.readOnly;
    if (createIfMissing) return sqlite.OpenMode.readWriteCreate;
    return sqlite.OpenMode.readWrite;
  }

  String get _label => inMemory ? ':memory:' : path ?? '<missing path>';

  Iterable<String> _pragmaStatements() sync* {
    final timeout = busyTimeout;
    if (timeout != null) {
      yield 'PRAGMA busy_timeout = ${timeout.inMilliseconds}';
    }
    final keys = foreignKeys;
    if (keys != null) yield 'PRAGMA foreign_keys = ${keys ? 'ON' : 'OFF'}';
    final journal = journalMode;
    if (journal != null) yield 'PRAGMA journal_mode = ${journal._sql}';
    final sync = synchronous;
    if (sync != null) yield 'PRAGMA synchronous = ${sync._sql}';
    for (final entry in pragmas.entries) {
      yield 'PRAGMA ${entry.key} = ${_pragmaValue(entry.value)}';
    }
  }
}

void _validatePragma(String name, Object value) {
  if (!_sqlitePragmaName.hasMatch(name)) {
    throw SqlxError.driver('Invalid SQLite pragma name `$name`.');
  }
  if (value is bool || value is String) return;
  if (value is num && value.isFinite) return;
  throw SqlxError.driver(
    'Invalid SQLite pragma value for `$name`: ${value.runtimeType}.',
  );
}

String _pragmaValue(Object value) {
  if (value is bool) return value ? 'ON' : 'OFF';
  if (value is num) return value.toString();
  if (value is String) return "'${value.replaceAll("'", "''")}'";
  throw SqlxError.driver(
    'Invalid SQLite pragma value: ${value.runtimeType}.',
  );
}

extension on SqliteJournalMode {
  String get _sql {
    return switch (this) {
      SqliteJournalMode.delete => 'DELETE',
      SqliteJournalMode.truncate => 'TRUNCATE',
      SqliteJournalMode.persist => 'PERSIST',
      SqliteJournalMode.memory => 'MEMORY',
      SqliteJournalMode.wal => 'WAL',
      SqliteJournalMode.off => 'OFF',
    };
  }
}

extension on SqliteSynchronousMode {
  String get _sql {
    return switch (this) {
      SqliteSynchronousMode.off => 'OFF',
      SqliteSynchronousMode.normal => 'NORMAL',
      SqliteSynchronousMode.full => 'FULL',
      SqliteSynchronousMode.extra => 'EXTRA',
    };
  }
}
