# Dust DB

Dust DB generates sqflite-style repository implementations and row mappers from raw SQL. It is not an ORM and does not own your schema.

## Dependencies

```yaml
dependencies:
  dust_db:
    path: ../../packages/dust_db
```

## Row Mapping

`FromRow` is owned by `dust_db`. Do not put row mapping in derive core.

```dart
import 'dart:convert';

import 'package:dust_db/dust_db.dart';

part 'user_repository.g.dart';

@FromRow()
@Sqlx(renameAll: SqlxRename.snakeCase)
final class UserProfile {
  const UserProfile({
    required this.id,
    required this.name,
    required this.address,
    this.bio = '',
    this.sessionActive = false,
    required this.preferences,
    required this.status,
  });

  final int id;

  @Sqlx(rename: 'display_name')
  final String name;

  @Sqlx(flatten: true)
  final Address address;

  final String bio;

  @Sqlx(skip: true)
  final bool sessionActive;

  @Sqlx(json: true)
  final UserPreferences preferences;

  @Sqlx(tryFrom: UserStatusFromInt())
  final UserStatus status;
}
```

## Repository

Use normal method annotations for most queries. Dust infers fetch behavior from return type.

```dart
@DustDb(driver: Driver.sqflite, migrations: 'migrations')
abstract interface class UserRepository {
  factory UserRepository(dynamic db) = _$UserRepository;

  @Query('SELECT id, display_name FROM users WHERE id = ?')
  Future<UserProfile?> findById(int id);

  @Query('SELECT id, display_name FROM users')
  Future<List<UserProfile>> listProfiles();

  @Query('SELECT COUNT(*) FROM users')
  Future<int> countProfiles();

  @Transaction()
  @Query('UPDATE users SET display_name = ? WHERE id = ?')
  Future<void> renameProfile(String name, int id);
}
```

## SQLx-Style Options

| Option | Purpose |
| :--- | :--- |
| `renameAll` | Apply a class-level column naming rule. |
| `rename` | Override one column name. |
| `flatten` | Decode another `@FromRow()` type from the same row. |
| `defaultValue` | Use a value when a column is absent. |
| `skip` | Ignore a field; it must have a default. |
| `json` | Decode a text column with `jsonDecode` and `Type.fromJson`. |
| `tryFrom` | Decode a DB value through a `SqlxTryFrom` converter. |

## CLI

```bash
dust build --db
dust build --db --offline
dust check --db --offline
```

Online `dust build --db` writes `.dart_tool/dust/db_query_cache_v1.json`. Offline mode uses that cache and fails if a required SQL/schema entry is missing or stale.

## Example

See `examples/dust_db_example` for migrations, flattened row mapping, JSON, tryFrom, transactions, scalar queries, and tests.
