part of 'postgres_pool.dart';

/// Table recording which migrations a database has already run.
///
/// The same name the SQLite runtime uses, so a project reading its own schema
/// finds the same thing on either dialect.
const _schemaMigrationsTable = '__dust_schema_migrations';

/// Applies unapplied migrations in name order, inside one transaction.
///
/// An advisory lock guards the whole run. Unlike SQLite, where one process
/// holds the file, several servers can start against the same database at once
/// and would otherwise race to apply the same migration.
Future<Result<Unit, SqlxError>> _applyMigrations(
  PostgresExecutor executor,
  Map<String, String> migrations,
) async {
  if (migrations.isEmpty) return const Ok<Unit, SqlxError>(unit);

  // `.down.sql` files describe how to undo a migration, and are never applied
  // going forward. The SQLite runtime skips them the same way.
  final pending = migrations.entries
      .where((migration) => !migration.key.endsWith('.down.sql'))
      .toList()
    ..sort((left, right) => left.key.compareTo(right.key));

  return executor.transaction<Unit>((tx) async {
    final locked = await tx.execute(
      r'SELECT pg_advisory_xact_lock($1)',
      const <Object?>[_migrationLockKey],
    );
    if (locked case Err(:final error)) return Err<Unit, SqlxError>(error);

    final ensured = await tx.execute(
      'CREATE TABLE IF NOT EXISTS $_schemaMigrationsTable ('
      'name TEXT PRIMARY KEY, '
      'applied_at TIMESTAMPTZ NOT NULL DEFAULT now())',
      const <Object?>[],
    );
    if (ensured case Err(:final error)) return Err<Unit, SqlxError>(error);

    final appliedRows = await tx.fetchAll<String>(
      'SELECT name FROM $_schemaMigrationsTable',
      const <Object?>[],
      (row) => row.read<String>('name'),
    );
    if (appliedRows case Err(:final error)) return Err<Unit, SqlxError>(error);
    final applied = appliedRows.unwrapOrElse((_) => const <String>[]).toSet();

    for (final migration in pending) {
      if (applied.contains(migration.key)) continue;

      // Migration files hold more than one statement, and the extended query
      // protocol the driver uses by default allows only one per prepared
      // statement. Simple mode takes the whole file; it accepts no parameters,
      // which a migration does not have.
      try {
        await (tx as PostgresExecutor).session.execute(
              migration.value,
              queryMode: pg.QueryMode.simple,
            );
      } catch (error) {
        return Err<Unit, SqlxError>(
          _postgresMigrationError(
            migration.key,
            _asPostgresError(error, migration.key),
          ),
        );
      }

      final recorded = await tx.execute(
        'INSERT INTO $_schemaMigrationsTable (name) VALUES (' r'$1)',
        <Object?>[migration.key],
      );
      if (recorded case Err(:final error)) {
        return Err<Unit, SqlxError>(
          _postgresMigrationError(migration.key, error),
        );
      }
    }
    return const Ok<Unit, SqlxError>(unit);
  });
}

/// Key for the advisory lock migrations run under.
///
/// Any constant works as long as every Dust process agrees on it; this one is
/// arbitrary and fixed.
const int _migrationLockKey = 0x64757374;

/// Names the migration that failed, keeping the server's own error as cause.
SqlxError _postgresMigrationError(String name, SqlxError cause) {
  return SqlxError.migration(
    'PostgreSQL migration `$name` failed.',
    cause: cause,
    driver: Driver.postgres,
    operation: name,
  );
}
