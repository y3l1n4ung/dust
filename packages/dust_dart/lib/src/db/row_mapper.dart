import 'pool.dart';
import 'sqlx_error.dart';

/// Converts one database row into a typed Dart object.
typedef RowMapper<T> = T Function(Row row);

/// Process-wide row mapper registry populated by generated `FromRow` code.
abstract final class RowMapperRegistry {
  static final Map<Type, RowMapper<Object?>> _mappers =
      <Type, RowMapper<Object?>>{};

  /// Registers a generated mapper for [T].
  static bool register<T>(RowMapper<T> mapper) {
    _mappers[T] = (row) => mapper(row);
    return true;
  }

  /// Decodes [row] as [T] using a generated mapper.
  static T map<T>(Row row) {
    if (T == Row) return row as T;
    final mapper = _mappers[T];
    if (mapper == null) {
      throw SqlxError.decode(
        'No Database FromRow mapper registered for $T. '
        'Pass a RowMapper directly or add @Derive([FromRow()]) and import '
        'the generated part file.',
        operation: 'RowMapperRegistry.map<$T>',
      );
    }
    return mapper(row) as T;
  }

  /// Clears registered mappers for tests.
  static void resetForTest() {
    _mappers.clear();
  }
}

/// Registers a generated row mapper and returns `true` for top-level initializers.
bool registerRowMapper<T>(RowMapper<T> mapper) {
  return RowMapperRegistry.register<T>(mapper);
}
