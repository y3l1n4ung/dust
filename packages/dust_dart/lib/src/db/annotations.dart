import '../derive/base.dart';

/// Supported Dust DB driver targets.
enum Driver {
  /// SQLite through `package:sqlite3`.
  sqlite3,

  /// Reserved for the future Postgres runtime.
  postgres,
}

/// Supported SQLx database types.
enum SqlxDatabaseType {
  /// SQLite through `package:sqlite3`.
  sqlite,

  /// PostgreSQL.
  postgres,
}

/// Declares the top-level generated SQLx database open/configuration type.
final class SqlxDatabase {
  /// Creates one database generation marker.
  const SqlxDatabase({
    SqlxDatabaseType? type,
    Driver? driver,
    this.migrations = './migrations',
  }) : type =
           type ??
           (driver == Driver.postgres
               ? SqlxDatabaseType.postgres
               : SqlxDatabaseType.sqlite);

  /// SQLx database type used by the generated database.
  final SqlxDatabaseType type;

  /// Directory containing user-owned `.sql` migration files.
  final String migrations;
}

/// Backwards-compatible short database marker.
typedef Database = SqlxDatabase;

/// Marks a repository-style class for generated SQLx methods.
final class SqlxDao {
  /// Creates one DAO generation marker.
  const SqlxDao();
}

/// Backwards-compatible short DAO marker.
typedef Dao = SqlxDao;

/// Marks a generated SQL method inside a [SqlxDao].
final class Query {
  /// Creates one SQL query marker.
  const Query(this.sql);

  /// Static SQL source using sqlx-style `$1`, `$2`, `$3` placeholders.
  final String sql;
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

/// Derive trait for a class that Dust can construct from one database row.
final class FromRow extends DeriveTrait {
  /// Creates the `FromRow` derive marker.
  const FromRow();
}

/// sqlx-style row/class mapping options.
final class Sqlx {
  /// Creates one SQL row mapping configuration.
  const Sqlx({
    this.rename,
    this.renameAll,
    this.flatten = false,
    this.defaultValue,
    this.skip = false,
    this.json = false,
    this.tryFrom,
  });

  /// Explicit database column name for a field.
  final String? rename;

  /// Rename strategy applied at class level.
  final SqlxRename? renameAll;

  /// Decode this field from columns on the same row through another FromRow type.
  final bool flatten;

  /// Value used when a column is absent or when a skipped field is constructed.
  final Object? defaultValue;

  /// Ignore this field during row decoding.
  final bool skip;

  /// Decode a JSON text column with `fromJson(Map<String, Object?>)`.
  final bool json;

  /// Const converter object implementing [SqlxTryFrom].
  final Object? tryFrom;
}

/// Converts one database value into a Dart value during generated row mapping.
abstract interface class SqlxTryFrom<DartT, DbT> {
  /// Decodes a database value.
  DartT decode(DbT value);
}
