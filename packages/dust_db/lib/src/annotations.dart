/// Supported Dust DB runtime driver targets.
enum Driver {
  /// Flutter sqflite driver.
  sqflite,
}

/// sqlx-style field/class rename strategies.
enum SqlxRename {
  lowerCase,
  upperCase,
  pascalCase,
  camelCase,
  snakeCase,
  screamingSnakeCase,
  kebabCase,
  screamingKebabCase,
}

/// Marks an abstract repository that Dust implements from raw SQL methods.
final class DustDb {
  const DustDb({required this.driver, required this.migrations});

  final Driver driver;
  final String migrations;
}

/// Raw SQL query carried by a repository method body.
final class Query {
  const Query(this.sql);

  final String sql;
}

/// Marks a repository method as a single sqflite transaction.
final class Transaction {
  const Transaction();
}

/// Marks a row class that Dust can construct from one database row.
final class FromRow {
  const FromRow();
}

/// sqlx-style row/class mapping options.
final class Sqlx {
  const Sqlx({
    this.rename,
    this.renameAll,
    this.flatten = false,
    this.defaultValue,
    this.skip = false,
    this.json = false,
    this.tryFrom,
  });

  final String? rename;
  final SqlxRename? renameAll;
  final bool flatten;
  final Object? defaultValue;
  final bool skip;
  final bool json;
  final Object? tryFrom;
}

/// Converts one database value into a Dart value during generated row mapping.
abstract interface class SqlxTryFrom<DartT, DbT> {
  const SqlxTryFrom();

  DartT decode(DbT value);
}

/// Marker API used in repository method bodies.
const $fetch = _Fetch();

final class _Fetch {
  const _Fetch();

  Future<T?> one<T>([Object? args]) => throw UnimplementedError();
  Future<List<T>> all<T>([Object? args]) => throw UnimplementedError();
  Future<num> scalar([Object? args]) => throw UnimplementedError();
  Future<T> insertOne<T>([Object? args]) => throw UnimplementedError();
  Future<void> execute([Object? args]) => throw UnimplementedError();
  Stream<T> stream<T>([Object? args]) => throw UnimplementedError();
}
