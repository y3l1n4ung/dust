import 'package:dust_dart/db.dart';

import '../models/account.dart';
import '../models/order.dart';

part 'queries.g.dart';

/// Every query the application makes.
///
/// Bound to a [DatabaseExecutor], so the same queries run on the connection or
/// inside a transaction.
@SqlxDao()
abstract final class AppQueries {
  /// Binds the queries to [db], a connection or a transaction.
  const factory AppQueries(DatabaseExecutor db) = _$AppQueries;

  /// The account a token belongs to, if the token exists and has not expired.
  ///
  /// The **hash** is what is looked up. The token itself is never stored, so a
  /// database read does not hand over working credentials — and expiry is
  /// checked in SQL rather than after, so an expired row cannot be used by code
  /// that forgets to look.
  @Query(r'''
SELECT a.id, a.email, a.password_hash, a.password_salt, a.scopes
FROM accounts a
JOIN api_tokens t ON t.account_id = a.id
WHERE t.token_hash = $1 AND t.expires_at > $2
''')
  Future<Result<Account?, SqlxError>> accountForToken(
    String tokenHash,
    String now,
  );

  /// One account by email, for signing in.
  @Query(r'''
SELECT id, email, password_hash, password_salt, scopes FROM accounts
WHERE email = $1
''')
  Future<Result<Account?, SqlxError>> accountByEmail(String email);

  /// Creates an account.
  @Query(r'''
INSERT INTO accounts (email, password_hash, password_salt, scopes)
VALUES ($1, $2, $3, $4)
''')
  Future<Result<ExecResult, SqlxError>> insertAccount(
    String email,
    String passwordHash,
    String passwordSalt,
    String scopes,
  );

  /// Stores a token's hash against an account.
  @Query(r'''
INSERT INTO api_tokens (account_id, token_hash, expires_at)
VALUES ($1, $2, $3)
''')
  Future<Result<ExecResult, SqlxError>> insertToken(
    int accountId,
    String tokenHash,
    String expiresAt,
  );

  /// Every order an account placed, newest first.
  ///
  /// Filtered by account in SQL. Filtering in Dart after fetching everything is
  /// how one customer reads another's orders the day someone forgets the `if`.
  @Query(r'''
SELECT id, account_id, item, quantity, placed_at FROM orders
WHERE account_id = $1
ORDER BY id DESC
''')
  Future<Result<List<Order>, SqlxError>> ordersFor(int accountId);

  /// One order, scoped to its owner.
  @Query(r'''
SELECT id, account_id, item, quantity, placed_at FROM orders
WHERE id = $1 AND account_id = $2
''')
  Future<Result<Order?, SqlxError>> orderFor(int id, int accountId);

  /// Places an order.
  @Query(r'''
INSERT INTO orders (account_id, item, quantity, placed_at)
VALUES ($1, $2, $3, $4)
''')
  Future<Result<ExecResult, SqlxError>> insertOrder(
    int accountId,
    String item,
    int quantity,
    String placedAt,
  );

  /// Cancels an order, scoped to its owner.
  @Query(r'''
DELETE FROM orders WHERE id = $1 AND account_id = $2
''')
  Future<Result<ExecResult, SqlxError>> deleteOrder(int id, int accountId);
}
