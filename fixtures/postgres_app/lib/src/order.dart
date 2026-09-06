import 'package:dust_dart/db.dart';

part 'order.g.dart';

/// One row of `orders`.
///
/// The column types are the ones PostgreSQL decodes differently from SQLite:
/// `express` arrives as a `bool` rather than 0 or 1, and `placedAt` as a
/// `DateTime` rather than ISO-8601 text.
@Derive([FromRow()])
@Sqlx(renameAll: SqlxRename.snakeCase)
final class Order {
  /// Creates one order row.
  const Order({
    required this.id,
    required this.accountId,
    required this.item,
    required this.quantity,
    required this.express,
    required this.placedAt,
  });

  /// Generated key, read back through `RETURNING`.
  final int id;

  /// Owning account.
  final int accountId;

  /// What was ordered.
  final String item;

  /// How many.
  final int quantity;

  /// Whether this order ships express.
  final bool express;

  /// When it was placed, in UTC.
  final DateTime placedAt;
}
