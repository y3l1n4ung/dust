#!/usr/bin/env bash
set -euo pipefail

DUST_ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DUST_MIN_SDK_DIR="$(mktemp -d)"
trap 'rm -rf "$DUST_MIN_SDK_DIR"' EXIT

DUST_RUNTIME_DIR="$DUST_MIN_SDK_DIR/dust_db_sqlite3"
DUST_CONSUMER_DIR="$DUST_MIN_SDK_DIR/consumer"
mkdir -p "$DUST_RUNTIME_DIR" "$DUST_CONSUMER_DIR/bin"

# A workspace member cannot resolve outside its workspace. Copy only the
# published runtime surface and remove that repository-only marker so this
# exercises the dependency exactly as a Dart 3.10 consumer does.
awk '$0 != "resolution: workspace"' \
  "$DUST_ROOT_DIR/packages/dust_db_sqlite3/pubspec.yaml" \
  > "$DUST_RUNTIME_DIR/pubspec.yaml"
cp -R "$DUST_ROOT_DIR/packages/dust_db_sqlite3/lib" "$DUST_RUNTIME_DIR/lib"

cat > "$DUST_CONSUMER_DIR/pubspec.yaml" <<'EOF'
name: dust_sqlite_min_sdk_consumer
publish_to: none

environment:
  sdk: ">=3.10.0 <4.0.0"

dependencies:
  dust_db_sqlite3:
    path: ../dust_db_sqlite3
EOF

cat > "$DUST_CONSUMER_DIR/bin/main.dart" <<'EOF'
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

Future<void> main() async {
  final database = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const <String, String>{
      '0001_create_users.sql':
          'CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);',
    },
  );

  try {
    final inserted = await database.execute(
      'INSERT INTO users (id, name) VALUES (?, ?)',
      const <Object?>[1, 'Ada'],
    );
    inserted.match(
      ok: (_) {},
      err: (error) => throw StateError('$error'),
    );

    final count = await database.fetchScalar<int>(
      'SELECT COUNT(*) FROM users',
      const <Object?>[],
    );
    final value = count.match(
      ok: (value) => value,
      err: (error) => throw StateError('$error'),
    );
    if (value != 1) throw StateError('Expected one persisted row, got $value.');
  } finally {
    await database.close();
  }
}
EOF

cd "$DUST_CONSUMER_DIR"
dart pub get
dart analyze --fatal-infos
dart run bin/main.dart
