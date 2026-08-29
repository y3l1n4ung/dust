import 'pool.dart';

/// Converts one database row into a typed Dart object.
typedef RowMapper<T> = T Function(Row row);

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

/// A [RowDeserializer] built from a plain [RowMapper] function.
///
/// The escape hatch for a row type Dust did not generate. Everything Dust does
/// generate has its own `$TypeFromRow`, which is the one the analyzer can check.
final class RowMapperDeserializer<T> implements RowDeserializer<T> {
  /// Wraps [mapper] as a row deserializer.
  const RowMapperDeserializer(this.mapper);

  /// The wrapped row mapping function.
  final RowMapper<T> mapper;

  @override
  T deserialize(Row row) => mapper(row);
}
