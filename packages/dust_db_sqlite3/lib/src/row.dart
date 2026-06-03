part of 'sqlite_pool.dart';

/// SQLite result row adapter.
final class SqliteRow implements Row {
  /// Creates one row adapter.
  const SqliteRow(this._row);

  final sqlite.Row _row;

  @override
  T read<T>(String column) {
    final value = readOrNull<T>(column);
    if (value == null) {
      throw SqlxError.nullColumn(column);
    }
    return value;
  }

  @override
  T? readOrNull<T>(String column) {
    return _coerce<T>(_row[column]);
  }

  @override
  T readIndex<T>(int index) {
    final value = readIndexOrNull<T>(index);
    if (value == null) {
      throw SqlxError.nullColumn('index $index');
    }
    return value;
  }

  @override
  T? readIndexOrNull<T>(int index) {
    return _coerce<T>(_row[index]);
  }

  @override
  bool readBool(String column) {
    final value = readBoolOrNull(column);
    if (value == null) {
      throw SqlxError.nullColumn(column);
    }
    return value;
  }

  @override
  bool? readBoolOrNull(String column) {
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
    final value = readDateTimeOrNull(column);
    if (value == null) {
      throw SqlxError.nullColumn(column);
    }
    return value;
  }

  @override
  DateTime? readDateTimeOrNull(String column) {
    final value = _row[column];
    if (value == null) return null;
    if (value is DateTime) return value.toUtc();
    if (value is String) return DateTime.parse(value).toUtc();
    throw SqlxError.decode('Column `$column` cannot be read as DateTime.');
  }

  T? _coerce<T>(Object? value) {
    if (value == null) return null;
    if (T == double && value is int) return value.toDouble() as T;
    if (T == num && value is num) return value as T;
    return value as T;
  }
}
