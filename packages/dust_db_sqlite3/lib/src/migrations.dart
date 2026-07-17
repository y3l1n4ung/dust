part of 'sqlite_pool.dart';

const _schemaMigrationsTable = '__dust_schema_migrations';

void _applyMigrations(
  sqlite.Database database,
  Map<String, String> migrations,
) {
  if (migrations.isEmpty) return;
  final pending = migrations.entries
      .where((migration) => !migration.key.endsWith('.down.sql'))
      .toList()
    ..sort((left, right) => left.key.compareTo(right.key));

  try {
    database.execute('BEGIN');
  } catch (error) {
    throw _sqliteMigrationError(
      'SQLite migration transaction begin failed.',
      cause: error,
      operation: 'BEGIN',
    );
  }
  try {
    _ensureMigrationTable(database);
    final applied = _appliedMigrations(database);
    for (final migration in pending) {
      if (applied.contains(migration.key)) continue;
      _applyMigration(database, migration.key, migration.value);
      _recordMigration(database, migration.key);
    }
    database.execute('COMMIT');
  } catch (error) {
    final migrationError = _asMigrationError(error);
    try {
      database.execute('ROLLBACK');
    } catch (rollbackError) {
      throw _sqliteMigrationError(
        'SQLite migration rollback failed.',
        cause: rollbackError,
        operation: 'ROLLBACK',
      );
    }
    throw migrationError;
  }
}

void _ensureMigrationTable(sqlite.Database database) {
  try {
    database.execute('''
CREATE TABLE IF NOT EXISTS $_schemaMigrationsTable (
  name TEXT PRIMARY KEY,
  applied_at TEXT NOT NULL
)
''');
  } catch (error) {
    throw _sqliteMigrationError(
      'SQLite migration table setup failed.',
      cause: error,
      operation: _schemaMigrationsTable,
    );
  }
}

Set<String> _appliedMigrations(sqlite.Database database) {
  try {
    final rows = database.select(
      'SELECT name FROM $_schemaMigrationsTable ORDER BY name',
    );
    return <String>{for (final row in rows) row['name'] as String};
  } catch (error) {
    throw _sqliteMigrationError(
      'SQLite migration history read failed.',
      cause: error,
      operation: _schemaMigrationsTable,
    );
  }
}

void _applyMigration(sqlite.Database database, String name, String sql) {
  try {
    database.execute(sql);
  } catch (error) {
    throw _sqliteMigrationError(
      'SQLite migration `$name` failed.',
      cause: error,
      operation: name,
    );
  }
}

void _recordMigration(sqlite.Database database, String name) {
  late final sqlite.PreparedStatement statement;
  try {
    statement = database.prepare(
      'INSERT INTO $_schemaMigrationsTable (name, applied_at) VALUES (?, ?)',
    );
  } catch (error) {
    throw _sqliteMigrationError(
      'SQLite migration history prepare failed.',
      cause: error,
      operation: name,
    );
  }
  try {
    statement.execute([name, DateTime.now().toUtc().toIso8601String()]);
  } catch (error) {
    throw _sqliteMigrationError(
      'SQLite migration history write failed.',
      cause: error,
      operation: name,
    );
  } finally {
    statement.close();
  }
}

SqlxError _asMigrationError(Object error) {
  if (error is SqlxError) return error;
  return _sqliteMigrationError(
    'SQLite migration failed.',
    cause: error,
  );
}
