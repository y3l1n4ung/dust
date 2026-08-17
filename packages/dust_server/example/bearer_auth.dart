import 'dart:io';

import 'package:dust_server/server.dart';

/// Requiring a bearer token.
///
/// The runtime reads the credential and stops there. `BearerTokenExtractable`
/// pulls the token out of `Authorization: Bearer ...` and answers **401** with
/// the `WWW-Authenticate` challenge the specification requires when it is
/// absent or malformed. What a token *means* — which user, which permissions —
/// is yours, because that is a product decision and no runtime can make it.
///
/// The two statuses are not interchangeable:
///
/// * **401** — no credential, or one that is not a credential. Retry with one.
/// * **403** — a real credential that is not allowed here. Retrying is pointless.
///
/// A client that retries on 401 loops forever against a wrong token if you
/// answer 401 for both.
///
/// > **Compare secrets in constant time.** `==` stops at the first wrong byte,
/// > and the timing difference leaks the token one character at a time. This is
/// > not theoretical over a fast local network.
///
/// Run it with `dart run example/bearer_auth.dart`:
///
/// ```bash
/// curl -s localhost:8080/me -H 'authorization: Bearer t-ada'
/// curl -i localhost:8080/me                                  # 401 + challenge
/// curl -i localhost:8080/me -H 'authorization: Basic abc'     # 401, wrong scheme
/// curl -i localhost:8080/me -H 'authorization: Bearer nope'   # 403
/// curl -s localhost:8080/public
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  return Router()
    ..route('/me', get(me))
    ..route('/public', get(public))
    ..withState(const Tokens({'t-ada': 'ada', 't-grace': 'grace'}));
}

/// `GET /me`
Future<Map<String, Object?>> me(Request request) async {
  final tokens = await request.state<Tokens>();
  final user = await request.extract(TokenAuth(tokens));

  return {'user': user};
}

/// `GET /public` — no credential asked for, so none required.
Map<String, Object?> public(Request request) => const {'anyone': true};

/// Which tokens exist, standing in for a database.
final class Tokens {
  /// Creates a [Tokens].
  const Tokens(this.users);

  /// Token to user name.
  final Map<String, String> users;

  /// The user a token names, compared without leaking where it differs.
  String? userFor(String given) {
    for (final entry in users.entries) {
      if (_constantTimeEquals(given, entry.key)) return entry.value;
    }
    return null;
  }

  /// Compares without stopping at the first difference.
  static bool _constantTimeEquals(String a, String b) {
    if (a.length != b.length) return false;

    var difference = 0;
    for (var index = 0; index < a.length; index++) {
      difference |= a.codeUnitAt(index) ^ b.codeUnitAt(index);
    }
    return difference == 0;
  }
}

/// Turns a bearer token into a user name.
final class TokenAuth implements FromRequestParts<String> {
  /// Checks against [tokens].
  const TokenAuth(this.tokens);

  /// The tokens that exist.
  final Tokens tokens;

  @override
  Future<Result<String, Rejection>> extract(Request request) async {
    // The class, so a failure is an Err this can pass on rather than a throw.
    switch (await const BearerTokenExtractable().extract(request)) {
      case Err(:final error):
        return Err(error);
      case Ok(value: final token):
        final user = tokens.userFor(token);
        return user == null
            ? const Err(Rejection.forbidden('unknown token'))
            : Ok(user);
    }
  }
}
