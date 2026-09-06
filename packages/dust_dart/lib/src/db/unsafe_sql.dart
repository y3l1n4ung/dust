import '../fp/result.dart';
import 'exec_result.dart';
import 'pool.dart';
import 'row_mapper.dart';
import 'sqlx_error.dart';

/// Unchecked SQL, for the cases build-time validation cannot reach.
///
/// Migrations, `EXPLAIN`, one-off administrative operations. Real, and rare.
/// Everything else has a checked form: an `IN` list is one bound list over
/// constant SQL, optional filters are a `switch` over described queries, and a
/// sort column is a `switch` over an enum, since no dialect binds an
/// identifier.
///
/// Three things about this type are deliberate.
///
/// It is **named to sting**. `queryRaw` read like a peer of `queryAs`; this
/// does not, and it greps.
///
/// It is reached from the **database facade**, never from an executor. A
/// request handler is handed an executor, and no cast takes an executor to a
/// [DatabaseClient], so a handler cannot reach unchecked SQL at all.
///
/// The decoder is **passed explicitly**. Generated terminals exist only for
/// validated queries, and the asymmetry is the point: the checked path is the
/// ergonomic one.
abstract interface class UnsafeSql {
  /// Runs unchecked SQL and returns untyped rows.
  Future<Result<List<Row>, SqlxError>> fetch(
    String sql,
    List<Object?> parameters,
  );

  /// Runs unchecked SQL and decodes each row with [mapper].
  Future<Result<List<T>, SqlxError>> fetchAs<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  );

  /// Runs an unchecked statement and returns execution metadata.
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  );
}
