part of 'postgres_pool.dart';

/// One PostgreSQL result row, read by column name or index.
///
/// Postgres decodes on the wire, so most of these are casts where the SQLite
/// adapter has to interpret: `boolean` arrives as a `bool` rather than as 0 or
/// 1, and `timestamptz` arrives as a `DateTime` rather than as ISO-8601 text.
final class PostgresRow implements Row {
  /// Wraps one driver row.
  PostgresRow(this._row, {String operation = ''})
      : _sharedIndex = null,
        _operation = operation;

  /// Wraps one row of a result whose rows share [_sharedIndex].
  PostgresRow._shared(this._row, this._sharedIndex, this._operation);

  final pg.ResultRow _row;

  /// Column positions shared by every row of one result, when there is one.
  final Map<String, int>? _sharedIndex;

  /// Column positions for this row.
  ///
  /// `late` so a row read only by index never builds one: `fetchScalar` and
  /// `readIndex` do not need names. The result's own index is used when there
  /// is one, so a query returning a thousand rows builds this once rather than
  /// a thousand times.
  late final Map<String, int> _columnIndex =
      _sharedIndex ?? postgresColumnIndex(_row.schema);

  final String _operation;

  /// Reads a column by name, or throws when the result has no such column.
  Object? _column(String column) {
    final index = _columnIndex[column];
    if (index == null) {
      throw _postgresDecodeError(
        'PostgreSQL result has no column `$column`.',
        operation: _operation,
      );
    }
    return _row[index];
  }

  @override
  T read<T>(String column) {
    final value = readNullable<T>(column);
    if (value == null) throw _postgresNullColumn(column, _operation);
    return value;
  }

  @override
  T? readNullable<T>(String column) {
    final value = _column(column);
    if (value == null) return null;
    if (value is! T) {
      throw _postgresDecodeError(
        'PostgreSQL column `$column` is ${value.runtimeType}, not $T.',
        operation: _operation,
      );
    }
    // The guard above establishes this; Dart does not promote to a type
    // variable on its own.
    return value as T;
  }

  @override
  T readIndex<T>(int index) {
    final value = readIndexNullable<T>(index);
    if (value == null) throw _postgresNullColumn('[$index]', _operation);
    return value;
  }

  @override
  T? readIndexNullable<T>(int index) {
    if (index < 0 || index >= _row.length) {
      throw _postgresDecodeError(
        'PostgreSQL result has no column at index $index.',
        operation: _operation,
      );
    }
    return _row[index] as T?;
  }

  /// Reads a `boolean` column.
  ///
  /// Postgres has a real boolean type, so unlike SQLite there is nothing to
  /// interpret. An integer column is still accepted, because a query is free to
  /// select one and the SQLite side allows it.
  @override
  bool readBool(String column) {
    final value = readBoolNullable(column);
    if (value == null) throw _postgresNullColumn(column, _operation);
    return value;
  }

  @override
  bool? readBoolNullable(String column) {
    final value = _column(column);
    if (value == null) return null;
    if (value is bool) return value;
    if (value is int) return value != 0;
    throw _postgresDecodeError(
      'PostgreSQL column `$column` is ${value.runtimeType}, not a boolean.',
      operation: _operation,
    );
  }

  /// Reads a `timestamptz` or `timestamp` column, normalised to UTC.
  ///
  /// The driver decodes these to `DateTime` already. Text is still accepted so
  /// that a query selecting a formatted timestamp behaves as it does on SQLite.
  @override
  DateTime readDateTime(String column) {
    final value = readDateTimeNullable(column);
    if (value == null) throw _postgresNullColumn(column, _operation);
    return value;
  }

  @override
  DateTime? readDateTimeNullable(String column) {
    final value = _column(column);
    if (value == null) return null;
    if (value is DateTime) return value.toUtc();
    if (value is String) {
      final parsed = DateTime.tryParse(value);
      if (parsed == null) {
        throw _postgresDecodeError(
          'PostgreSQL column `$column` is not an ISO-8601 date/time.',
          operation: _operation,
        );
      }
      return parsed.toUtc();
    }
    throw _postgresDecodeError(
      'PostgreSQL column `$column` is ${value.runtimeType}, not a date/time.',
      operation: _operation,
    );
  }
}

/// Maps column name to position for one result schema.
///
/// The driver offers `ResultRow.toColumnMap()`, which allocates a map of every
/// value for every row. Rows of one result share a schema, so the positions can
/// be worked out once and the values read straight out of the row.
///
/// An unnamed column is keyed `[i]`, and a name appearing twice resolves to the
/// last of them — both as `toColumnMap` does, since a query is free to select
/// either and the behaviour should not depend on how the row is read.
Map<String, int> postgresColumnIndex(pg.ResultSchema schema) {
  final index = <String, int>{};
  for (final (position, column) in schema.columns.indexed) {
    index[column.columnName ?? '[$position]'] = position;
  }
  return index;
}
