# dust_db_sqlite3

SQLite runtime for `dust` database code generation.

## Usage

```dart
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

final pool = await Sqlite3Pool.open('app.db');
```

Use this package with generated `dust` DB code from `package:dust_dart/db.dart`.
