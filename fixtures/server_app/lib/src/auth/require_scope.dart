import 'package:dust_server/server.dart';

import '../db/queries.dart';
import '../models/account.dart';
import 'tokens.dart';

/// Authenticates a bearer token against the database.
///
/// The token arrives in full and is looked up **by fingerprint**, so nothing
/// stored can be replayed. Expiry is checked in the query rather than after it,
/// which is what stops an expired row being used by code that forgets to look.
///
/// A missing or malformed credential is **401** — retry with one. A real
/// credential lacking the scope is **403** — retrying is pointless. Answering
/// 401 for both makes a client that retries on 401 loop forever.
final class RequireScope implements FromRequestParts<Account> {
  /// Requires [scope], or any valid token when it is null.
  const RequireScope([this.scope]);

  /// The scope a caller must hold.
  final String? scope;

  @override
  Future<Result<Account, Rejection>> extract(Request request) async {
    switch (await const BearerTokenExtractable().extract(request)) {
      case Err(:final error):
        return Err(error);
      case Ok(value: final token):
        final queries = await const StateExtractable<AppQueries>()
            .extract(request)
            .then((outcome) => outcome);
        if (queries case Err(:final error)) return Err(error);

        final found =
            await (queries as Ok<AppQueries, Rejection>).value.accountForToken(
                  Tokens.fingerprint(token),
                  DateTime.now().toUtc().toIso8601String(),
                );

        switch (found) {
          case Err(:final error):
            // A database fault is a 500, not a 401. Telling a caller their
            // credential is bad when the database is down sends them to reset a
            // password that was never the problem.
            ServerErrors.report(error, StackTrace.current);
            return const Err(Rejection.internal());
          case Ok(value: final account?):
            final required = scope;
            if (required != null && !account.grantedScopes.contains(required)) {
              return Err(Rejection.forbidden('requires scope $required'));
            }
            return Ok(account);
          case Ok():
            // Unknown or expired — the same answer either way, so a caller
            // cannot tell a wrong token from an expired one.
            return const Err(Rejection.unauthorized('invalid token'));
        }
    }
  }
}

/// The scoped variant, which is how configuration reaches `@Extract`.
final class OrdersWrite extends RequireScope {
  /// Requires `orders:write`.
  const OrdersWrite() : super('orders:write');
}
