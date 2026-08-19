import 'package:dust_server/server.dart';

import 'package:dust_dart/db.dart';

import '../auth/passwords.dart';
import '../auth/tokens.dart';
import '../db/queries.dart';
import '../models/account.dart';
import '../models/credentials.dart';

part 'auth.g.dart';

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
  @State() AppQueries queries,
  @Body() Credentials input,
) async {
  const invalid = Err<IssuedToken, Rejection>(
    Rejection.unauthorized('invalid credentials'),
  );

  final found = await queries.accountByEmail(input.email);
  if (found case Err()) return const Err(Rejection.internal());

  final account = (found as Ok<Account?, SqlxError>).value;
  if (account == null) {
    // Same work, thrown away, so the timing matches a real account.
    Passwords.verify(input.password, _dummySalt, _dummyHash);
    return invalid;
  }

  if (!Passwords.verify(
    input.password,
    account.passwordSalt,
    account.passwordHash,
  )) {
    return invalid;
  }

  final token = Tokens.issue();
  final expiresAt =
      DateTime.now().toUtc().add(const Duration(days: 7)).toIso8601String();

  final stored = await queries.insertToken(
    account.id,
    Tokens.fingerprint(token),
    expiresAt,
  );
  if (stored case Err()) return const Err(Rejection.internal());

  // The only time the token is ever readable.
  return Ok(IssuedToken(token: token, expiresAt: expiresAt));
}

/// A salt and hash for an account that does not exist, so the work is real.
final String _dummySalt = Passwords.newSalt();
final String _dummyHash = Passwords.hash('not a real password', _dummySalt);
