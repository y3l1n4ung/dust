# dust_db_sqlite3

SQLite runtime for DustDB code generation.

## Usage

```dart
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

final pool = await Sqlite3Pool.open('app.db');
```

Use this package with generated DB code from `package:dust_dart/db.dart`.
