/// sqlite3 runtime for Database.
library;

export 'src/sqlite_pool.dart';

// Every query answers a `Result`, and its methods are extensions, which are in
// scope only where imported. Exporting the extensions without the types keeps
// `db.fetchScalar(...).unwrapOr(0)` compiling with this import alone, and
// cannot collide with another package's `Result` class.
export 'package:dust_dart/fp.dart'
    show
        ResultChain,
        ResultCollect,
        ResultFlatten,
        ResultQuery,
        ResultTransform,
        ResultTranspose;
