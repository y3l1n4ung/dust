part of 'postgres_pool.dart';

/// One PostgreSQL result row, read by column name or index.
///
/// Postgres decodes on the wire, so most of these are casts where the SQLite
/// adapter has to interpret: `boolean` arrives as a `bool` rather than as 0 or
/// 1, and `timestamptz` arrives as a `DateTime` rather than as ISO-8601 text.
final class PostgresRow implements Row {
  /// Wraps one driver row.
  PostgresRow(this._row, {String operation = ''})
      : _columns = _row.toColumnMap(),
        _operation = operation;

  final pg.ResultRow _row;
  final Map<String, dynamic> _columns;
  final String _operation;

  @override
  T read<T>(String column) {
    final value = readNullable<T>(column);
    if (value == null) throw _postgresNullColumn(column, _operation);
    return value;
  }

  @override
  T? readNullable<T>(String column) {
    if (!_columns.containsKey(column)) {
      throw _postgresDecodeError(
        'PostgreSQL result has no column `$column`.',
        operation: _operation,
      );
    }
    final value = _columns[column];
    if (value == null) return null;
    if (value is! T) {
      throw _postgresDecodeError(
        'PostgreSQL column `$column` is ${value.runtimeType}, not $T.',
        operation: _operation,
      );
    }
    return value;
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
    final value = _columns[column];
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
    final value = _columns[column];
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
