import 'package:dust_server/server.dart';

import 'package:dust_dart/db.dart';

import '../../shared/auth/passwords.dart';
import '../../shared/auth/tokens.dart';
import 'account_model.dart';
import 'accounts_repo.dart';
import 'credentials_dto.dart';
import 'require_scope.dart';

part 'accounts_handler.g.dart';

// Signing in. This is the one endpoint that sees a password.

/// `POST /tokens` — exchange a password for a token.
///
/// One answer for every failure: a wrong email and a wrong password are both
/// `invalid credentials`. Distinguishing them turns the endpoint into a way to
/// find out which email addresses have accounts.
///
/// The password is verified even when no account was found, against a dummy
/// hash. Skipping that makes a missing account measurably faster to reject than
/// a wrong password, which leaks the same thing timing-wise that the message
/// would have leaked outright.
@POST('/tokens', status: 201)
Future<Result<IssuedToken, Rejection>> signIn(
  @State() AccountsRepo repo,
  @Body() Credentials input,
) async {
  const invalid = Err<IssuedToken, Rejection>(
    Rejection.unauthorized('invalid credentials'),
  );

  final found = await repo.accountByEmail(input.email);
  if (found case Err()) return const Err(Rejection.internal());

  final account = (found as Ok<Account?, SqlxError>).value;
  if (account == null) {
    // Same work, thrown away, so the timing matches a real account.
    await Passwords.verify(input.password, _dummySalt, await _dummyHash());
    return invalid;
  }

  if (!await Passwords.verify(
    input.password,
    account.passwordSalt,
    account.passwordHash,
  )) {
    return invalid;
  }

  final token = Tokens.issue();
  final expiresAt =
      DateTime.now().toUtc().add(const Duration(days: 7)).toIso8601String();

  final stored = await repo.insertToken(
    account.id,
    await Tokens.fingerprint(token),
    expiresAt,
  );
  if (stored case Err()) return const Err(Rejection.internal());

  // The only time the token is ever readable.
  return Ok(IssuedToken(token: token, expiresAt: expiresAt));
}

/// A salt and hash for an account that does not exist, so the work is real.
///
/// Computed once and kept, because deriving it per request would double the
/// cost of every sign-in that succeeds — the point is to spend the same time as
/// a real verification, not twice it.
final String _dummySalt = Passwords.newSalt();
String? _cachedDummyHash;

Future<String> _dummyHash() async => _cachedDummyHash ??=
    await Passwords.hash('not a real password', _dummySalt);

/// `GET /me` — who the token belongs to.
///
/// Answers with [AccountView], never [Account]: the stored type carries a hash
/// and a salt, and it derives no `Serialize`, so this cannot go wrong quietly.
@GET('/me')
Future<AccountView> whoAmI(
  @Extract(RequireScope) Account account,
) async {
  return AccountView.of(account);
}
