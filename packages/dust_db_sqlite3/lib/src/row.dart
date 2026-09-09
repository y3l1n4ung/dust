part of 'sqlite_pool.dart';

/// SQLite implementation of the Database row interface.
///
/// Reads the result set's own row data rather than the driver's `Row`, whose
/// constructor copies that data with `List.unmodifiable` for every row. Over
/// 1000 rows of six columns the copy and the lookups through it measured about
/// 90us, which was over half of what this adapter cost above the driver.
final class Sqlite3Row implements Row {
  /// Creates a typed view over one native sqlite3 result row.
  Sqlite3Row(sqlite.Row row)
      : _data = row.values,
        _names = row.keys,
        _sharedIndex = null;

  /// Wraps one row of a result whose rows share [_sharedIndex].
  Sqlite3Row._shared(this._data, this._names, this._sharedIndex);

  /// This row's values, by column position.
  final List<Object?> _data;

  /// Column names, in the same order.
  final List<String> _names;

  /// Column positions shared by every row of one result, when there is one.
  final Map<String, int>? _sharedIndex;

  /// Column positions for this row.
  ///
  /// `late` so a row read only by index never builds one: `fetchScalar` and
  /// `readIndex` do not need names. The result's own index is used when there
  /// is one, so a query returning a thousand rows builds this once rather than
  /// a thousand times.
  late final Map<String, int> _columnIndex =
      _sharedIndex ?? sqliteColumnIndex(_names);

  /// Reads a column by name, or null when the result has no such column.
  ///
  /// Null rather than a throw, because that is what the driver's own row does
  /// and a caller cannot tell an absent column from a NULL one either way.
  Object? _column(String column) {
    final index = _columnIndex[column];
    if (index == null) return null;
    return _data[index];
  }

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
    return _coerce<T>(_column(column), column);
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
    if (index < 0 || index >= _data.length) return null;
    return _coerce<T>(_data[index], 'index $index');
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
    final value = _column(column);
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
    final value = _column(column);
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

    // First, because it is what almost every read is: the value already has
    // the type the caller asked for. It also settles `num`, which an `int` and
    // a `double` both satisfy.
    if (value is T) return value as T;

    // SQLite stores a whole number in a REAL column as an integer, so a column
    // declared REAL hands back an `int` for `1.0`. Reading it as `double` is
    // the schema's answer, not a conversion the caller asked for.
    if (T == double && value is int) return value.toDouble() as T;

    throw _sqliteDecodeError(
      'Column `$column` cannot be read as $T.',
      operation: 'read:$column',
    );
  }
}

/// Maps column name to position for one result set.
///
/// Rows of one result share their column names, so the positions are worked
/// out once and every row reads through the same map. A name selected twice
/// resolves to the last of them, as the driver's own row does.
Map<String, int> sqliteColumnIndex(List<String> names) {
  final index = <String, int>{};
  for (var position = 0; position < names.length; position++) {
    index[names[position]] = position;
  }
  return index;
}
