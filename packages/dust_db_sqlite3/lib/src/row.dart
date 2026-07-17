part of 'sqlite_pool.dart';

/// SQLite implementation of the Database row interface.
final class Sqlite3Row implements Row {
  /// Creates a typed view over one native sqlite3 result row.
  const Sqlite3Row(this._row);

  final sqlite.Row _row;

  @override
  T read<T>(String column) {
    final value = readNullable<T>(column);
    if (value == null) {
      throw _sqliteNullColumn(column, 'read:$column');
    }
    return value;
  }

  @override
  T? readNullable<T>(String column) {
    return _coerce<T>(_row[column], column);
  }

  @override
  T readIndex<T>(int index) {
    final value = readIndexNullable<T>(index);
    if (value == null) {
      throw _sqliteNullColumn('index $index', 'readIndex:$index');
    }
    return value;
  }

  @override
  T? readIndexNullable<T>(int index) {
    if (index < 0 || index >= _row.length) return null;
    return _coerce<T>(_row.columnAt(index), 'index $index');
  }

  @override
  bool readBool(String column) {
    final value = readBoolNullable(column);
    if (value == null) {
      throw _sqliteNullColumn(column, 'readBool:$column');
    }
    return value;
  }

  @override
  bool? readBoolNullable(String column) {
    final value = _row[column];
    if (value == null) return null;
    if (value is bool) return value;
    if (value is int) return value != 0;
    if (value is String) {
      return switch (value.toLowerCase()) {
        'true' || '1' => true,
        'false' || '0' => false,
        _ => throw _sqliteDecodeError(
            'Column `$column` cannot be read as bool.',
            operation: 'readBool:$column',
          ),
      };
    }
    throw _sqliteDecodeError(
      'Column `$column` cannot be read as bool.',
      operation: 'readBool:$column',
    );
  }

  @override
  DateTime readDateTime(String column) {
    final value = readDateTimeNullable(column);
    if (value == null) {
      throw _sqliteNullColumn(column, 'readDateTime:$column');
    }
    return value;
  }

  @override
  DateTime? readDateTimeNullable(String column) {
    final value = _row[column];
    if (value == null) return null;
    if (value is DateTime) return value.toUtc();
    if (value is String) {
      try {
        return DateTime.parse(value).toUtc();
      } on FormatException catch (error) {
        throw _sqliteDecodeError(
          'Column `$column` cannot be read as DateTime.',
          cause: error,
          operation: 'readDateTime:$column',
        );
      }
    }
    throw _sqliteDecodeError(
      'Column `$column` cannot be read as DateTime.',
      operation: 'readDateTime:$column',
    );
  }

  static T? _coerce<T>(Object? value, String column) {
    if (value == null) return null;
    if (T == double && value is int) return value.toDouble() as T;
    if (T == num && value is num) return value as T;
    if (value is T) return value as T;
    throw _sqliteDecodeError(
      'Column `$column` cannot be read as $T.',
      operation: 'read:$column',
    );
  }
}
