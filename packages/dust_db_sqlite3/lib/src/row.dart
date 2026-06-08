part of 'sqlite_pool.dart';

/// SQLite implementation of the Dust DB row interface.
final class Sqlite3Row implements Row {
  /// Creates a typed view over one native sqlite3 result row.
  const Sqlite3Row(this._row);

  final sqlite.Row _row;

  @override
  T read<T>(String column) {
    final value = readNullable<T>(column);
    if (value == null) {
      throw SqlxError.nullColumn(column);
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
      throw SqlxError.nullColumn('index $index');
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
      throw SqlxError.nullColumn(column);
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
        _ => throw SqlxError.decode('Column `$column` cannot be read as bool.'),
      };
    }
    throw SqlxError.decode('Column `$column` cannot be read as bool.');
  }

  @override
  DateTime readDateTime(String column) {
    final value = readDateTimeNullable(column);
    if (value == null) {
      throw SqlxError.nullColumn(column);
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
        throw SqlxError.decode(
          'Column `$column` cannot be read as DateTime.',
          cause: error,
        );
      }
    }
    throw SqlxError.decode('Column `$column` cannot be read as DateTime.');
  }

  static T? _coerce<T>(Object? value, String column) {
    if (value == null) return null;
    if (T == double && value is int) return value.toDouble() as T;
    if (T == num && value is num) return value as T;
    if (value is T) return value as T;
    throw SqlxError.decode('Column `$column` cannot be read as $T.');
  }
}
