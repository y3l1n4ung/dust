/// A fixture for the server plugin: annotated handlers, hand-emitted output.
///
/// Two halves, and the split is the point:
///
/// * **Generated for real** by `dust build` — the models (`Deserialize`,
///   `Validate`, `Serialize`, `FromRow`), the database, and the queries.
/// * **Hand-written** — `lib/src/api/*.g.dart`, which is what the server plugin
///   will emit, and the spec it has to satisfy.
///
/// ```text
/// lib/src/
///   api/auth.dart      -> auth.g.dart      hand-written
///   api/orders.dart    -> orders.g.dart    hand-written
///   api/admin.dart     -> admin.g.dart     hand-written
///   auth/passwords.dart               PBKDF2, salted, constant-time
///   auth/tokens.dart                  issued from Random.secure(), stored hashed
///   auth/require_scope.dart           looks the token up in the database
///   db/database.dart   -> database.g.dart  generated
///   db/queries.dart    -> queries.g.dart   generated
///   models/*.dart      -> *.g.dart         generated
/// ```
library;

export 'src/api/admin.dart';
export 'src/api/auth.dart';
export 'src/api/orders.dart';
export 'src/auth/passwords.dart';
export 'src/auth/require_scope.dart';
export 'src/auth/tokens.dart';
export 'src/db/database.dart';
export 'src/db/queries.dart';
export 'src/models/account.dart';
export 'src/models/credentials.dart';
export 'src/models/new_order.dart';
export 'src/models/order.dart';
