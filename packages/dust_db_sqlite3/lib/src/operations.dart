part of 'sqlite_pool.dart';

sqlite.Database _openDatabase(SqliteConnectOptions options) {
  try {
    if (options.inMemory) return sqlite.sqlite3.openInMemory();
    return sqlite.sqlite3.open(options.path!, mode: options._openMode);
  } catch (error) {
    throw _sqliteConnectionError(
      'SQLite database open failed for `${options._label}`.',
      cause: error,
      operation: 'open',
    );
  }
}

void _applyConnectOptions(
  sqlite.Database database,
  SqliteConnectOptions options,
) {
  for (final statement in options._pragmaStatements()) {
    try {
      database.execute(statement);
    } catch (error) {
      throw _sqliteConnectionError(
        'SQLite connection option failed: `$statement`.',
        cause: error,
        operation: statement,
      );
    }
  }
}

extension _Sqlite3DriverOperations on Sqlite3Driver {
  List<Row> _queryUnchecked(String sql, List<Object?> parameters) {
    _checkOpen();
    final result = _database.select(sql, parameters);
    return <Row>[for (final row in result) Sqlite3Row(row)];
  }

  ExecResult _executeUnchecked(String sql, List<Object?> parameters) {
    _checkOpen();
    final statement = _database.prepare(sql);
    try {
      statement.execute(parameters);
      return ExecResult(
        rowsAffected: _database.updatedRows,
        lastInsertId: _database.lastInsertRowId,
      );
    } finally {
      statement.close();
    }
  }
}
