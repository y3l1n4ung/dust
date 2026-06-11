part of 'sqlite_pool.dart';

const _schemaMigrationsTable = '__dust_schema_migrations';

void _applyMigrations(
  sqlite.Database database,
  Map<String, String> migrations,
) {
  if (migrations.isEmpty) return;
  final pending = migrations.entries.toList()
    ..sort((left, right) => left.key.compareTo(right.key));

  database.execute('BEGIN');
  try {
    _ensureMigrationTable(database);
    final applied = _appliedMigrations(database);
    for (final migration in pending) {
      if (applied.contains(migration.key)) continue;
      database.execute(migration.value);
      _recordMigration(database, migration.key);
    }
    database.execute('COMMIT');
  } catch (_) {
    database.execute('ROLLBACK');
    rethrow;
  }
}

void _ensureMigrationTable(sqlite.Database database) {
  database.execute('''
CREATE TABLE IF NOT EXISTS $_schemaMigrationsTable (
  name TEXT PRIMARY KEY,
  applied_at TEXT NOT NULL
)
''');
}

Set<String> _appliedMigrations(sqlite.Database database) {
  final rows = database.select(
    'SELECT name FROM $_schemaMigrationsTable ORDER BY name',
  );
  return <String>{for (final row in rows) row['name'] as String};
}

void _recordMigration(sqlite.Database database, String name) {
  final statement = database.prepare(
    'INSERT INTO $_schemaMigrationsTable (name, applied_at) VALUES (?, ?)',
  );
  try {
    statement.execute([name, DateTime.now().toUtc().toIso8601String()]);
  } finally {
    statement.close();
  }
}
