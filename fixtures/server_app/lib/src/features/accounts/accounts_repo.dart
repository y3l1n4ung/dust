import 'package:dust_dart/db.dart';

import 'account_model.dart';

part 'accounts_repo.g.dart';

/// Every query the accounts feature makes.
///
/// One DAO per feature, bound to a [DatabaseExecutor] so the same queries run
/// on the connection or inside a transaction. A feature owning its own queries
/// is what keeps a schema change to one directory.
@SqlxDao()
abstract final class AccountsRepo {
  /// Binds the queries to [db].
  const factory AccountsRepo(DatabaseExecutor db) = _$AccountsRepo;

  /// The account a token belongs to, if it exists and has not expired.
  ///
  /// The **fingerprint** is what is looked up, and expiry is checked in SQL —
  /// so a stale row cannot be used by code that forgets to look.
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

  /// Stores a token's fingerprint against an account.
  @Query(r'''
INSERT INTO api_tokens (account_id, token_hash, expires_at)
VALUES ($1, $2, $3)
''')
  Future<Result<ExecResult, SqlxError>> insertToken(
    int accountId,
    String tokenHash,
    String expiresAt,
  );
}
