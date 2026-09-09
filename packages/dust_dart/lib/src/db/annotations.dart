// coverage:ignore-file

import '../derive/base.dart';

/// Supported Database driver targets.
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
  ///
  /// [Driver] and [SqlxDatabaseType] name the same databases, and this maps one
  /// to the other. A const initializer cannot read a field off an enum value,
  /// so the mapping is a conditional with a default rather than a lookup, and a
  /// driver nobody adds a branch for silently becomes SQLite.
  ///
  /// `dust_dart_test.dart` walks [Driver.values] and fails on exactly that, so
  /// a new driver cannot be added without adding its branch here. The
  /// generator does not read this value — it parses the annotation source — so
  /// the mistake would only ever show up in a caller's own code.
  const SqlxDatabase({
    SqlxDatabaseType? type,
    Driver? driver,
    this.migrations = './migrations',
  }) : type = type ??
            (driver == Driver.postgres
                ? SqlxDatabaseType.postgres
                : SqlxDatabaseType.sqlite);

  /// SQLx database type used by the generated database.
  final SqlxDatabaseType type;

  /// Directory containing user-owned `.sql` or SQLx reversible migrations.
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
  /// Keep column names lowercased.
  lowerCase,

  /// Keep column names uppercased.
  upperCase,

  /// Convert field names to `PascalCase`.
  pascalCase,

  /// Convert field names to `camelCase`.
  camelCase,

  /// Convert field names to `snake_case`.
  snakeCase,

  /// Convert field names to `SCREAMING_SNAKE_CASE`.
  screamingSnakeCase,

  /// Convert field names to `kebab-case`.
  kebabCase,

  /// Convert field names to `SCREAMING-KEBAB-CASE`.
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
