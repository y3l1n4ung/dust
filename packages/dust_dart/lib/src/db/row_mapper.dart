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

/// Directional conversion contract for generated and custom row mappers.
///
/// This mirrors `Deserializer` on the JSON side, and for the same reason.
/// `serialize()` can be an instance method because the value already exists;
/// reading a row *constructs* one, so there is no instance to declare it on.
/// The generated witness object carries the capability instead, which is what
/// lets the analyzer see that a type has no row mapping — passing a type with
/// no `@Derive([FromRow()])` names a `$TFromRow` that does not exist.
abstract interface class RowDeserializer<T> {
  /// Converts [row] into a Dart value.
  T deserialize(Row row);
}

/// Plain-function view of a row deserializer.
extension RowDeserializerMapper<T> on RowDeserializer<T> {
  /// This deserializer as a [RowMapper], for APIs that take the function form.
  RowMapper<T> get asMapper => deserialize;
}
