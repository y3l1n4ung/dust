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

/// Whether [value] is a list Dust binds as JSON text.
///
/// A `Uint8List` is how a BLOB is bound, so it is excluded: encoding one would
/// silently write a JSON array of byte values in place of the bytes.
bool _isJsonListBind(Object? value) => value is List && value is! Uint8List;

/// Encodes `List` arguments as JSON text so one placeholder carries a set.
///
/// SQLite has no array type. A set membership test is written
/// `WHERE id IN (SELECT value FROM json_each($1))` — constant SQL, one
/// placeholder, describable at build time — and `package:sqlite3` will not bind
/// a nested `List`. Encoding here rather than at the call site is what keeps
/// callers from reaching for dynamic SQL to build an `IN (?, ?, ?)`.
List<Object?> _encodeListParameters(List<Object?> parameters) {
  if (!parameters.any(_isJsonListBind)) return parameters;
  return <Object?>[
    for (final parameter in parameters)
      if (_isJsonListBind(parameter))
        _encodeJsonListBind(parameter! as List<Object?>)
      else
        parameter,
  ];
}

/// Encodes one list argument, reporting a value JSON cannot carry.
String _encodeJsonListBind(List<Object?> values) {
  try {
    return jsonEncode(values);
  } catch (error) {
    throw _sqliteQueryError(
      'SQLite binds a List argument as JSON text for the `json_each` idiom, '
      'and this list holds a value JSON cannot represent.',
      cause: error,
      operation: 'bind',
    );
  }
}

extension _Sqlite3DriverOperations on Sqlite3Driver {
  /// Runs [sql] and returns the driver's own result set.
  ///
  /// Rows are wrapped by the caller rather than here, so a terminal reading one
  /// row builds one adapter instead of a list of them.
  sqlite.ResultSet _selectUnchecked(String sql, List<Object?> parameters) {
    _checkOpen();
    final rewrite = rewritePlaceholders(sql);
    final statement = _connection.statements.statementFor(
      _database,
      rewrite.sql,
    );
    return statement.select(
      _encodeListParameters(orderParameters(rewrite, parameters)),
    );
  }

  ExecResult _executeUnchecked(String sql, List<Object?> parameters) {
    _checkOpen();
    final rewrite = rewritePlaceholders(sql);
    final statement = _connection.statements.statementFor(
      _database,
      rewrite.sql,
    );
    statement.execute(
      _encodeListParameters(orderParameters(rewrite, parameters)),
    );
    return ExecResult(
      rowsAffected: _database.updatedRows,
      lastInsertId: _database.lastInsertRowId,
    );
  }
}
